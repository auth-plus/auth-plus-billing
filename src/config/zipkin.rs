use super::env_var::get_config;

pub fn get_tracer() -> opentelemetry::sdk::trace::Tracer {
    let config = get_config();
    opentelemetry_zipkin::new_pipeline()
        .with_service_name(config.app.name)
        .install_simple()
        .expect("Failed to initialise tracer.")
}
