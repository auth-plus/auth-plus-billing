use crate::core::gateway::GatewayMap;
use crate::core::usecase::driven::gateway::GatewayAPI;
use crate::core::usecase::driven::reading_gateway::{ReadingGateway, ReadingGatewayError};
use log::error;
pub use sqlx::postgres::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct GatewayRepository {
    conn: PgPool,
    gateway_map: GatewayMap,
}

#[derive(sqlx::FromRow)]
struct GatewayDAO {
    id: Uuid,
    name: String,
    // priority: i32,
}

async fn get_priority_list(
    conn: &PgPool,
    gateway_map: &GatewayMap,
) -> Result<Box<dyn GatewayAPI + Send>, ReadingGatewayError> {
    let result = sqlx::query_as::<_, GatewayDAO>("SELECT * FROM gateway ORDER BY priority ASC")
        .fetch_all(conn)
        .await;
    let list = match result {
        Ok(list) => list,
        Err(err) => {
            error!("GatewayRepository.get_priority_list :{:?}", err);
            return Err(ReadingGatewayError::UnmappedError);
        }
    };
    match list.first() {
        Some(gateway) => match gateway.name.as_str() {
            "stripe" => {
                let mut resp = Box::new(gateway_map.stripe.clone());
                resp.set_id(gateway.id).expect("erro setting gateway id");
                Ok(resp)
            }
            _ => Err(ReadingGatewayError::NoGatewayFound),
        },
        None => Err(ReadingGatewayError::NoGatewayFound),
    }
}

#[async_trait::async_trait]
impl ReadingGateway for GatewayRepository {
    async fn get_priority_list(&self) -> Result<Box<dyn GatewayAPI + Send>, ReadingGatewayError> {
        get_priority_list(&self.conn, &self.gateway_map).await
    }
}

impl GatewayRepository {
    pub fn new(conn: PgPool, gateway_map: GatewayMap) -> Self {
        GatewayRepository { conn, gateway_map }
    }
}

#[cfg(test)]
mod test {

    use super::get_priority_list;
    use crate::{
        config::database::get_connection,
        core::{
            gateway::{GatewayMap, stripe::StripeGateway},
            repository::orm::read_main_gateway,
            usecase::driven::gateway::GatewayAPI,
        },
    };

    #[actix_rt::test]
    async fn should_get_priority_list() {
        let conn = get_connection().await;
        let mut gm = GatewayMap {
            stripe: StripeGateway::new(),
        };
        let gateway = read_main_gateway(&conn)
            .await
            .expect("read_main_gateway: read main gateway went wrong");
        gm.stripe
            .set_id(gateway.id)
            .expect("erro setting gateway id");

        let result = get_priority_list(&conn, &gm).await;

        match result {
            Ok(gate) => {
                assert_eq!(gate.get_id().to_string(), gateway.id.to_string());
            }
            Err(error) => panic!("should_get_priority_list test went wrong: {:?}", error),
        }
    }
}
