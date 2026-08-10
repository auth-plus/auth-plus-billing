use crate::config::env_var::get_config;
use opentelemetry::{global, trace::TracerProvider};
use opentelemetry_sdk::{
    trace::{RandomIdGenerator, SdkTracerProvider},
    metrics::{SdkMeterProvider, PeriodicReader},
    logs::SdkLoggerProvider,
    Resource,
};
use opentelemetry_otlp::{WithExportConfig, WithTonicConfig};
use std::time::Duration;
use tonic::metadata::MetadataMap;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

pub fn init_uptrace() {
    let env = get_config();

    if let Some(ref dsn) = env.app.uptrace_dsn {
        let mut metadata = MetadataMap::with_capacity(1);
        if let Ok(val) = dsn.parse() {
            metadata.insert("uptrace-dsn", val);
        }

        let endpoint = match reqwest::Url::parse(dsn) {
            Ok(url) => {
                let host = url.host_str().unwrap_or("localhost");
                let port = url.port().unwrap_or(4317);
                format!("{}://{}:{}", url.scheme(), host, port)
            }
            Err(e) => {
                eprintln!("Failed to parse UPTRACE_DSN: {}", e);
                return;
            }
        };

        // Configure traces
        let trace_exporter = match opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .with_endpoint(endpoint.clone())
            .with_metadata(metadata.clone())
            .build()
        {
            Ok(exporter) => exporter,
            Err(e) => {
                eprintln!("Failed to build OTLP trace exporter: {}", e);
                return;
            }
        };

        let tracer_provider = SdkTracerProvider::builder()
            .with_batch_exporter(trace_exporter)
            .with_id_generator(RandomIdGenerator::default())
            .with_resource(
                Resource::builder_empty()
                    .with_service_name(env.app.name.clone())
                    .build(),
            )
            .build();
        
        let tracer = tracer_provider.tracer(env.app.name.clone());
        global::set_tracer_provider(tracer_provider);

        // Configure metrics
        let metric_exporter = match opentelemetry_otlp::MetricExporter::builder()
            .with_tonic()
            .with_endpoint(endpoint.clone())
            .with_metadata(metadata.clone())
            .build()
        {
            Ok(exporter) => exporter,
            Err(e) => {
                eprintln!("Failed to build OTLP metric exporter: {}", e);
                return;
            }
        };

        let reader = PeriodicReader::builder(metric_exporter)
            .with_interval(Duration::from_secs(15))
            .build();

        let meter_provider = SdkMeterProvider::builder()
            .with_reader(reader)
            .with_resource(
                Resource::builder_empty()
                    .with_service_name(env.app.name.clone())
                    .build(),
            )
            .build();
        global::set_meter_provider(meter_provider);

        // Configure logs
        let log_exporter = match opentelemetry_otlp::LogExporter::builder()
            .with_tonic()
            .with_endpoint(endpoint)
            .with_metadata(metadata)
            .build()
        {
            Ok(exporter) => exporter,
            Err(e) => {
                eprintln!("Failed to build OTLP log exporter: {}", e);
                return;
            }
        };

        let logger_provider = SdkLoggerProvider::builder()
            .with_batch_exporter(log_exporter)
            .with_resource(
                Resource::builder_empty()
                    .with_service_name(env.app.name)
                    .build(),
            )
            .build();

        // Configure tracing subscriber
        let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
        
        let log_layer = opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge::new(&logger_provider);
        
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info"));
            
        let fmt_layer = tracing_subscriber::fmt::layer();

        Registry::default()
            .with(filter)
            .with(fmt_layer)
            .with(telemetry)
            .with(log_layer)
            .init();
    } else {
        println!("UPTRACE_DSN not set. Telemetry is not enabled.");
        
        // Setup default console logging when telemetry is disabled
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info"));
            
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .init();
    }
}
