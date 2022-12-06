use crate::core::dto::gateway::Gateway;
use crate::core::usecase::driven::reading_gateway::{ReadingGateway, ReadingGatewayError};
pub use sqlx::postgres::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct GatewayRepository {
    conn: PgPool,
}

#[derive(sqlx::FromRow)]
struct GatewayDAO {
    id: Uuid,
    name: String,
}

async fn get_priority_list(conn: &PgPool) -> Result<Vec<Gateway>, ReadingGatewayError> {
    let result = sqlx::query_as::<_, GatewayDAO>("SELECT * FROM gateway")
        .fetch_all(conn)
        .await;
    match result {
        Ok(list) => {
            let mapped_list = list
                .into_iter()
                .map(|x| Gateway {
                    id: x.id,
                    name: x.name,
                })
                .collect();
            Ok(mapped_list)
        }
        Err(err) => {
            tracing::error!("GatewayRepository.get_priority_list :{:?}", err);
            Err(ReadingGatewayError::UnmappedError)
        }
    }
}

#[async_trait::async_trait]
impl ReadingGateway for GatewayRepository {
    async fn get_priority_list(&self) -> Result<Vec<Gateway>, ReadingGatewayError> {
        get_priority_list(&self.conn).await
    }
}

impl GatewayRepository {
    pub fn new(conn: PgPool) -> Self {
        GatewayRepository { conn }
    }
}

#[cfg(test)]
mod test {

    use super::get_priority_list;
    use crate::{
        config::database::get_connection,
        core::repository::orm::{create_gateway, delete_gateway},
    };
    use fake::{faker::lorem::en::Word, uuid::UUIDv4, Fake};
    use uuid::Uuid;

    #[actix_rt::test]
    async fn should_get_priority_list() {
        let conn = get_connection().await;
        let gateway_id: Uuid = UUIDv4.fake();
        let gateway_name: String = Word().fake();
        create_gateway(&conn, gateway_id, &gateway_name)
            .await
            .expect("get_priority_list: gateway setup went wrong");

        let result = get_priority_list(&conn).await;

        match result {
            Ok(list) => {
                assert_eq!(list[0].id.to_string(), gateway_id.to_string());
                assert_eq!(list[0].name, gateway_name);
            }
            Err(error) => panic!("should_get_priority_list test went wrong: {:?}", error),
        };

        delete_gateway(&conn, gateway_id)
            .await
            .expect("should_get_priority_list: gateway remove went wrong");
    }
}
