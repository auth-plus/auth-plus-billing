use uuid::Uuid;

#[mockall::automock]
#[async_trait::async_trait]
pub trait ReadingInvoiceItem {
    async fn get_sum_by_invoice_id(
        &self,
        invoice_id: Uuid,
    ) -> Result<rust_decimal::Decimal, ReadingInvoiceItemError>;
}

#[derive(Debug, Clone, Copy)]
pub enum ReadingInvoiceItemError {
    UnmappedError,
}
