use crate::core::dto::invoice::{Invoice, InvoiceStatus};
use crate::core::dto::invoice_item::InvoiceItem;
use crate::core::usecase::driven::creating_invoice::{CreatingInvoice, CreatingInvoiceError};
use crate::core::usecase::driven::reading_invoice::{ReadingInvoice, ReadingInvoiceError};
use crate::core::usecase::driven::updating_invoice::{UpdatingInvoice, UpdatingInvoiceError};
pub use sqlx::postgres::PgPool;
use uuid::Uuid;

#[derive(sqlx::FromRow)]
struct InvoiceDAO {
    id: Option<Uuid>,
    user_id: Uuid,
    status: String,
}

#[derive(Clone)]
pub struct InvoiceRepository {
    conn: PgPool,
}

async fn list_by_user_id(
    conn: &PgPool,
    user_id: Uuid,
) -> Result<Vec<Invoice>, ReadingInvoiceError> {
    let result =
        sqlx::query_as::<_, InvoiceDAO>("SELECT * FROM invoice WHERE  user_id :: text = $1")
            .bind(user_id.to_string())
            .fetch_all(conn)
            .await;
    match result {
        Ok(list) => {
            let list_transformed = list
                .into_iter()
                .map(|x| Invoice {
                    id: x.id.unwrap(),
                    user_id: x.user_id,
                    status: InvoiceStatus::from(x.status.as_str()),
                })
                .collect();
            Ok(list_transformed)
        }
        Err(err) => {
            tracing::error!("InvoiceRepository.list_by_user_id :{:?}", err);
            Err(ReadingInvoiceError::UnmappedError)
        }
    }
}

async fn create(
    conn: &PgPool,
    user_id: &Uuid,
    itens: &[InvoiceItem],
) -> Result<Invoice, CreatingInvoiceError> {
    let invoice_id = Uuid::new_v4();
    let q_invoice = format!(
        "INSERT INTO invoice (id, user_id, status) VALUES ('{}','{}', 'draft');",
        invoice_id, user_id
    );
    let r_invoice = sqlx::query(&q_invoice).execute(conn).await;
    match r_invoice {
        Ok(_) => {}
        Err(error) => {
            tracing::error!("InvoiceRepository.create :{:?}", error);
            return Err(CreatingInvoiceError::UnmappedError);
        }
    }
    let mut insert_iten: Vec<Uuid> = Vec::new();
    for it in itens {
        let item_id = Uuid::new_v4();
        let q_invoice_item = format!(
            "INSERT INTO invoice_item (id, invoice_id, description, quantity, amount, currency) VALUES ('{}','{}', '{}', '{}', '{}', '{}');",
            item_id,
            invoice_id,
            it.description,
            it.quantity,
            it.amount,
            it.currency
        );
        let r_invoice_item = sqlx::query(&q_invoice_item).execute(conn).await;
        match r_invoice_item {
            Ok(_) => insert_iten.push(item_id),
            Err(error) => {
                tracing::error!("InvoiceRepository.create :{:?}", error);
                return Err(CreatingInvoiceError::UnmappedError);
            }
        }
    }
    let value = Invoice {
        id: invoice_id,
        status: InvoiceStatus::from("draft"),
        user_id: *user_id,
    };
    Ok(value)
}

async fn get_by_id(conn: &PgPool, invoice_id: Uuid) -> Result<Invoice, ReadingInvoiceError> {
    let result = sqlx::query_as::<_, InvoiceDAO>("SELECT * FROM invoice WHERE  id :: text = $1")
        .bind(invoice_id.to_string())
        .fetch_one(conn)
        .await;
    match result {
        Ok(invoice) => {
            let id = invoice.id.unwrap();
            let item = Invoice {
                id,
                user_id: invoice.user_id,
                status: InvoiceStatus::from(invoice.status.as_str()),
            };
            Ok(item)
        }
        Err(err) => {
            tracing::error!("InvoiceRepository.get_by_id :{:?}", err);
            Err(ReadingInvoiceError::UnmappedError)
        }
    }
}

async fn update(
    conn: &PgPool,
    invoice_id: Uuid,
    status: InvoiceStatus,
) -> Result<Invoice, UpdatingInvoiceError> {
    let q_invoice = format!(
        "UPDATE invoice SET status = '{}' WHERE id :: text = '{}' RETURNING id, user_id, status;",
        status, invoice_id,
    );
    let result_invoice = sqlx::query_as::<_, InvoiceDAO>(&q_invoice)
        .fetch_one(conn)
        .await;
    match result_invoice {
        Ok(r) => {
            let item = Invoice {
                id: invoice_id,
                user_id: r.user_id,
                status,
            };
            Ok(item)
        }
        Err(error) => {
            tracing::error!("InvoiceRepository.update :{:?}", error);
            Err(UpdatingInvoiceError::UnmappedError)
        }
    }
}

#[async_trait::async_trait]
impl ReadingInvoice for InvoiceRepository {
    async fn list_by_user_id(&self, user_id: Uuid) -> Result<Vec<Invoice>, ReadingInvoiceError> {
        list_by_user_id(&self.conn, user_id).await
    }
    async fn get_by_id(&self, invoice_id: Uuid) -> Result<Invoice, ReadingInvoiceError> {
        get_by_id(&self.conn, invoice_id).await
    }
}

