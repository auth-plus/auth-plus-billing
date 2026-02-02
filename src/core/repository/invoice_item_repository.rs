use crate::core::dto::invoice_item::InvoiceItem;
use crate::core::usecase::driven::creating_invoice_item::{
    CreatingInvoiceItem, CreatingInvoiceItemError,
};
use log::error;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
pub use sqlx::postgres::PgPool;
use uuid::Uuid;

#[derive(sqlx::FromRow)]
pub struct InvoiceItemDAO {
    pub invoice_id: Uuid,
    pub description: String,
    pub quantity: u16,
    pub amount: Decimal,
    pub currency: String,
}

#[derive(Clone)]
pub struct InvoiceItemRepository {
    conn: PgPool,
}

async fn insert_item(
    conn: &PgPool,
    invoice_id: &Uuid,
    item: &InvoiceItem,
) -> Result<InvoiceItem, CreatingInvoiceItemError> {
    let item_id = Uuid::new_v4();
    match sqlx::query("INSERT INTO invoice_item (id, invoice_id, description, quantity, amount, currency) VALUES ($1, $2, $3, $4, $5, $6);").bind(item_id).bind(invoice_id).bind(item.description.clone()).bind(item.quantity.to_i32()).bind(item.amount.to_i32()).bind(item.currency.clone()).execute(conn).await {
        Ok(_) => (),
        Err(error) => {
            error!("InvoiceRepository.insert_item :{:?}", error);
            return Err(CreatingInvoiceItemError::UnmappedError);
        }
    }
    let value = InvoiceItem {
        id: Some(item_id),
        description: item.description.clone(),
        quantity: item.quantity,
        amount: item.amount,
        currency: item.currency.clone(),
    };
    Ok(value)
}

#[async_trait::async_trait]
impl CreatingInvoiceItem for InvoiceItemRepository {
    async fn insert_item(
        &self,
        invoice_id: &Uuid,
        item: &InvoiceItem,
    ) -> Result<InvoiceItem, CreatingInvoiceItemError> {
        insert_item(&self.conn, invoice_id, item).await
    }
}

impl InvoiceItemRepository {
    pub fn new(conn: PgPool) -> Self {
        InvoiceItemRepository { conn }
    }
}

#[cfg(test)]
mod test {

    use super::insert_item;
    use crate::{
        config::database::get_connection,
        core::{
            dto::{invoice::InvoiceStatus, invoice_item::InvoiceItem},
            repository::orm::{create_invoice, create_user, delete_invoice, delete_user},
        },
    };
    use fake::{Fake, Faker, faker::lorem::en::Sentence, uuid::UUIDv4};
    use rust_decimal::Decimal;
    use uuid::Uuid;

    #[actix_rt::test]
    async fn should_create_invoices() {
        let conn = get_connection().await;
        let user_id: Uuid = UUIDv4.fake();
        let invoice_id: Uuid = UUIDv4.fake();
        let external_id: Uuid = UUIDv4.fake();
        let description: String = Sentence(3..5).fake();
        let quantity = Faker.fake::<u16>();
        let amount = Faker.fake::<f32>();
        let currency = "BRL";
        create_user(&conn, user_id, external_id)
            .await
            .expect("should_create_invoices: user setup went wrong");
        create_invoice(&conn, invoice_id, user_id, InvoiceStatus::Draft)
            .await
            .expect("should_create_invoices: invoice setup went wrong");
        let item = InvoiceItem {
            id: None,
            quantity,
            description: description.clone(),
            amount: Decimal::from_f32_retain(amount).unwrap(),
            currency: String::from(currency),
        };

        let result = insert_item(&conn, &invoice_id, &item).await;

        match result {
            Ok(new_item) => {
                assert!(new_item.id.is_some());
                assert_eq!(new_item.quantity, quantity);
                assert_eq!(new_item.description, description);
                assert_eq!(new_item.amount, Decimal::from_f32_retain(amount).unwrap());
                assert_eq!(new_item.currency, String::from(currency));
            }
            Err(error) => panic!("should_create_invoices test went wrong {:?}", error),
        };
        delete_invoice(&conn, invoice_id)
            .await
            .expect("should_get_invoice_by_id: invoice remove went wrong");
        delete_user(&conn, user_id)
            .await
            .expect("should_get_invoice_by_id: user remove went wrong");
    }
}
