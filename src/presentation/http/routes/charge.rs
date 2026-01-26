use crate::core;
use actix_web::{HttpResponse, Responder, post, web};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct CreateChargeInputSchema {
    pub invoice_id: String,
}

#[post("/charge")]
pub async fn create_charge(json: web::Json<CreateChargeInputSchema>) -> impl Responder {
    let core_x = core::get_core().await;
    let result = core_x.charge.create.create_charge(&json.invoice_id).await;
    match result {
        Ok(charge) => {
            let json = web::Json(charge);
            HttpResponse::Ok().json(json)
        }
        Err(error) => {
            let resp = format!("Something wrong happen: {}", error);
            HttpResponse::InternalServerError().body(resp)
        }
    }
}
