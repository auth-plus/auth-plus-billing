use uuid::Uuid;

use crate::core::{
    dto::invoice::Invoice,
    usecase::driven::reading_invoice::{ReadingInvoice, ReadingInvoiceError},
    usecase::driven::reading_user::{ReadingUser, ReadingUserError},
};

pub struct InvoiceListUsecase {
    pub reading_user: Box<dyn ReadingUser>,
    pub reading_invoice: Box<dyn ReadingInvoice>,
}

impl InvoiceListUsecase {
    pub async fn get_by_user_id(&self, external_user_id_str: &str) -> Result<Vec<Invoice>, String> {
        let user_id = match Uuid::parse_str(external_user_id_str) {
            Ok(id) => id,
            Err(_) => return Err(String::from("external id provided isn't uuid")),
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
}

#[cfg(test)]
mod test {

    use super::InvoiceListUsecase;
    use crate::core::{
        dto::{
            invoice::{Invoice, InvoiceStatus},
            user::User,
        },
        usecase::driven::{
            reading_invoice::{MockReadingInvoice, ReadingInvoiceError},
            reading_user::{MockReadingUser, ReadingUserError},
        },
    };
    use fake::{uuid::UUIDv4, Fake};
    use mockall::predicate;
    use uuid::Uuid;

    #[actix_rt::test]
    async fn should_succeed_listing_invoice_by_user() {
        let id: Uuid = UUIDv4.fake();
        let external_id: Uuid = UUIDv4.fake();
        let user = User { id, external_id };
        let invoices = vec![Invoice {
            id: Uuid::new_v4(),
            status: InvoiceStatus::Pending,
            user_id: id,
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
        let invoice_usecase = InvoiceListUsecase {
            reading_user: Box::new(mock_ru),
            reading_invoice: Box::new(mock_ri),
        };
        let result = invoice_usecase
            .get_by_user_id(&external_id.to_string())
            .await;

        match result {
            Ok(resp) => {
                let inv = invoices[0].clone();
                assert_eq!(inv.id, resp[0].id);
                assert_eq!(inv.status.to_string(), resp[0].status.to_string());
                assert_eq!(inv.user_id, resp[0].user_id);
            }
            Err(error) => panic!("Test wen wrong: {}", error),
        }
    }

    #[actix_rt::test]
    async fn should_fail_when_uuid_is_wrong() {
        let mut mock_ru = MockReadingUser::new();
        mock_ru.expect_list_by_id().times(0);
        let mut mock_ri = MockReadingInvoice::new();
        mock_ri.expect_list_by_user_id().times(0);
        let invoice_usecase = InvoiceListUsecase {
            reading_user: Box::new(mock_ru),
            reading_invoice: Box::new(mock_ri),
        };
        let result = invoice_usecase
            .get_by_user_id("any-hash-that-is-not-uuid")
            .await;

        match result {
            Ok(_) => panic!("should_fail_when_uuid_is_wrong test went wrong"),
            Err(error) => {
                assert_eq!(error, String::from("external id provided isn't uuid"));
            }
        }
    }

    #[actix_rt::test]
    async fn should_fail_when_user_provider_went_wrong() {
        let external_id: Uuid = UUIDv4.fake();
        let mut mock_ru = MockReadingUser::new();
        mock_ru
            .expect_list_by_id()
            .with(predicate::eq(external_id))
            .times(1)
            .return_const(Err(ReadingUserError::UserNotFoundError));
        let mut mock_ri = MockReadingInvoice::new();
        mock_ri.expect_list_by_user_id().times(0);
        let invoice_usecase = InvoiceListUsecase {
            reading_user: Box::new(mock_ru),
            reading_invoice: Box::new(mock_ri),
        };
        let result = invoice_usecase
            .get_by_user_id(&external_id.to_string())
            .await;

        match result {
            Ok(_) => panic!("should_fail_when_user_provider_went_wrong test went wrong"),
            Err(error) => {
                assert_eq!(error, String::from("User Not found"));
            }
        }
    }

    #[actix_rt::test]
    async fn should_fail_when_invoice_provider_went_wrong() {
        let id: Uuid = UUIDv4.fake();
        let external_id: Uuid = UUIDv4.fake();
        let user = User { id, external_id };
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
            .return_const(Err(ReadingInvoiceError::InvoiceNotFoundError));
        let invoice_usecase = InvoiceListUsecase {
            reading_user: Box::new(mock_ru),
            reading_invoice: Box::new(mock_ri),
        };
        let result = invoice_usecase
            .get_by_user_id(&external_id.to_string())
            .await;

        match result {
            Ok(_) => panic!("should_fail_when_invoice_provider_went_wrong test went wrong"),
            Err(error) => {
                assert_eq!(error, String::from("Invoice Not found"));
            }
        }
    }
}
