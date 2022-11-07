pub mod routes;
use crate::config;
use actix_web::{App, HttpServer};
use routes::invoice;

#[actix_web::main]
pub async fn start() -> std::io::Result<()> {
    let config = config::env_var::get_config();
    HttpServer::new(|| App::new().service(invoice::get_invoice))
        .bind(("127.0.0.1", config.app.port))?
        .run()
        .await
}
