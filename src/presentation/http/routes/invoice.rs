use crate::core::{self, dto::invoice_item::InvoiceItem};
use actix_web::{HttpResponse, Responder, get, patch, post, web};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct CreateInvoiceInputSchema {
    pub external_user_id: String,
    pub itens: Vec<InvoiceItem>,
    pub idempotency_key: String,
}

#[post("/invoice")]
pub async fn create_invoice(json: web::Json<CreateInvoiceInputSchema>) -> impl Responder {
    let core_x = core::get_core().await;
    let result = core_x
        .invoice
        .create
        .create_invoice(&json.external_user_id, &json.itens, &json.idempotency_key)
        .await;
    match result {
        Ok(invoice) => {
            let json = web::Json(invoice);
            HttpResponse::Ok().json(json)
        }
        Err(error) => {
            let resp = format!("Something wrong happen: {}", error);
            HttpResponse::InternalServerError().body(resp)
        }
    }
}

#[derive(Serialize)]
pub struct GetInvoiceOutputSchema {
    invoices: Vec<core::dto::invoice::Invoice>,
}

#[get("/invoice/{user_id}")]
pub async fn get_invoice(external_user_id: web::Path<String>) -> impl Responder {
    let core_x = core::get_core().await;
    let result = core_x.invoice.list.get_by_user_id(&external_user_id).await;
    match result {
        Ok(invoices) => {
            let resp = GetInvoiceOutputSchema { invoices };
            let json = web::Json(resp);
            HttpResponse::Ok().json(json)
        }
        Err(error) => {
            let resp = format!("Something wrong happen: {}", error);
            HttpResponse::InternalServerError().body(resp)
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct UpdateInvoiceInputSchema {
    pub invoice_id: String,
    pub status: String,
}

#[patch("/invoice")]
pub async fn update_invoice(json: web::Json<UpdateInvoiceInputSchema>) -> impl Responder {
    let core_x = core::get_core().await;
    let result = core_x
        .invoice
        .update
        .update(&json.invoice_id, &json.status)
        .await;
    match result {
        Ok(invoice) => {
            let json = web::Json(invoice);
            HttpResponse::Ok().json(json)
        }
        Err(error) => {
            let resp = format!("Something wrong happen: {}", error);
            HttpResponse::InternalServerError().body(resp)
        }
    }
}
