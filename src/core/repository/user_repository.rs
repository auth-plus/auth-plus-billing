use crate::core::dto::user::User;
use crate::core::usecase::driven::reading_user::{ReadingUser, ReadingUserError};
pub use sqlx::postgres::PgPool;

#[derive(sqlx::FromRow)]
struct UserDAO {
    id: String,
    external_id: String,
}

pub struct UserRepository {
    conn: PgPool,
}

#[async_trait::async_trait]
impl ReadingUser for UserRepository {
    async fn list_by_id(&self, external_id: &str) -> Result<User, ReadingUserError> {
        let result = sqlx::query_as::<_, UserDAO>("SELECT * FROM user WHERE external_id = ?")
            .bind(external_id)
            .fetch_one(&self.conn)
            .await;

        match result {
            Ok(user) => {
                let u = User {
                    id: String::from(&user.id),
                    external_id: String::from(&user.external_id),
                };
                Ok(u)
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
