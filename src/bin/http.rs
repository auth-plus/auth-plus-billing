use auth_plus_billing::presentation;
use tracing_subscriber;

fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    presentation::http::routes::start()
}
