pub mod routes;
use crate::config;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use routes::{invoice, user};

async fn get_health_status() -> impl Responder {
    HttpResponse::Ok()
        .content_type("application/json")
        .body("Ok")
}

#[actix_web::main]
pub async fn start() -> std::io::Result<()> {
    let config = config::env_var::get_config();
    HttpServer::new(|| {
        App::new()
            .route("/health", web::get().to(get_health_status))
            .service(invoice::get_invoice)
            .service(invoice::create_invoice)
            .service(user::create_user)
    })
    .bind(("0.0.0.0", config.app.port))?
    .run()
    .await
}
