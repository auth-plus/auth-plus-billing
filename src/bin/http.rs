use auth_plus_billing::{config::prometheus::Prometheus, presentation};

fn main() -> std::io::Result<()> {
    Prometheus::init();
    tracing_subscriber::fmt::init();
    presentation::http::start()
}
