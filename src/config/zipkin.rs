use super::env_var::get_config;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub fn configure_tracing(level: String) {
    let env = get_config();
    opentelemetry::global::set_text_map_propagator(opentelemetry_zipkin::Propagator::new());

    let tracer = opentelemetry_zipkin::new_pipeline()
        .with_service_name(env.app.name)
        .with_collector_endpoint("http://zipkin:9411/api/v2/spans")
        .install_simple()
        .expect("unable to install zipkin tracer");

    let tracer = tracing_opentelemetry::layer().with_tracer(tracer);
    let level = EnvFilter::new(level);

    let subscriber = tracing_subscriber::fmt::layer();
    tracing_subscriber::registry()
        .with(subscriber)
        .with(level)
        .with(tracer)
        .init();
}
