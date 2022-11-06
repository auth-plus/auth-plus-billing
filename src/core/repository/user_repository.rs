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

async fn list_by_id(conn: &PgPool, external_id: &Uuid) -> Result<User, ReadingUserError> {
    let result =
        sqlx::query_as::<_, UserDAO>("SELECT * FROM \"user\" WHERE external_id::text = $1")
            .bind(external_id.to_string())
            .fetch_one(conn)
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
            error => {
                tracing::error!("{:?}", error);
                return Err(ReadingUserError::UnmappedError);
            }
        },
    }
}

#[async_trait::async_trait]
impl ReadingUser for UserRepository {
    async fn list_by_id(&self, external_id: &Uuid) -> Result<User, ReadingUserError> {
        list_by_id(&self.conn, external_id).await
    }
}

impl UserRepository {
    pub fn new(conn: PgPool) -> Self {
        UserRepository { conn }
    }
}

#[cfg(test)]
mod test {

    use super::list_by_id;
    use crate::{
        config::database::get_connection, core::usecase::driven::reading_user::ReadingUserError,
    };
    use fake::{uuid::UUIDv4, Fake};
    use uuid::Uuid;

    #[actix_rt::test]
    async fn should_get_user_by_external() {
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
            .expect("should_get_user_by_external: user setup went wrong");

        let result = list_by_id(&conn, &external_id).await;

        match result {
            Ok(user) => {
                assert_eq!(user.id.to_string(), user_id.to_string())
            }
            Err(error) => match error {
                ReadingUserError::UserNotFoundError => panic!("Test did'n found"),
                ReadingUserError::UnmappedError => panic!("Test went wrong"),
            },
        }
    }
}
