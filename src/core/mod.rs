pub mod dto;
pub mod repository;
pub mod usecase;

pub struct Core {
    pub invoice_usecase: usecase::invoice_usecase::InvoiceUsecase,
}

pub fn get_core() -> Core {
    let invoice_repository = repository::invoice_repository::InvoiceRepository {};
    let user_repository = repository::user_repository::UserRepository {};

    //usecases
    let invoice_usecase = usecase::invoice_usecase::InvoiceUsecase {
        reading_user: Box::new(user_repository),
        reading_invoice: Box::new(invoice_repository),
    };

    Core { invoice_usecase }
}
