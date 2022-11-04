use crate::core::dto::invoice::Invoice;
use crate::core::usecase::driven::reading_invoice::{ReadingInvoice, ReadingInvoiceError};
pub use sqlx::postgres::PgPool;

#[derive(sqlx::FromRow)]
struct InvoiceDAO {
    id: String,
    user_id: String,
    status: String,
}

pub struct InvoiceRepository {
    conn: PgPool,
}

#[async_trait::async_trait]
impl ReadingInvoice for InvoiceRepository {
    async fn list_by_user_id(&self, user_id: String) -> Result<Vec<Invoice>, ReadingInvoiceError> {
        let result = sqlx::query_as::<_, InvoiceDAO>("SELECT * FROM invoice WHERE user_id = ?")
            .bind(user_id)
            .fetch_all(&self.conn)
            .await;

        let list = match result {
            Ok(list) => list,
            Err(_err) => return Err(ReadingInvoiceError::UnmappedError),
        };
        let mapped_list = list
            .iter()
            .map(|x| Invoice {
                id: String::from(&x.id),
                status: String::from(&x.status),
                user_id: String::from(&x.user_id),
            })
            .collect();
        Ok(mapped_list)
    }
}

impl InvoiceRepository {
    pub fn new(conn: PgPool) -> Self {
        InvoiceRepository { conn }
    }
}
