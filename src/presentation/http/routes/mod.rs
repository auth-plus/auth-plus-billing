pub mod invoice;

use actix_web::{web, App, HttpServer};

#[actix_web::main]
pub async fn start() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/invoice/{user_id}", web::get().to(invoice::get_invoice)))
        .bind(("127.0.0.1", 3000))?
        .run()
        .await
}
