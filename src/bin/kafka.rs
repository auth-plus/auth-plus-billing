use auth_plus_billing::presentation;

fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    presentation::kafka::start()
}
