use crate::config::cache;
use crate::core::{
    dto::{
        invoice::{Invoice, InvoiceStatus},
        invoice_item::InvoiceItem,
    },
    usecase::driven::{
        creating_invoice::{CreatingInvoice, CreatingInvoiceError},
        creating_invoice_item::CreatingInvoiceItem,
        reading_invoice::ReadingInvoice,
        reading_user::{ReadingUser, ReadingUserError},
    },
};
use log::error;
use redis::Commands;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub struct InvoiceInsertItemUsecase {
    pub reading_user: Box<dyn ReadingUser>,
    pub reading_invoice: Box<dyn ReadingInvoice>,
    pub creating_invoice: Box<dyn CreatingInvoice>,
    pub creating_invoice_item: Box<dyn CreatingInvoiceItem>,
}

#[derive(Serialize, Clone, Debug, Deserialize)]
pub enum InvoiceInsertItemResponse {
    Invoice(Invoice),
    Item(Vec<InvoiceItem>),
}

impl InvoiceInsertItemUsecase {
    pub async fn create_invoice(
        &self,
        external_user_id: &str,
        itens: &[InvoiceItem],
        idempotency_key: &str,
    ) -> Result<InvoiceInsertItemResponse, String> {
        let user_id = match Uuid::parse_str(external_user_id) {
            Ok(id) => id,
            Err(_error) => return Err(String::from("external id provided isn't uuid")),
        };
        let mut cache_client = cache::get_connection();
        let idempotency: Option<String> = cache_client
            .get(format!("create_invoice:{}", idempotency_key))
            .map_err(|e| format!("error fetching create_invoice cache: {}", e))?;
        if let Some(value) = idempotency {
            let v: InvoiceInsertItemResponse = serde_json::from_str(&value)
                .map_err(|e| format!("create_invoice error when parsing from cache: {}", e))?;
            return Ok(v);
        };
        let result_user = self.reading_user.list_by_id(&user_id).await;
        let user = match result_user {
            Ok(user) => user,
            Err(error) => match error {
                ReadingUserError::UserNotFoundError => return Err(String::from("User Not found")),
                ReadingUserError::UnmappedError => {
                    return Err(String::from(
                        "ReadingUserError::UnmappedError Something wrong happen",
                    ));
                }
            },
        };
        let current_invoices = self.reading_invoice.list_by_user_id(user.id).await;
        let does_exist: Option<Invoice> = match current_invoices {
            Ok(list) => {
                let draft_list: Vec<Invoice> = list
                    .into_iter()
                    .filter(|inv| inv.status == InvoiceStatus::Draft)
                    .collect();
                draft_list.first().cloned()
            }
            Err(_error) => return Err(String::from("external id provided isn't uuid")),
        };

        match does_exist {
            None => {
                let result_invoice = self
                    .creating_invoice
                    .create(&user.id, itens, idempotency_key)
                    .await;
                match result_invoice {
                    Ok(invoice) => {
                        let cache_value = serde_json::to_string(
                            &InvoiceInsertItemResponse::Invoice(invoice.clone()),
                        )
                        .map_err(|e| format!("Failed to serialize create_invoice cache: {}", e))?;
                        if let Err(e) = cache_client
                            .set::<_, _, ()>(format!("create_invoice:{}", idempotency_key), cache_value)
                        {
                            error!("create_invoice error when setting into cache: {:?}", e);
                        }
                        Ok(InvoiceInsertItemResponse::Invoice(invoice))
                    }
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
            Some(invoice) => {
                let mut item_list: Vec<InvoiceItem> = vec![];
                for item in itens.iter() {
                    let result_invoice_item = self
                        .creating_invoice_item
                        .insert_item(&invoice.id, item)
                        .await;
                    match result_invoice_item {
                        Ok(invoice_item) => item_list.push(invoice_item),
                        Err(_) => {
                            return Err(String::from(
                                "CreatingInvoiceError::UnmappedError Something wrong happen",
                            ));
                        }
                    }
                }
                let cache_value =
                    serde_json::to_string(&InvoiceInsertItemResponse::Item(item_list.clone()))
                        .map_err(|e| format!("Failed to serialize create_invoice cache: {}", e))?;
                let set_idempotency: Result<(), redis::RedisError> =
                    cache_client.set(format!("create_invoice:{}", idempotency_key), cache_value);
                if set_idempotency.is_err() {
                    error!("create_invoice error when setting into cache");
                }
                Ok(InvoiceInsertItemResponse::Item(item_list))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::InvoiceInsertItemUsecase;
    use crate::core::{
        dto::{
            invoice::{Invoice, InvoiceStatus},
            invoice_item::InvoiceItem,
            user::User,
        },
        usecase::{
            driven::{
                creating_invoice::MockCreatingInvoice,
                creating_invoice_item::MockCreatingInvoiceItem,
                reading_invoice::MockReadingInvoice, reading_user::MockReadingUser,
            },
            invoice::invoice_insert_item_usecase::InvoiceInsertItemResponse,
        },
    };
    use fake::{Fake, Faker, faker::lorem::en::Sentence, uuid::UUIDv4};
    use mockall::predicate;
    use rust_decimal::Decimal;
    use uuid::Uuid;

    #[actix_rt::test]
    async fn should_succeed_creating_invoice() {
        let now = time::OffsetDateTime::now_utc().to_string();
        let user_id: Uuid = UUIDv4.fake();
        let external_id: Uuid = UUIDv4.fake();
        let invoice_id: Uuid = UUIDv4.fake();
        let long_hash: String = (64..65).fake();
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
        let mut mock_ri = MockReadingInvoice::new();
        mock_ri
            .expect_list_by_user_id()
            .with(predicate::eq(user_id))
            .times(1)
            .return_const(Ok(Vec::from([])));
        let mut mock_ci = MockCreatingInvoice::new();
        mock_ci
            .expect_create()
            .with(
                predicate::eq(user_id),
                predicate::always(),
                predicate::eq(long_hash.clone()),
            )
            .times(1)
            .return_const(Ok(invoice.clone()));
        let mut mock_cii = MockCreatingInvoiceItem::new();
        mock_cii.expect_insert_item().times(0);
        let invoice_usecase = InvoiceInsertItemUsecase {
            reading_user: Box::new(mock_ru),
            reading_invoice: Box::new(mock_ri),
            creating_invoice: Box::new(mock_ci),
            creating_invoice_item: Box::new(mock_cii),
        };
        let result = invoice_usecase
            .create_invoice(external_id.to_string().as_str(), &itens, &long_hash)
            .await;

        match result {
            Ok(resp) => match resp {
                InvoiceInsertItemResponse::Invoice(invoice) => {
                    assert_eq!(user_id, invoice.user_id);
                    assert_eq!("draft", invoice.status.to_string());
                }
                InvoiceInsertItemResponse::Item(_) => {
                    panic!("Test went wrong: should not create invoice item")
                }
            },
            Err(error) => panic!("Test wen wrong: {}", error),
        }
    }

    #[actix_rt::test]
    async fn should_succeed_creating_invoice_item() {
        let now = time::OffsetDateTime::now_utc().to_string();
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
        let long_hash: String = (64..65).fake();
        let item = InvoiceItem {
            id: None,
            amount: Decimal::from_f32_retain(amount).unwrap(),
            quantity,
            description: description.clone(),
            currency: String::from(currency),
        };
        let itens = Vec::from([item.clone()]);
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
        let mut mock_ri = MockReadingInvoice::new();
        mock_ri
            .expect_list_by_user_id()
            .with(predicate::eq(user_id))
            .times(1)
            .return_const(Ok(Vec::from([invoice])));
        let mut mock_ci = MockCreatingInvoice::new();
        mock_ci.expect_create().times(0);
        let mut mock_cii = MockCreatingInvoiceItem::new();
        mock_cii
            .expect_insert_item()
            .with(predicate::eq(invoice_id), predicate::always())
            .times(1)
            .return_const(Ok(item.clone()));
        let invoice_usecase = InvoiceInsertItemUsecase {
            reading_user: Box::new(mock_ru),
            reading_invoice: Box::new(mock_ri),
            creating_invoice: Box::new(mock_ci),
            creating_invoice_item: Box::new(mock_cii),
        };
        let result = invoice_usecase
            .create_invoice(external_id.to_string().as_str(), &itens, &long_hash)
            .await;

        match result {
            Ok(resp) => match resp {
                InvoiceInsertItemResponse::Invoice(_) => {
                    panic!("Test went wrong: should not create invoice")
                }
                InvoiceInsertItemResponse::Item(new_item) => {
                    assert_eq!(new_item.len(), 1);
                    assert!(new_item[0].id.is_none());
                    assert_eq!(new_item[0].amount, item.amount);
                }
            },
            Err(error) => panic!("Test wen wrong: {}", error),
        }
    }
}
