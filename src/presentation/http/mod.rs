pub mod routes;

use crate::config::{
    self,
    prometheus::{Prometheus, C_HTTP_FAIL, C_HTTP_SUCCESS},
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

async fn get_metrics() -> impl Responder {
    let result = Prometheus::export();
    HttpResponse::Ok()
        .content_type("application/json")
        .body(result)
}

#[actix_web::main]
pub async fn start() -> std::io::Result<()> {
    let config = config::env_var::get_config();
    HttpServer::new(move || {
        App::new()
            // .wrap_fn(move |req, srv| {
            //     let tr = get_tracer();
            //     let config = config::env_var::get_config();
            //     let attributes_list = Vec::from([
            //         Key::new("environment").string(config.app.env),
            //         Key::new("HTTP_URL").string(req.path().to_string()),
            //         Key::new("HTTP_METHOD").string(req.method().to_string()),
            //     ]);
            //     tr.in_span("middleware", move |cx| {
            //         cx.span().set_attributes(attributes_list.into_iter());
            //         cx.span().add_event("HTTP_STARTED", vec![]);
            //         srv.call(req).with_context(cx)
            //     })
            // })
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
