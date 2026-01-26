pub mod routes;

use crate::config::{
    self,
    prometheus::{C_HTTP_FAIL, C_HTTP_SUCCESS, Prometheus},
};
use actix_cors::Cors;
use actix_service::Service;
use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use routes::{charge, invoice, payment_method, user};
use tracing_actix_web::TracingLogger;

async fn get_health_status() -> impl Responder {
    HttpResponse::Ok()
        .content_type("application/json")
        .body("Ok")
}

async fn get_metrics() -> impl Responder {
    let result = Prometheus::export();
    HttpResponse::Ok()
        .content_type("application/json")
        .body(result)
}

#[actix_web::main]
pub async fn start() -> std::io::Result<()> {
    let config = config::env_var::get_config();

    HttpServer::new(|| {
        App::new()
            .wrap(Cors::permissive())
            .wrap(TracingLogger::default())
            .wrap_fn(|req, srv| {
                let fut = srv.call(req);
                async {
                    let res = fut.await?;
                    if res.status().is_success() {
                        C_HTTP_SUCCESS.inc();
                    } else {
                        C_HTTP_FAIL.inc()
                    }
                    Ok(res)
                }
            })
            .route("/health", web::get().to(get_health_status))
            .route("/metrics", web::get().to(get_metrics))
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
