use crate::core;
use actix_web::{web, HttpResponse, Responder};
use serde::Serialize;

#[derive(Serialize)]
struct GetInvoiceOutputSchema {
    invoces: Vec<core::dto::invoice::Invoice>,
}

pub async fn get_invoice(user_id: web::Path<String>) -> impl Responder {
    match core::get_core().invoice_usecase.get_by_user_id(&user_id) {
        Ok(invoces) => {
            let resp = GetInvoiceOutputSchema { invoces };
            let json = web::Json(resp);
            HttpResponse::Ok().json(json)
        }
        Err(error) => {
            let resp = format!("Something wrong happen: {}", error);
            HttpResponse::InternalServerError().body(resp)
        }
    }
}
