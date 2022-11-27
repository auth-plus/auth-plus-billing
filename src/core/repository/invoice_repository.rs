use crate::core::dto::invoice::{Invoice, InvoiceStatus};
use crate::core::dto::invoice_item::InvoiceItem;
use crate::core::usecase::driven::creating_invoice::{CreatingInvoice, CreatingInvoiceError};
use crate::core::usecase::driven::reading_invoice::{ReadingInvoice, ReadingInvoiceError};
pub use sqlx::postgres::PgPool;
use uuid::Uuid;

#[derive(sqlx::FromRow)]
struct InvoiceDAO {
    invoice_id: Uuid,
    user_id: Uuid,
    status: String,
    invoice_item_id: Uuid,
}

#[derive(Clone)]
pub struct InvoiceRepository {
    conn: PgPool,
}

async fn list_by_user_id(
    conn: &PgPool,
    user_id: Uuid,
) -> Result<Vec<Invoice>, ReadingInvoiceError> {
    let result = sqlx::query_as::<_, InvoiceDAO>(
        "SELECT i.id as  invoice_id,
                i.user_id,
                i.status,
                ii.id as invoice_item_id
            FROM   invoice AS i
                inner join invoice_item AS ii
                        ON ii.invoice_id = i.id
            WHERE  user_id :: text = $1",
    )
    .bind(user_id.to_string())
    .fetch_all(conn)
    .await;

    let list = match result {
        Ok(list) => list,
        Err(err) => {
            tracing::error!("InvoiceRepository.list_by_user_id :{:?}", err);
            return Err(ReadingInvoiceError::UnmappedError);
        }
    };
    let mapped_list = list.into_iter().fold(Vec::new(), |mut out, curre| {
        let clone_out = out.clone();
        match clone_out
            .into_iter()
            .position(|x: Invoice| x.id == curre.invoice_id)
        {
            Some(idx) => {
                out[idx].itens.push(curre.invoice_item_id);
                out
            }
            None => {
                let invoice = Invoice {
                    id: curre.invoice_id,
                    itens: vec![curre.invoice_item_id],
                    status: InvoiceStatus::from(curre.status.as_str()),
                    user_id: curre.user_id,
                };
                out.push(invoice);
                out
            }
        }
    });
    Ok(mapped_list)
}

async fn create(
    conn: &PgPool,
    user_id: &Uuid,
    itens: &Vec<InvoiceItem>,
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
            tracing::error!("{:?}", error);
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
                tracing::error!("{:?}", error);
                return Err(CreatingInvoiceError::UnmappedError);
            }
        }
    }
    let value = Invoice {
        id: invoice_id,
        itens: insert_iten,
        status: InvoiceStatus::from("draft"),
        user_id: *user_id,
    };
    Ok(value)
}

#[async_trait::async_trait]
impl ReadingInvoice for InvoiceRepository {
    async fn list_by_user_id(&self, user_id: Uuid) -> Result<Vec<Invoice>, ReadingInvoiceError> {
        list_by_user_id(&self.conn, user_id).await
    }
}

#[async_trait::async_trait]
impl CreatingInvoice for InvoiceRepository {
    async fn create(
        &self,
        user_id: &Uuid,
        itens: &Vec<InvoiceItem>,
    ) -> Result<Invoice, CreatingInvoiceError> {
        create(&self.conn, user_id, itens).await
    }
}

impl InvoiceRepository {
    pub fn new(conn: PgPool) -> Self {
        InvoiceRepository { conn }
    }
}

#[cfg(test)]
mod test {

    use super::{create, list_by_user_id, InvoiceDAO};
    use crate::{
        config::database::get_connection,
        core::{
            dto::invoice_item::InvoiceItem, usecase::driven::reading_invoice::ReadingInvoiceError,
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
        let description: String = Sentence(3..5).fake();
        let quantity = Faker.fake::<i32>();
        let amount = Faker.fake::<f32>();
        let currency = String::from("BRL");
        let q_user = format!(
            "INSERT INTO \"user\" (id, external_id) VALUES ('{}', '{}');",
            user_id.to_string(),
            external_id.to_string()
        );
        sqlx::query(&q_user)
            .execute(&conn)
            .await
            .expect("should_list_invoices: user setup went wrong");
        let q_invoice = format!(
            "INSERT INTO invoice (id, user_id, status) VALUES ('{}', '{}', 'pending');",
            invoice_id.to_string(),
            user_id.to_string(),
        );
        sqlx::query(&q_invoice)
            .execute(&conn)
            .await
            .expect("should_list_invoices: invoice setup went wrong");
        let q_invoice_item = format!(
            "INSERT INTO invoice_item (invoice_id, description, quantity, amount, currency) VALUES ('{}', '{}', '{}', '{}', '{}');",
            invoice_id.to_string(),
            description,
            quantity,
            amount,
            currency
        );
        sqlx::query(&q_invoice_item)
            .execute(&conn)
            .await
            .expect("should_list_invoices: invoice setup went wrong");

        let result = list_by_user_id(&conn, user_id).await;

        match result {
            Ok(list) => {
                assert_eq!(list[0].user_id.to_string(), user_id.to_string())
            }
            Err(error) => match error {
                ReadingInvoiceError::InvoiceNotFoundError => panic!("Test did'n found"),
                ReadingInvoiceError::UnmappedError => panic!("Test went wrong"),
            },
        };
    }

    #[actix_rt::test]
    async fn should_create_invoices() {
        let conn = get_connection().await;
        let user_id: Uuid = UUIDv4.fake();
        let external_id: Uuid = UUIDv4.fake();
        let description: String = Sentence(3..5).fake();
        let quantity = Faker.fake::<i32>();
        let amount = Faker.fake::<f32>();
        let currency = "BRL";
        let q_user = format!(
            "INSERT INTO \"user\" (id, external_id) VALUES ('{}', '{}');",
            user_id.to_string(),
            external_id.to_string()
        );
        sqlx::query(&q_user)
            .execute(&conn)
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
                assert_eq!(invoice.user_id.to_string(), user_id.to_string());
                assert_eq!(invoice.status.to_string(), String::from("draft"));
            }
            Err(_) => {
                panic!("Test went wrong")
            }
        };

        let q_invoice = sqlx::query_as::<_, InvoiceDAO>(
            "SELECT i.id as  invoice_id,
                    i.user_id,
                    i.status,
                    ii.id as invoice_item_id,
                    ii.description,
                    ii.quantity,
                    ii.amount,
                    ii.currency
                FROM   invoice AS i
                    inner join invoice_item AS ii
                            ON ii.invoice_id = i.id
                WHERE  user_id :: text = $1",
        )
        .bind(user_id.to_string())
        .fetch_all(&conn)
        .await;

        match q_invoice {
            Ok(list) => {
                assert_eq!(list[0].user_id.to_string(), user_id.to_string())
            }
            Err(error) => match error {
                sqlx::Error::RowNotFound => panic!("Test didn't found"),
                _ => panic!("Test went wrong"),
            },
        };
    }
}
