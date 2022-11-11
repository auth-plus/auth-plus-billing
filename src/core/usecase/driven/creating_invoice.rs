use crate::core::dto::{invoice::Invoice, invoice_item::InvoiceItem};
use uuid::Uuid;

#[mockall::automock]
#[async_trait::async_trait]
pub trait CreatingInvoice {
    async fn create(
        &self,
        user_id: &Uuid,
        itens: &Vec<InvoiceItem>,
    ) -> Result<Invoice, CreatingInvoiceError>;
}

#[derive(Debug, Clone, Copy)]
pub enum CreatingInvoiceError {
    InvoiceNotFoundError,
    UnmappedError,
}
