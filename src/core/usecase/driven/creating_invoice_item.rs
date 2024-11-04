use crate::core::dto::invoice_item::InvoiceItem;
use uuid::Uuid;

#[mockall::automock]
#[async_trait::async_trait]
pub trait CreatingInvoiceItem {
    async fn insert_item(
        &self,
        invoice_id: &Uuid,
        item: &InvoiceItem,
    ) -> Result<InvoiceItem, CreatingInvoiceItemError>;
}

#[derive(Debug, Clone, Copy)]
pub enum CreatingInvoiceItemError {
    UnmappedError,
}
