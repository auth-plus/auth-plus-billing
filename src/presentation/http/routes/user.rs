use crate::core;
use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateUserInputSchema {
    pub external_id: String,
}

#[post("/user")]
pub async fn create_user(json: web::Json<CreateUserInputSchema>) -> impl Responder {
    let core_x = core::get_core().await;
    match core_x.user.create.create_user(&json.external_id).await {
        Ok(user) => {
            let json = web::Json(user);
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
    match core_x.invoice.list.get_by_user_id(&user_id).await {
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
