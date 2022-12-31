use auth_plus_billing::{
    config::{prometheus::Prometheus, zipkin::configure_tracing},
    presentation,
};

fn main() -> std::io::Result<()> {
    Prometheus::init();
    configure_tracing("info".to_owned());
    presentation::http::start()
}
