use uuid::Uuid;

use crate::core::{
    dto::{invoice::Invoice, invoice_item::InvoiceItem},
    usecase::driven::creating_invoice::{CreatingInvoice, CreatingInvoiceError},
    usecase::driven::reading_invoice::{ReadingInvoice, ReadingInvoiceError},
    usecase::driven::reading_user::{ReadingUser, ReadingUserError},
};

pub struct InvoiceUsecase {
    pub reading_user: Box<dyn ReadingUser>,
    pub reading_invoice: Box<dyn ReadingInvoice>,
    pub creating_invoice: Box<dyn CreatingInvoice>,
}

impl InvoiceUsecase {
    pub async fn get_by_user_id(&self, external_user_id_str: &str) -> Result<Vec<Invoice>, String> {
        let user_id = match Uuid::parse_str(external_user_id_str) {
            Ok(id) => id,
            Err(_error) => return Err(String::from("external id provided isn't uuid")),
        };
        let result_user = self.reading_user.list_by_id(&user_id).await;
        let user = match result_user {
            Ok(user) => user,
            Err(error) => match error {
                ReadingUserError::UserNotFoundError => return Err(String::from("User Not found")),
                ReadingUserError::UnmappedError => {
                    return Err(String::from(
                        "ReadingUserError::UnmappedError Something wrong happen",
                    ))
                }
            },
        };
        let result_invoice = self.reading_invoice.list_by_user_id(user.id).await;
        match result_invoice {
            Ok(invoices) => Ok(invoices),
            Err(error) => match error {
                ReadingInvoiceError::InvoiceNotFoundError => Err(String::from("Invoice Not found")),
                ReadingInvoiceError::UnmappedError => Err(String::from(
                    "ReadingInvoiceError::UnmappedError Something wrong happen",
                )),
            },
        }
    }

    pub async fn create_invoice(
        &self,
        external_user_id_str: &str,
        itens: &Vec<InvoiceItem>,
    ) -> Result<Invoice, String> {
        let user_id = match Uuid::parse_str(external_user_id_str) {
            Ok(id) => id,
            Err(_error) => return Err(String::from("external id provided isn't uuid")),
        };
        let result_user = self.reading_user.list_by_id(&user_id).await;
        let user = match result_user {
            Ok(user) => user,
            Err(error) => match error {
                ReadingUserError::UserNotFoundError => return Err(String::from("User Not found")),
                ReadingUserError::UnmappedError => {
                    return Err(String::from(
                        "ReadingUserError::UnmappedError Something wrong happen",
                    ))
                }
            },
        };
        let result_invoice = self.creating_invoice.create(&user.id, itens).await;
        match result_invoice {
            Ok(invoice) => Ok(invoice),
            Err(error) => match error {
                CreatingInvoiceError::InvoiceNotFoundError => {
                    Err(String::from("Invoice Not found"))
                }
                CreatingInvoiceError::UnmappedError => Err(String::from(
                    "CreatingInvoiceError::UnmappedError Something wrong happen",
                )),
            },
        }
    }
}

#[cfg(test)]
mod test {

    use super::InvoiceUsecase;
    use crate::core::{
        dto::{invoice::Invoice, user::User},
        usecase::driven::{
            creating_invoice::MockCreatingInvoice, reading_invoice::MockReadingInvoice,
            reading_user::MockReadingUser,
        },
    };
    use mockall::predicate;
    use uuid::Uuid;

    #[actix_rt::test]
    async fn should_succeed_listing_invoice_by_user() {
        let id = Uuid::new_v4();
        let external_id = Uuid::new_v4();
        let user = User {
            id,
            external_id: Uuid::new_v4(),
        };
        let invoices = vec![Invoice {
            id: Uuid::new_v4(),
            status: String::from("pending"),
            user_id: id,
            itens: Vec::new(),
        }];
        let mut mock_ru = MockReadingUser::new();
        mock_ru
            .expect_list_by_id()
            .with(predicate::eq(external_id))
            .times(1)
            .return_const(Ok(user.clone()));
        let mut mock_ri = MockReadingInvoice::new();
        mock_ri
            .expect_list_by_user_id()
            .with(predicate::eq(user.id))
            .times(1)
            .return_const(Ok(invoices.clone()));
        let mut mock_ci = MockCreatingInvoice::new();
        mock_ci.expect_create().times(0);
        let invoice_usecase = InvoiceUsecase {
            reading_user: Box::new(mock_ru),
            reading_invoice: Box::new(mock_ri),
            creating_invoice: Box::new(mock_ci),
        };
        let result = invoice_usecase
            .get_by_user_id(&external_id.to_string())
            .await;

        match result {
            Ok(resp) => {
                let inv = invoices[0].clone();
                assert_eq!(inv.id, resp[0].id);
                assert_eq!(inv.status, resp[0].status);
                assert_eq!(inv.user_id, resp[0].user_id);
            }
            Err(error) => panic!("Test wen wrong: {}", error),
        }
    }
}
