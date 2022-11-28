use crate::core::dto::invoice::Invoice;
use uuid::Uuid;

#[mockall::automock]
#[async_trait::async_trait]
pub trait ReadingInvoice {
    async fn list_by_user_id(&self, user_id: Uuid) -> Result<Vec<Invoice>, ReadingInvoiceError>;
    async fn get_by_id(&self, invoice_id: Uuid) -> Result<Invoice, ReadingInvoiceError>;
}

#[derive(Debug, Clone, Copy)]
pub enum ReadingInvoiceError {
    InvoiceNotFoundError,
    UnmappedError,
}
