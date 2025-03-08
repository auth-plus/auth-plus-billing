use super::env_var::get_config;
use opentelemetry_zipkin::ZipkinExporter;
use opentelemetry_sdk::{trace::SdkTracerProvider, Resource};
use opentelemetry::global;
use opentelemetry_sdk::trace::RandomIdGenerator;
pub fn configure_tracing() {

    let env = get_config();
    global::set_text_map_propagator(opentelemetry_zipkin::Propagator::new());
    let exporter = ZipkinExporter::builder().with_collector_endpoint("http://zipkin:9411/api/v2/spans").build().expect("fail on building exporter");
    let provider = SdkTracerProvider::builder()
        .with_simple_exporter(exporter)
        .with_id_generator(RandomIdGenerator::default())
        .with_resource(
            Resource::builder_empty()
                .with_service_name(env.app.name)
                .build(),
        )
        .build();

    global::set_tracer_provider(provider.clone());
}
