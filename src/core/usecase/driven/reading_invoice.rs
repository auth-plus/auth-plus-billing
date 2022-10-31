use crate::core::dto::invoice::Invoice;

pub trait ReadingInvoice {
    fn list_by_user_id(&self, user_id: String) -> Result<Vec<Invoice>, ReadingInvoiceError>;
}

#[derive(Debug, Clone, Copy)]
pub enum ReadingInvoiceError {
    InvoiceNotFoundError,
}
