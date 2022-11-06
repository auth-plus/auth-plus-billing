pub mod dto;
pub mod repository;
pub mod usecase;

use crate::config::database::get_connection;

pub struct Core {
    pub invoice_usecase: usecase::invoice_usecase::InvoiceUsecase,
}

pub async fn get_core() -> Core {
    // Let start transaction
    let conn = get_connection().await;

    // repositories
    let invoice_repository = repository::invoice_repository::InvoiceRepository::new(conn.clone());
    let user_repository = repository::user_repository::UserRepository::new(conn);

    //usecases
    let invoice_usecase = usecase::invoice_usecase::InvoiceUsecase {
        reading_user: Box::new(user_repository),
        reading_invoice: Box::new(invoice_repository),
    };

    Core { invoice_usecase }
}
