use crate::core;
use actix_web::{post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct CreateUserInputSchema {
    pub external_id: String,
}

#[post("/user")]
pub async fn create_user(json: web::Json<CreateUserInputSchema>) -> impl Responder {
    let core_x = core::get_core().await;
    let result = core_x.user.create.create_user(&json.external_id).await;
    match result {
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
