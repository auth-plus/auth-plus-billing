use crate::core::{self, dto::invoice_item::InvoiceItem};
use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateInvoiceInputSchema {
    pub user_id: String,
    pub itens: Vec<InvoiceItem>,
}

#[post("/invoice")]
pub async fn create_invoice(json: web::Json<CreateInvoiceInputSchema>) -> impl Responder {
    let core_x = core::get_core().await;
    match core_x
        .invoice_usecase
        .create_invoice(&json.user_id, &json.itens)
        .await
    {
        Ok(invoce) => {
            let json = web::Json(invoce);
            HttpResponse::Ok().json(json)
        }
        Err(error) => {
            let resp = format!("Something wrong happen: {}", error);
            HttpResponse::InternalServerError().body(resp)
        }
    }
}

#[derive(Serialize)]
struct GetInvoiceOutputSchema {
    invoces: Vec<core::dto::invoice::Invoice>,
}

#[get("/invoice/{user_id}")]
pub async fn get_invoice(user_id: web::Path<String>) -> impl Responder {
    let core_x = core::get_core().await;
    match core_x.invoice_usecase.get_by_user_id(&user_id).await {
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
