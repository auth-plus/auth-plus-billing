pub mod routes;
use crate::config;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use routes::{charge, invoice, payment_method, user};

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
            .service(charge::create_charge)
            .service(invoice::get_invoice)
            .service(invoice::create_invoice)
            .service(invoice::update_invoice)
            .service(payment_method::create_payment_method)
            .service(user::create_user)
    })
    .bind(("0.0.0.0", config.app.port))?
    .run()
    .await
}
