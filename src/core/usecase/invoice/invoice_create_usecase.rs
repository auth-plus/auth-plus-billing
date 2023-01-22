use uuid::Uuid;

use crate::core::{
    dto::{invoice::Invoice, invoice_item::InvoiceItem},
    usecase::driven::creating_invoice::{CreatingInvoice, CreatingInvoiceError},
    usecase::driven::reading_user::{ReadingUser, ReadingUserError},
};

pub struct InvoiceCreateUsecase {
    pub reading_user: Box<dyn ReadingUser>,
    pub creating_invoice: Box<dyn CreatingInvoice>,
}

impl InvoiceCreateUsecase {
    pub async fn create_invoice(
        &self,
        external_user_id: &str,
        itens: &[InvoiceItem],
    ) -> Result<Invoice, String> {
        let user_id = match Uuid::parse_str(external_user_id) {
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
    use super::InvoiceCreateUsecase;
    use crate::core::{
        dto::{
            invoice::{Invoice, InvoiceStatus},
            invoice_item::InvoiceItem,
            user::User,
        },
        usecase::driven::{
            creating_invoice::{CreatingInvoiceError, MockCreatingInvoice},
            reading_user::{MockReadingUser, ReadingUserError},
        },
    };
    use fake::{faker::lorem::en::Sentence, uuid::UUIDv4, Fake, Faker};
    use mockall::predicate;
    use rust_decimal::Decimal;
    use uuid::Uuid;

    #[actix_rt::test]
    async fn should_succeed_creating_invoice_by_user() {
        let now = chrono::offset::Utc::now().to_string();
        let user_id: Uuid = UUIDv4.fake();
        let external_id: Uuid = UUIDv4.fake();
        let invoice_id: Uuid = UUIDv4.fake();
        let user = User {
            id: user_id,
            external_id,
        };
        let quantity = Faker.fake::<u16>();
        let amount = Faker.fake::<f32>();
        let description: String = Sentence(3..5).fake();
        let currency = "BRL";
        let item = InvoiceItem {
            id: None,
            amount: Decimal::from_f32_retain(amount).unwrap(),
            quantity,
            description: description.clone(),
            currency: String::from(currency),
        };
        let itens = Vec::from([item]);
        let invoice = Invoice {
            id: invoice_id,
            status: InvoiceStatus::from("draft"),
            user_id,
            created_at: now,
        };
        let mut mock_ru = MockReadingUser::new();
        mock_ru
            .expect_list_by_id()
            .with(predicate::eq(external_id))
            .times(1)
            .return_const(Ok(user.clone()));
        let mut mock_ci = MockCreatingInvoice::new();
        // TODO: fix predicate::always() to use predicate::eq(itens)
        mock_ci
            .expect_create()
            .with(predicate::eq(user_id), predicate::always())
            .times(1)
            .return_const(Ok(invoice.clone()));
        let invoice_usecase = InvoiceCreateUsecase {
            reading_user: Box::new(mock_ru),
            creating_invoice: Box::new(mock_ci),
        };
        let result = invoice_usecase
            .create_invoice(external_id.to_string().as_str(), &itens)
            .await;

        match result {
            Ok(resp) => {
                assert_eq!(user_id, resp.user_id);
                assert_eq!("draft", resp.status.to_string());
            }
            Err(error) => panic!("Test wen wrong: {}", error),
        }
    }

    #[actix_rt::test]
    async fn should_fail_when_uuid_is_wrong() {
        let amount = Faker.fake::<f32>();
        let description: String = Sentence(3..5).fake();
        let item = InvoiceItem {
            id: None,
            amount: Decimal::from_f32_retain(amount).unwrap(),
            quantity: Faker.fake::<u16>(),
            description: description.clone(),
            currency: String::from("BRL"),
        };
        let itens = Vec::from([item]);
        let mut mock_ru = MockReadingUser::new();
        mock_ru.expect_list_by_id().times(0);
        let mut mock_ci = MockCreatingInvoice::new();
        mock_ci.expect_create().times(0);
        let invoice_usecase = InvoiceCreateUsecase {
            reading_user: Box::new(mock_ru),
            creating_invoice: Box::new(mock_ci),
        };
        let result = invoice_usecase
            .create_invoice("a-hash-not-uuid", &itens)
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
        let quantity = Faker.fake::<u16>();
        let amount = Faker.fake::<f32>();
        let description: String = Sentence(3..5).fake();
        let currency = "BRL";
        let item = InvoiceItem {
            id: None,
            amount: Decimal::from_f32_retain(amount).unwrap(),
            quantity,
            description: description.clone(),
            currency: String::from(currency),
        };
        let itens = Vec::from([item]);
        let mut mock_ru = MockReadingUser::new();
        mock_ru
            .expect_list_by_id()
            .with(predicate::eq(external_id))
            .times(1)
            .return_const(Err(ReadingUserError::UserNotFoundError));
        let mut mock_ci = MockCreatingInvoice::new();
        mock_ci.expect_create().times(0);
        let invoice_usecase = InvoiceCreateUsecase {
            reading_user: Box::new(mock_ru),
            creating_invoice: Box::new(mock_ci),
        };
        let result = invoice_usecase
            .create_invoice(&external_id.to_string(), &itens)
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
        let user_id: Uuid = UUIDv4.fake();
        let external_id: Uuid = UUIDv4.fake();
        let user = User {
            id: user_id,
            external_id,
        };
        let quantity = Faker.fake::<u16>();
        let amount = Faker.fake::<f32>();
        let description: String = Sentence(3..5).fake();
        let currency = "BRL";
        let item = InvoiceItem {
            id: None,
            amount: Decimal::from_f32_retain(amount).unwrap(),
            quantity,
            description: description.clone(),
            currency: String::from(currency),
        };
        let itens = Vec::from([item]);
        let mut mock_ru = MockReadingUser::new();
        mock_ru
            .expect_list_by_id()
            .with(predicate::eq(external_id))
            .times(1)
            .return_const(Ok(user.clone()));
        let mut mock_ci = MockCreatingInvoice::new();
        // TODO: fix predicate::always() to use predicate::eq(itens)
        mock_ci
            .expect_create()
            .with(predicate::eq(user_id), predicate::always())
            .times(1)
            .return_const(Err(CreatingInvoiceError::InvoiceNotFoundError));
        let invoice_usecase = InvoiceCreateUsecase {
            reading_user: Box::new(mock_ru),
            creating_invoice: Box::new(mock_ci),
        };
        let result = invoice_usecase
            .create_invoice(&external_id.to_string(), &itens)
            .await;

        match result {
            Ok(_) => panic!("should_fail_when_invoice_provider_went_wrong test went wrong"),
            Err(error) => {
                assert_eq!(error, String::from("Invoice Not found"));
            }
        }
    }
}
