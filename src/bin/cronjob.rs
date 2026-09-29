use auth_plus_billing::{config::uptrace::init_uptrace, presentation};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut guard = init_uptrace();
    let result = presentation::cronjob::start().await;
    guard.shutdown();
    result
}
