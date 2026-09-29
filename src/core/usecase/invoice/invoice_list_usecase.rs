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
    #[tracing::instrument(skip(self))]
    pub async fn get_by_user_id(&self, external_user_id_str: &str) -> Result<Vec<Invoice>, String> {
        tracing::info!(
            "Starting to fetch invoices for external_user_id: {}",
            external_user_id_str
        );
        let user_id = match Uuid::parse_str(external_user_id_str) {
            Ok(id) => id,
            Err(_) => {
                tracing::error!("external id provided isn't uuid: {}", external_user_id_str);
                return Err(String::from("external id provided isn't uuid"));
            }
        };
        tracing::info!("Fetching user by id: {}", user_id);
        let result_user = self.reading_user.list_by_id(&user_id).await;
        let user = match result_user {
            Ok(user) => user,
            Err(error) => match error {
                ReadingUserError::UserNotFoundError => {
                    tracing::error!("User Not found: {}", external_user_id_str);
                    return Err(String::from("User Not found"));
                }
                ReadingUserError::UnmappedError => {
                    tracing::error!(
                        "ReadingUserError::UnmappedError Something wrong happen for user: {}",
                        external_user_id_str
                    );
                    return Err(String::from(
                        "ReadingUserError::UnmappedError Something wrong happen",
                    ));
                }
            },
        };
        tracing::info!("Fetching invoices for user: {}", user.id);
        let result_invoice = self.reading_invoice.list_by_user_id(user.id).await;
        match result_invoice {
            Ok(invoices) => {
                tracing::info!("Found {} invoices for user: {}", invoices.len(), user.id);
                Ok(invoices)
            }
            Err(error) => match error {
                ReadingInvoiceError::InvoiceNotFoundError => {
                    tracing::error!("Invoice Not found for user: {}", user.id);
                    Err(String::from("Invoice Not found"))
                }
                ReadingInvoiceError::UnmappedError => {
                    tracing::error!(
                        "ReadingInvoiceError::UnmappedError Something wrong happen for user: {}",
                        user.id
                    );
                    Err(String::from(
                        "ReadingInvoiceError::UnmappedError Something wrong happen",
                    ))
                }
            },
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_all_invoices_to_charge(&self) -> Result<Vec<Invoice>, String> {
        tracing::info!("Starting to fetch all invoices to charge");
        let result_invoice = self.reading_invoice.list_all_should_be_charged().await;
        match result_invoice {
            Ok(list) => {
                tracing::info!("Found {} invoices to charge", list.len());
                Ok(list)
            }
            Err(error) => match error {
                ReadingInvoiceError::InvoiceNotFoundError => {
                    tracing::error!("Invoice Not found when fetching invoices to charge");
                    Err(String::from("Invoice Not found"))
                }
                ReadingInvoiceError::UnmappedError => {
                    tracing::error!(
                        "ReadingInvoiceError::UnmappedError Something wrong happen when fetching invoices to charge"
                    );
                    Err(String::from(
                        "ReadingInvoiceError::UnmappedError Something wrong happen",
                    ))
                }
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
    use fake::{Fake, uuid::UUIDv4};
    use mockall::predicate;
    use uuid::Uuid;

    #[actix_rt::test]
    async fn should_succeed_listing_invoice_by_user() {
        let id: Uuid = UUIDv4.fake();
        let external_id: Uuid = UUIDv4.fake();
        let user = User { id, external_id };
        let now = time::OffsetDateTime::now_utc().to_string();
        let invoices = vec![Invoice {
            id: Uuid::new_v4(),
            status: InvoiceStatus::Pending,
            user_id: id,
            created_at: now,
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
