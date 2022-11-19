use crate::core::dto::{invoice::Invoice, invoice_item::InvoiceItem};
use uuid::Uuid;

#[mockall::automock]
#[async_trait::async_trait]
pub trait UpdatingInvoice {
    async fn update(
        &self,
        invoice_id: &Uuid,
        status: &Vec<InvoiceItem>,
    ) -> Result<Invoice, UpdatingInvoiceError>;
}

#[derive(Debug, Clone, Copy)]
pub enum UpdatingInvoiceError {
    InvoiceNotFoundError,
    UnmappedError,
}
