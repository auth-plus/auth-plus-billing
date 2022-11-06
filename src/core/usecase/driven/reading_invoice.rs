use crate::core::dto::invoice::Invoice;
use uuid::Uuid;

#[mockall::automock]
#[async_trait::async_trait]
pub trait ReadingInvoice {
    async fn list_by_user_id(&self, user_id: Uuid) -> Result<Vec<Invoice>, ReadingInvoiceError>;
}

#[derive(Debug, Clone, Copy)]
pub enum ReadingInvoiceError {
    InvoiceNotFoundError,
    UnmappedError,
}
