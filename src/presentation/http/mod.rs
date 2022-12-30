pub mod routes;

use crate::config::{
    self,
    prometheus::{get_metrics, init_metrics},
    zipkin::get_tracer,
};
use actix_service::Service;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use opentelemetry::{
    trace::{FutureExt, TraceContextExt, Tracer},
    Key,
};

use routes::{charge, invoice, payment_method, user};
async fn get_health_status() -> impl Responder {
    HttpResponse::Ok()
        .content_type("application/json")
        .body("Ok")
}

#[actix_web::main]
pub async fn start() -> std::io::Result<()> {
    let config = config::env_var::get_config();
    init_metrics();
    HttpServer::new(|| {
        App::new()
            .wrap_fn(|req, srv| {
                let tr = get_tracer();
                let config = config::env_var::get_config();
                let attributes_list = Vec::from([
                    Key::new("environment").string(config.app.env),
                    Key::new("HTTP_URL").string(req.path().to_string()),
                    Key::new("HTTP_METHOD").string(req.method().to_string()),
                ]);
                tr.in_span("middleware", move |cx| {
                    cx.span().set_attributes(attributes_list.into_iter());
                    cx.span().add_event("HTTP_STARTED", vec![]);
                    srv.call(req).with_context(cx)
                })
            })
            .wrap_fn(|req, srv| {
                let fut = srv.call(req);
                async {
                    let res = fut.await?;
                    let (success_counter, failed_counter) = get_metrics();
                    if res.status().is_success() {
                        success_counter.inc();
                    } else {
                        failed_counter.inc()
                    }
                    Ok(res)
                }
            })
            .route("/health", web::get().to(get_health_status))
            .route("/metrics", web::get().to(get_health_status))
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
