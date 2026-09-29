use auth_plus_billing::{config::uptrace::init_uptrace, presentation};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let _guard = init_uptrace();
    presentation::kafka::start().await
}
