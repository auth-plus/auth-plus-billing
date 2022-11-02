use crate::core::{
    dto::invoice::Invoice,
    usecase::driven::reading_invoice::{ReadingInvoice, ReadingInvoiceError},
    usecase::driven::reading_user::{ReadingUser, ReadingUserError},
};

pub struct InvoiceUsecase {
    pub reading_user: Box<dyn ReadingUser>,
    pub reading_invoice: Box<dyn ReadingInvoice>,
}

impl InvoiceUsecase {
    pub async fn get_by_user_id(&self, user_id: &str) -> Result<Vec<Invoice>, String> {
        let result = self.reading_user.list_by_id(user_id).await;
        let user = match result {
            Ok(user) => user,
            Err(error) => match error {
                ReadingUserError::UserNotFoundError => return Err(String::from("User Not found")),
                ReadingUserError::UnmappedError => {
                    return Err(String::from("Something wrong happen"))
                }
            },
        };
        let result = self.reading_invoice.list_by_user_id(user.id).await;
        match result {
            Ok(invoices) => Ok(invoices),
            Err(error) => match error {
                ReadingInvoiceError::InvoiceNotFoundError => Err(String::from("Invoice Not found")),
                ReadingInvoiceError::UnmappedError => Err(String::from("Something wrong happen")),
            },
        }
    }
}
