pub mod invoice;
use crate::config;
use actix_web::{web, App, HttpServer};

#[actix_web::main]
pub async fn start() -> std::io::Result<()> {
    let config = config::env_var::get_config();
    HttpServer::new(|| App::new().route("/invoice/{user_id}", web::get().to(invoice::get_invoice)))
        .bind(("127.0.0.1", config.app.port))?
        .run()
        .await
}
