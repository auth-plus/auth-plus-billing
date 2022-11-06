use crate::core::dto::invoice::Invoice;
use crate::core::usecase::driven::reading_invoice::{ReadingInvoice, ReadingInvoiceError};
pub use sqlx::postgres::PgPool;
use uuid::Uuid;

#[derive(sqlx::FromRow)]
struct InvoiceDAO {
    id: Uuid,
    user_id: Uuid,
    status: String,
}

pub struct InvoiceRepository {
    conn: PgPool,
}

async fn list_by_user_id(
    conn: &PgPool,
    user_id: Uuid,
) -> Result<Vec<Invoice>, ReadingInvoiceError> {
    let result = sqlx::query_as::<_, InvoiceDAO>("SELECT * FROM invoice WHERE user_id::text = $1")
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
    let mapped_list = list
        .iter()
        .map(|x| Invoice {
            id: x.id,
            status: String::from(&x.status),
            user_id: x.user_id,
        })
        .collect();
    Ok(mapped_list)
}

#[async_trait::async_trait]
impl ReadingInvoice for InvoiceRepository {
    async fn list_by_user_id(&self, user_id: Uuid) -> Result<Vec<Invoice>, ReadingInvoiceError> {
        list_by_user_id(&self.conn, user_id).await
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
