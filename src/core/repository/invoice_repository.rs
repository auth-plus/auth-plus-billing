use crate::core::dto::invoice::Invoice;
use crate::core::usecase::driven::reading_invoice::{ReadingInvoice, ReadingInvoiceError};
use rust_decimal_macros::dec;
pub struct InvoiceRepository {}

impl ReadingInvoice for InvoiceRepository {
    fn list_by_user_id(&self, _user_id: String) -> Result<Vec<Invoice>, ReadingInvoiceError> {
        //Get invoices from DB
        let inv = Invoice {
            id: String::from("asdasd"),
            value: dec!(1.64),
        };
        Ok(vec![inv])
    }
}
