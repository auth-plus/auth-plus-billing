pub mod routes;

use std::time::Duration;

use crate::config;
use actix_cors::Cors;
use actix_service::Service;
use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use lazy_static::lazy_static;
use opentelemetry::{global, propagation::Extractor};
use routes::{charge, invoice, payment_method, user};
use tracing_actix_web::{DefaultRootSpanBuilder, RootSpanBuilder, TracingLogger};
use tracing_opentelemetry::OpenTelemetrySpanExt;

struct HeaderMapExtractor<'a>(&'a actix_web::http::header::HeaderMap);

impl<'a> Extractor for HeaderMapExtractor<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|v| v.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(|k| k.as_str()).collect()
    }
}

pub struct RequestTracingSpanBuilder;

impl RootSpanBuilder for RequestTracingSpanBuilder {
    fn on_request_start(request: &actix_web::dev::ServiceRequest) -> tracing::Span {
        let span = tracing_actix_web::root_span!(level = tracing::Level::INFO, request);
        let parent_cx = global::get_text_map_propagator(|propagator| {
            propagator.extract(&HeaderMapExtractor(request.headers()))
        });
        let _ = span.set_parent(parent_cx);
        span
    }

    fn on_request_end<B: actix_web::body::MessageBody>(
        span: tracing::Span,
        outcome: &Result<actix_web::dev::ServiceResponse<B>, actix_web::Error>,
    ) {
        DefaultRootSpanBuilder::on_request_end(span, outcome);
    }
}

lazy_static! {
    static ref C_HTTP_SUCCESS: opentelemetry::metrics::Counter<u64> =
        global::meter("auth_plus_billing")
            .u64_counter("success_counter")
            .with_description("counter 20X")
            .build();
    static ref C_HTTP_FAIL: opentelemetry::metrics::Counter<u64> =
        global::meter("auth_plus_billing")
            .u64_counter("failed_counter")
            .with_description("counter 50X")
            .build();
}

async fn get_health_status() -> impl Responder {
    HttpResponse::Ok()
        .content_type("application/json")
        .body("Ok")
}

pub async fn start() -> std::io::Result<()> {
    let config = config::env_var::get_config();

    HttpServer::new(|| {
        App::new()
            .wrap(Cors::permissive())
            .wrap(TracingLogger::<RequestTracingSpanBuilder>::new())
            .wrap_fn(|req, srv| {
                let fut = srv.call(req);
                async {
                    let res = fut.await?;
                    if res.status().is_success() {
                        C_HTTP_SUCCESS.add(1, &[]);
                    } else {
                        C_HTTP_FAIL.add(1, &[]);
                    }
                    Ok(res)
                }
            })
            .route("/health", web::get().to(get_health_status))
            .service(charge::create_charge)
            .service(invoice::get_invoice)
            .service(invoice::create_invoice)
            .service(invoice::update_invoice)
            .service(payment_method::create_payment_method)
            .service(user::create_user)
    })
    .workers(4) // Number of worker threads
    .keep_alive(Duration::from_secs(75))
    .client_request_timeout(Duration::from_millis(5000))
    .client_disconnect_timeout(Duration::from_millis(5000))
    .bind(("0.0.0.0", config.app.port))?
    .run()
    .await
}
