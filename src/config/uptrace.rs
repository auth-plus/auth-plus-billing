use crate::config::env_var::get_config;
use opentelemetry::{global, trace::TracerProvider};
use opentelemetry_otlp::{Compression, WithExportConfig, WithTonicConfig};
use opentelemetry_sdk::{
    Resource,
    logs::SdkLoggerProvider,
    metrics::{PeriodicReader, SdkMeterProvider},
    propagation::TraceContextPropagator,
    trace::{RandomIdGenerator, SdkTracerProvider},
};
use std::time::Duration;
use tonic::metadata::MetadataMap;
use tracing_subscriber::{EnvFilter, Registry, layer::SubscriberExt, util::SubscriberInitExt};

/// Guard holding the OpenTelemetry providers to ensure all buffered traces,
/// metrics, and logs are flushed and cleanly exported upon shutdown.
#[must_use = "holding the TelemetryGuard ensures traces, metrics, and logs are flushed on shutdown"]
pub struct TelemetryGuard {
    tracer_provider: Option<SdkTracerProvider>,
    meter_provider: Option<SdkMeterProvider>,
    logger_provider: Option<SdkLoggerProvider>,
}

impl TelemetryGuard {
    pub fn shutdown(&mut self) {
        if let Some(tracer) = self.tracer_provider.take() {
            if let Err(e) = tracer.shutdown() {
                eprintln!("Error shutting down tracer provider: {:?}", e);
            }
        }
        if let Some(meter) = self.meter_provider.take() {
            if let Err(e) = meter.shutdown() {
                eprintln!("Error shutting down meter provider: {:?}", e);
            }
        }
        if let Some(logger) = self.logger_provider.take() {
            if let Err(e) = logger.shutdown() {
                eprintln!("Error shutting down logger provider: {:?}", e);
            }
        }
    }
}

impl Drop for TelemetryGuard {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Parse the OTLP gRPC endpoint from environment or UPTRACE_DSN.
pub fn parse_endpoint(dsn: &str) -> Result<String, String> {
    if let Ok(ep) = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT") {
        return Ok(ep);
    }
    let url = reqwest::Url::parse(dsn).map_err(|e| format!("Failed to parse UPTRACE_DSN: {e}"))?;
    let host = url.host_str().unwrap_or("localhost");
    let port = url
        .port()
        .or_else(|| {
            url.query_pairs()
                .find(|(k, _)| k == "grpc")
                .and_then(|(_, v)| v.parse().ok())
        })
        .unwrap_or(4317);
    Ok(format!("{}://{}:{}", url.scheme(), host, port))
}

