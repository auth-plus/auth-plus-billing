use crate::core::dto::invoice::{Invoice, InvoiceStatus};
use uuid::Uuid;

#[mockall::automock]
#[async_trait::async_trait]
pub trait UpdatingInvoice {
    async fn update(
        &self,
        invoice_id: Uuid,
        status: InvoiceStatus,
    ) -> Result<Invoice, UpdatingInvoiceError>;
}

#[derive(Debug, Clone, Copy)]
pub enum UpdatingInvoiceError {
    UnmappedError,
}
