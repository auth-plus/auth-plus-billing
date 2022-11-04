use crate::core::dto::invoice::Invoice;

#[async_trait::async_trait]
pub trait ReadingInvoice {
    async fn list_by_user_id(&self, user_id: String) -> Result<Vec<Invoice>, ReadingInvoiceError>;
}

#[derive(Debug, Clone, Copy)]
pub enum ReadingInvoiceError {
    InvoiceNotFoundError,
    UnmappedError,
}
