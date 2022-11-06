use crate::core::dto::user::User;
use crate::core::usecase::driven::reading_user::{ReadingUser, ReadingUserError};
pub use sqlx::postgres::PgPool;
use uuid::Uuid;

#[derive(sqlx::FromRow)]
struct UserDAO {
    id: Uuid,
    external_id: Uuid,
}

pub struct UserRepository {
    conn: PgPool,
}

#[async_trait::async_trait]
impl ReadingUser for UserRepository {
    async fn list_by_id(&self, external_id: &Uuid) -> Result<User, ReadingUserError> {
        let result = sqlx::query_as::<_, UserDAO>("SELECT * FROM user WHERE external_id = $1")
            .bind(external_id.to_string())
            .fetch_one(&self.conn)
            .await;

        match result {
            Ok(dao) => {
                let u = User {
                    id: dao.id,
                    external_id: dao.external_id,
                };
                return Ok(u);
            }
            Err(err) => match err {
                sqlx::Error::RowNotFound => return Err(ReadingUserError::UserNotFoundError),
                _ => return Err(ReadingUserError::UnmappedError),
            },
        }
    }
}

impl UserRepository {
    pub fn new(conn: PgPool) -> Self {
        UserRepository { conn }
    }
}
