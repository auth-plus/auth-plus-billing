use crate::core::dto::user::User;
use crate::core::usecase::driven::creating_user::{CreatingUser, CreatingUserError};
use crate::core::usecase::driven::reading_user::{ReadingUser, ReadingUserError};
pub use sqlx::postgres::PgPool;
use uuid::Uuid;

#[derive(sqlx::FromRow)]
struct UserDAO {
    id: Uuid,
    external_id: Uuid,
}

#[derive(Clone)]
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
            Ok(u)
        }
        Err(err) => match err {
            sqlx::Error::RowNotFound => Err(ReadingUserError::UserNotFoundError),
            error => {
                tracing::error!("{:?}", error);
                Err(ReadingUserError::UnmappedError)
            }
        },
    }
}

async fn create(conn: &PgPool, external_id: &Uuid) -> Result<User, CreatingUserError> {
    let user_id = Uuid::new_v4();
    let q_user = format!(
        "INSERT INTO \"user\" (id, external_id) VALUES ('{}', '{}');",
        user_id, external_id
    );
    let result = sqlx::query(&q_user).execute(conn).await;
    match result {
        Ok(_) => {
            let u = User {
                id: user_id,
                external_id: *external_id,
            };
            Ok(u)
        }
        Err(_) => Err(CreatingUserError::UnmappedError),
    }
}

#[async_trait::async_trait]
impl ReadingUser for UserRepository {
    async fn list_by_id(&self, external_id: &Uuid) -> Result<User, ReadingUserError> {
        list_by_id(&self.conn, external_id).await
    }
}

#[async_trait::async_trait]
impl CreatingUser for UserRepository {
    async fn create(&self, external_id: &Uuid) -> Result<User, CreatingUserError> {
        create(&self.conn, external_id).await
    }
}

impl UserRepository {
    pub fn new(conn: PgPool) -> Self {
        UserRepository { conn }
    }
}

#[cfg(test)]
mod test {

    use super::{create, list_by_id};
    use crate::{
        config::database::get_connection,
        core::usecase::driven::{creating_user::CreatingUserError, reading_user::ReadingUserError},
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

    #[actix_rt::test]
    async fn should_create_user() {
        let conn = get_connection().await;
        let external_id: Uuid = UUIDv4.fake();

        let result = create(&conn, &external_id).await;

        match result {
            Ok(user) => {
                assert_eq!(user.external_id.to_string(), external_id.to_string());
                assert_eq!(user.id.to_string().is_empty(), false)
            }
            Err(error) => match error {
                CreatingUserError::UnmappedError => panic!("Test went wrong"),
            },
        }
    }
}
