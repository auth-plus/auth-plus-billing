use auth_plus_billing::{config::uptrace::init_uptrace, presentation};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let _guard = init_uptrace();
    presentation::http::start().await
}
