use auth_plus_billing::presentation;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    presentation::kafka::start().await
}
