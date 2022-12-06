use crate::core::{
    self,
    dto::payment_method::{Method, PaymentMethodInfo},
};
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreatePaymentMethodInputSchema {
    pub external_user_id: String,
    pub is_default: bool,
    pub method: String,
    pub info: PaymentMethodInfo,
}

#[post("/payment_method")]
pub async fn create_payment_method(
    json: web::Json<CreatePaymentMethodInputSchema>,
) -> impl Responder {
    let core_x = core::get_core().await;
    let method = Method::from(json.method.as_str());
    println!("{:?}", json.info);
    match core_x
        .payment_method
        .create
        .create(&json.external_user_id, json.is_default, method, &json.info)
        .await
    {
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
