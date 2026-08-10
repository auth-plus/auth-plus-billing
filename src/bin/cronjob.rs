use auth_plus_billing::presentation;
#[tokio::main]
async fn main() -> std::io::Result<()> {
    presentation::cronjob::start().await
}
