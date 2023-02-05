use crate::core::{
    dto::invoice::Invoice, usecase::invoice::invoice_list_usecase::InvoiceFilterSchema,
};
use uuid::Uuid;

#[mockall::automock]
#[async_trait::async_trait]
pub trait ReadingInvoice {
    async fn list_by_user_id(
        &self,
        user_id: Uuid,
        filter: &InvoiceFilterSchema,
    ) -> Result<Vec<Invoice>, ReadingInvoiceError>;
    async fn get_by_id(&self, invoice_id: Uuid) -> Result<Invoice, ReadingInvoiceError>;
    async fn list_all_should_be_charged(&self) -> Result<Vec<Invoice>, ReadingInvoiceError>;
}

#[derive(Debug, Clone, Copy)]
pub enum ReadingInvoiceError {
    InvoiceNotFoundError,
    UnmappedError,
}
