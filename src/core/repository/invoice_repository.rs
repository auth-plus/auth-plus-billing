use crate::core::dto::invoice::Invoice;
use crate::core::dto::invoice_item::InvoiceItem;
use crate::core::usecase::driven::creating_invoice::{CreatingInvoice, CreatingInvoiceError};
use crate::core::usecase::driven::reading_invoice::{ReadingInvoice, ReadingInvoiceError};
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
pub use sqlx::postgres::PgPool;
use uuid::Uuid;

#[derive(sqlx::FromRow)]
struct InvoiceDAO {
    invoice_id: Uuid,
    user_id: Uuid,
    status: String,
    invoice_item_id: Uuid,
    description: String,
    quantity: i32,
    amount: f32,
    currency: String,
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
        let item = InvoiceItem {
            id: curre.invoice_item_id,
            amount: Decimal::from_f32(curre.amount).unwrap(),
            currency: curre.currency,
            description: curre.description,
            quantity: curre.quantity,
        };
        let clone_out = out.clone();
        match clone_out
            .into_iter()
            .position(|x: Invoice| x.id == curre.invoice_id)
        {
            Some(idx) => {
                out[idx].itens.push(item);
                out
            }
            None => {
                let invoice = Invoice {
                    id: curre.invoice_id,
                    itens: vec![item],
                    status: curre.status,
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
    let id = Uuid::new_v4();
    let q_invoice = format!(
        "INSERT INTO invoice (id, user_id, status) VALUES ('{}','{}', 'draft');",
        id.to_string(),
        user_id.to_string()
    );
    let r_invoice = sqlx::query(&q_invoice).execute(conn).await;
    match r_invoice {
        Ok(_) => {}
        Err(error) => {
            tracing::error!("{:?}", error);
            return Err(CreatingInvoiceError::UnmappedError);
        }
    }
    for it in itens {
        let q_invoice_item = format!(
            "INSERT INTO invoice_item (invoice_id, description, quantity, amount, currency) VALUES ('{}', '{}', '{}', '{}', '{}');",
            id.to_string(),
            it.description,
            it.quantity,
            it.amount,
            it.currency
        );
        let r_invoice_item = sqlx::query(&q_invoice_item).execute(conn).await;
        match r_invoice_item {
            Ok(_) => {}
            Err(error) => {
                tracing::error!("{:?}", error);
                return Err(CreatingInvoiceError::UnmappedError);
            }
        }
    }
    let value = Invoice {
        id,
        itens: itens.clone(),
        status: String::from("draft"),
        user_id: user_id.clone(),
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

    use super::list_by_user_id;
    use crate::{
        config::database::get_connection,
        core::usecase::driven::reading_invoice::ReadingInvoiceError,
    };
    use fake::{uuid::UUIDv4, Fake};
    use uuid::Uuid;

    #[actix_rt::test]
    async fn should_list_invoices() {
        let conn = get_connection().await;
        let user_id: Uuid = UUIDv4.fake();
        let external_id: Uuid = UUIDv4.fake();
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
            "INSERT INTO invoice (user_id, status) VALUES ('{}', 'pending');",
            user_id.to_string(),
        );
        sqlx::query(&q_invoice)
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
        }
    }
}