#[async_trait::async_trait]
impl CreatingInvoice for InvoiceRepository {
    async fn create(
        &self,
        user_id: &Uuid,
        itens: &[InvoiceItem],
    ) -> Result<Invoice, CreatingInvoiceError> {
        create(&self.conn, user_id, itens).await
    }
}

#[async_trait::async_trait]
impl UpdatingInvoice for InvoiceRepository {
    async fn update(
        &self,
        invoice_id: Uuid,
        status: InvoiceStatus,
    ) -> Result<Invoice, UpdatingInvoiceError> {
        update(&self.conn, invoice_id, status).await
    }
}

impl InvoiceRepository {
    pub fn new(conn: PgPool) -> Self {
        InvoiceRepository { conn }
    }
}

#[cfg(test)]
mod test {

    use super::{create, get_by_id, list_by_user_id, InvoiceDAO};
    use crate::{
        config::database::get_connection,
        core::{
            dto::{invoice::InvoiceStatus, invoice_item::InvoiceItem},
            repository::orm::{create_invoice, create_user, delete_invoice, delete_user},
        },
    };
    use fake::{faker::lorem::en::Sentence, uuid::UUIDv4, Fake, Faker};
    use rust_decimal::Decimal;
    use uuid::Uuid;

    #[actix_rt::test]
    async fn should_list_invoices() {
        let conn = get_connection().await;
        let user_id: Uuid = UUIDv4.fake();
        let invoice_id: Uuid = UUIDv4.fake();
        let external_id: Uuid = UUIDv4.fake();
        create_user(&conn, user_id, external_id)
            .await
            .expect("should_list_invoices: user setup went wrong");
        create_invoice(&conn, invoice_id, user_id, InvoiceStatus::Draft)
            .await
            .expect("should_list_invoices: invoice setup went wrong");

        let result = list_by_user_id(&conn, user_id).await;

        match result {
            Ok(list) => {
                assert_eq!(list[0].user_id.to_string(), user_id.to_string());
                assert_eq!(list[0].id.to_string(), invoice_id.to_string());
                assert_eq!(list[0].status, InvoiceStatus::Draft);
            }
            Err(error) => panic!("should_list_invoices test went wrong : {:?}", error),
        };
        delete_invoice(&conn, invoice_id)
            .await
            .expect("should_list_invoices: invoice remove went wrong");
        delete_user(&conn, user_id)
            .await
            .expect("should_list_invoices: user remove went wrong");
    }

    #[actix_rt::test]
    async fn should_get_invoice_by_id() {
        let conn = get_connection().await;
        let user_id: Uuid = UUIDv4.fake();
        let invoice_id: Uuid = UUIDv4.fake();
        let external_id: Uuid = UUIDv4.fake();
        create_user(&conn, user_id, external_id)
            .await
            .expect("should_get_invoice_by_id: user setup went wrong");
        create_invoice(&conn, invoice_id, user_id, InvoiceStatus::Pending)
            .await
            .expect("should_get_invoice_by_id: invoice setup went wrong");

        let result = get_by_id(&conn, invoice_id).await;

        match result {
            Ok(inv) => {
                assert_eq!(inv.id.to_string(), invoice_id.to_string());
                assert_eq!(inv.user_id.to_string(), user_id.to_string());
                assert_eq!(inv.status, InvoiceStatus::from("pending"));
            }
            Err(error) => panic!("should_get_invoice_by_id test went wrong : {:?}", error),
        };

        delete_invoice(&conn, invoice_id)
            .await
            .expect("should_get_invoice_by_id: invoice remove went wrong");
        delete_user(&conn, user_id)
            .await
            .expect("should_get_invoice_by_id: user remove went wrong");
    }

    #[actix_rt::test]
    async fn should_create_invoices() {
        let conn = get_connection().await;
        let user_id: Uuid = UUIDv4.fake();
        let external_id: Uuid = UUIDv4.fake();
        let description: String = Sentence(3..5).fake();
        let quantity = Faker.fake::<u16>();
        let amount = Faker.fake::<f32>();
        let currency = "BRL";
        create_user(&conn, user_id, external_id)
            .await
            .expect("should_create_invoices: user setup went wrong");
        let item = InvoiceItem {
            id: None,
            quantity,
            description: description.clone(),
            amount: Decimal::from_f32_retain(amount).unwrap(),
            currency: String::from(currency),
        };
        let itens = Vec::from([item]);

        let result = create(&conn, &user_id, &itens).await;

        match result {
            Ok(invoice) => {
                assert!(!invoice.id.to_string().is_empty());
                assert_eq!(invoice.user_id.to_string(), user_id.to_string());
                assert_eq!(invoice.status.to_string(), String::from("draft"));

                let q_invoice = sqlx::query_as::<_, InvoiceDAO>(
                    "SELECT * FROM invoice WHERE  user_id :: text = $1",
                )
                .bind(user_id.to_string())
                .fetch_all(&conn)
                .await;
                match q_invoice {
                    Ok(list) => {
                        assert_eq!(list[0].user_id.to_string(), user_id.to_string());
                    }
                    Err(error) => panic!("should_create_invoices test went wrong {:?}", error),
                };
                delete_invoice(&conn, invoice.id)
                    .await
                    .expect("should_get_invoice_by_id: invoice remove went wrong");
            }
            Err(error) => panic!("should_create_invoices test went wrong {:?}", error),
        };

        delete_user(&conn, user_id)
            .await
            .expect("should_get_invoice_by_id: user remove went wrong");
    }
}
