use crate::core::dto::{charge::Charge, invoice::Invoice, invoice_item::InvoiceItem};
use uuid::Uuid;

#[mockall::automock]
#[async_trait::async_trait]
pub trait UpdatingInvoice {
    async fn charge(
        &self,
        invoice_id: &Uuid,
        payment_method: &Uuid,
    ) -> Result<Charge, UpdatingInvoiceError>;

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
