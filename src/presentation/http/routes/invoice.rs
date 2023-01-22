use crate::core::{
    self, dto::invoice_item::InvoiceItem,
    usecase::invoice::invoice_list_usecase::InvoiceFilterSchema,
};
use actix_web::{get, patch, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct CreateInvoiceInputSchema {
    pub external_user_id: String,
    pub itens: Vec<InvoiceItem>,
}

#[post("/invoice")]
pub async fn create_invoice(json: web::Json<CreateInvoiceInputSchema>) -> impl Responder {
    let core_x = core::get_core().await;
    match core_x
        .invoice
        .create
        .create_invoice(&json.external_user_id, &json.itens)
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
pub struct GetInvoiceOutputSchema {
    invoces: Vec<core::dto::invoice::Invoice>,
}

#[get("/invoice/{user_id}")]
pub async fn get_invoice(
    external_user_id: web::Path<String>,
    filter: web::Query<InvoiceFilterSchema>,
) -> impl Responder {
    let core_x = core::get_core().await;
    println!("{:?}", filter);
    match core_x
        .invoice
        .list
        .get_by_user_id(&external_user_id, &filter)
        .await
    {
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

#[derive(Deserialize, Serialize)]
pub struct UpdateInvoiceInputSchema {
    pub invoice_id: String,
    pub status: String,
}

#[patch("/invoice")]
pub async fn update_invoice(json: web::Json<UpdateInvoiceInputSchema>) -> impl Responder {
    let core_x = core::get_core().await;
    match core_x
        .invoice
        .update
        .update(&json.invoice_id, &json.status)
        .await
    {
        Ok(invoce) => {
            let json = web::Json(invoce);
            HttpResponse::Ok().json(json)
        }
        Err(error) => {
            let resp = format!("Something wrong happen: {}", error);
            println!("{:?}", resp);
            HttpResponse::InternalServerError().body(resp)
        }
    }
}