pub fn init_uptrace() -> TelemetryGuard {
    // Route log crate macros (info!, warn!, error!) to tracing
    let _ = tracing_log::LogTracer::init();

    let env = get_config();

    if let Some(ref dsn) = env.app.uptrace_dsn {
        let mut metadata = MetadataMap::with_capacity(1);
        match dsn.parse() {
            Ok(val) => {
                metadata.insert("uptrace-dsn", val);
            }
            Err(e) => {
                eprintln!("Failed to parse UPTRACE_DSN into gRPC metadata: {}", e);
                return setup_fallback_subscriber();
            }
        }

        let endpoint = match parse_endpoint(dsn) {
            Ok(ep) => ep,
            Err(e) => {
                eprintln!("{}", e);
                return setup_fallback_subscriber();
            }
        };

        // Standard resource attributes recognized by Uptrace
        let resource = Resource::builder_empty()
            .with_service_name(env.app.name.clone())
            .with_attributes(vec![
                opentelemetry::KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                opentelemetry::KeyValue::new("deployment.environment", env.app.env.clone()),
            ])
            .build();

        // Configure W3C trace context propagator for distributed tracing
        global::set_text_map_propagator(TraceContextPropagator::new());

        // Configure traces
        let trace_exporter = match opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .with_endpoint(endpoint.clone())
            .with_metadata(metadata.clone())
            .with_compression(Compression::Gzip)
            .with_timeout(Duration::from_secs(10))
            .build()
        {
            Ok(exporter) => exporter,
            Err(e) => {
                eprintln!("Failed to build OTLP trace exporter: {}", e);
                return setup_fallback_subscriber();
            }
        };

        let tracer_provider = SdkTracerProvider::builder()
            .with_batch_exporter(trace_exporter)
            .with_id_generator(RandomIdGenerator::default())
            .with_resource(resource.clone())
            .build();

        let tracer = tracer_provider.tracer(env.app.name.clone());
        global::set_tracer_provider(tracer_provider.clone());

        // Configure metrics
        let metric_exporter = match opentelemetry_otlp::MetricExporter::builder()
            .with_tonic()
            .with_endpoint(endpoint.clone())
            .with_metadata(metadata.clone())
            .with_compression(Compression::Gzip)
            .with_timeout(Duration::from_secs(10))
            .build()
        {
            Ok(exporter) => exporter,
            Err(e) => {
                eprintln!("Failed to build OTLP metric exporter: {}", e);
                return setup_fallback_subscriber();
            }
        };

        let reader = PeriodicReader::builder(metric_exporter)
            .with_interval(Duration::from_secs(15))
            .build();

        let meter_provider = SdkMeterProvider::builder()
            .with_reader(reader)
            .with_resource(resource.clone())
            .build();

        global::set_meter_provider(meter_provider.clone());

        // Configure logs
        let log_exporter = match opentelemetry_otlp::LogExporter::builder()
            .with_tonic()
            .with_endpoint(endpoint)
            .with_metadata(metadata)
            .with_compression(Compression::Gzip)
            .with_timeout(Duration::from_secs(10))
            .build()
        {
            Ok(exporter) => exporter,
            Err(e) => {
                eprintln!("Failed to build OTLP log exporter: {}", e);
                return setup_fallback_subscriber();
            }
        };

        let logger_provider = SdkLoggerProvider::builder()
            .with_batch_exporter(log_exporter)
            .with_resource(resource)
            .build();

        // Configure tracing subscriber with console, OTel traces, and OTel logs
        let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);
        let log_layer = opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge::new(
            &logger_provider,
        );
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
        let fmt_layer = tracing_subscriber::fmt::layer();

        let _ = Registry::default()
            .with(filter)
            .with(fmt_layer)
            .with(telemetry_layer)
            .with(log_layer)
            .try_init();

        TelemetryGuard {
            tracer_provider: Some(tracer_provider),
            meter_provider: Some(meter_provider),
            logger_provider: Some(logger_provider),
        }
    } else {
        println!("UPTRACE_DSN not set. Telemetry is not enabled.");
        setup_fallback_subscriber()
    }
}

fn setup_fallback_subscriber() -> TelemetryGuard {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();

    TelemetryGuard {
        tracer_provider: None,
        meter_provider: None,
        logger_provider: None,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_endpoint_cloud_dsn() {
        let dsn = "https://secret_token@api.uptrace.dev/12345";
        let endpoint = parse_endpoint(dsn).unwrap();
        assert_eq!(endpoint, "https://api.uptrace.dev:4317");
    }

    #[test]
    fn test_parse_endpoint_query_grpc() {
        let dsn = "https://secret_token@api.uptrace.dev?grpc=4317";
        let endpoint = parse_endpoint(dsn).unwrap();
        assert_eq!(endpoint, "https://api.uptrace.dev:4317");
    }

    #[test]
    fn test_parse_endpoint_self_hosted() {
        let dsn = "http://secret_token@uptrace:14317/2";
        let endpoint = parse_endpoint(dsn).unwrap();
        assert_eq!(endpoint, "http://uptrace:14317");
    }

    #[test]
    fn test_parse_endpoint_invalid() {
        let dsn = "not a valid url";
        assert!(parse_endpoint(dsn).is_err());
    }

    #[test]
    fn test_fallback_guard_shutdown() {
        let mut guard = setup_fallback_subscriber();
        guard.shutdown();
        assert!(guard.tracer_provider.is_none());
        assert!(guard.meter_provider.is_none());
        assert!(guard.logger_provider.is_none());
    }
}
