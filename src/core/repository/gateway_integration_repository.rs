use crate::core::dto::gateway_integration::GatewayIntegration;
use crate::core::usecase::driven::creating_gateway_integration::{
    CreatingGatewayIntegration, CreatingGatewayIntegrationError,
};
use crate::core::usecase::driven::updating_gateway_integration::{
    UpdatingGatewayIntegration, UpdatingGatewayIntegrationError,
};
use log::error;
pub use sqlx::postgres::PgPool;
use uuid::Uuid;
#[derive(Clone)]
pub struct GatewayIntegrationRepository {
    conn: PgPool,
}

#[derive(sqlx::FromRow)]
struct GatewayIntegrationDAO {
    id: Uuid,
    // gateway_id: Uuid,
    // payment_method_id: Option<Uuid>,
    // user_id: Uuid,
    gateway_external_user_id: String,
    // gateway_external_payment_method_id: Option<String>,
}

async fn create(
    conn: &PgPool,
    gateway_id: Uuid,
    user_id: Uuid,
    gateway_user_id: &str,
) -> Result<GatewayIntegration, CreatingGatewayIntegrationError> {
    let result = sqlx::query_as::<_, GatewayIntegrationDAO>("SELECT * FROM gateway_integration WHERE user_id=$1 AND gateway_id=$2").bind(user_id).bind(gateway_id)
        .fetch_all(conn)
        .await
        .expect("GatewayIntegrationRepository::create Something wrong happened when fetching all gateway integration for this use");
    if !result.is_empty() {
        return Err(CreatingGatewayIntegrationError::DuplicateGatewayIntegration);
    }
    let gateway_integration_id = Uuid::new_v4();
    let result = sqlx::query("INSERT INTO gateway_integration (id, gateway_id, user_id, gateway_external_user_id) VALUES ($1, $2, $3, $4);")
        .bind(gateway_integration_id)
        .bind(gateway_id)
        .bind(user_id)
        .bind(gateway_user_id)
        .execute(conn)
        .await;
    match result {
        Ok(_) => {
            let item = GatewayIntegration {
                id: gateway_integration_id,
                payment_method_id: None,
                gateway_id,
                user_id,
                gateway_user_id: gateway_user_id.into(),
                gateway_payment_method_id: None,
            };
            Ok(item)
        }
        Err(err) => {
            error!("GatewayIntegration.create :{:?}", err);
            Err(CreatingGatewayIntegrationError::UnmappedError)
        }
    }
}
async fn update(
    conn: &PgPool,
    gateway_id: Uuid,
    payment_method_id: Uuid,
    user_id: Uuid,
    gateway_payment_method_id: &str,
) -> Result<GatewayIntegration, UpdatingGatewayIntegrationError> {
    let gateway_integration_list: Vec<GatewayIntegrationDAO> =
        sqlx::query_as::<_, GatewayIntegrationDAO>("SELECT * FROM gateway_integration WHERE user_id::text=$1 AND gateway_id::text=$2;").bind(user_id.to_string()).bind(gateway_id.to_string())
            .fetch_all(conn)
            .await
            .expect("GatewayIntegrationRepository::update Something wrong happened when fetching all gateway integration for this use");
    if gateway_integration_list.is_empty() {
        return Err(UpdatingGatewayIntegrationError::NoGatewayIntegration);
    }
    if gateway_integration_list.len() > 1 {
        return Err(UpdatingGatewayIntegrationError::DuplicateGatewayIntegration);
    }
    let gateway_integration_id: Uuid = gateway_integration_list[0].id;
    let result = sqlx::query("UPDATE gateway_integration SET payment_method_id=$1, gateway_external_payment_method_id=$2 WHERE id::text=$3;")
        .bind(payment_method_id)
        .bind(gateway_payment_method_id)
        .bind(gateway_integration_id.to_string())
        .execute(conn)
        .await;
    match result {
        Ok(_) => {
            let item = GatewayIntegration {
                id: gateway_integration_id,
                payment_method_id: Some(payment_method_id),
                gateway_id,
                user_id,
                gateway_user_id: gateway_integration_list[0]
                    .gateway_external_user_id
                    .to_string(),
                gateway_payment_method_id: Some(gateway_payment_method_id.to_string()),
            };
            Ok(item)
        }
        Err(err) => {
            error!("GatewayIntegration.create :{:?}", err);
            Err(UpdatingGatewayIntegrationError::UnmappedError)
        }
    }
}

#[async_trait::async_trait]
impl CreatingGatewayIntegration for GatewayIntegrationRepository {
    async fn create(
        &self,
        gateway_id: Uuid,
        user_id: Uuid,
        gateway_user_id: &str,
    ) -> Result<GatewayIntegration, CreatingGatewayIntegrationError> {
        create(&self.conn, gateway_id, user_id, gateway_user_id).await
    }
}

#[async_trait::async_trait]
impl UpdatingGatewayIntegration for GatewayIntegrationRepository {
    async fn update(
        &self,
        gateway_id: Uuid,
        user_id: Uuid,
        payment_method_id: Uuid,
        gateway_payment_method_id: &str,
    ) -> Result<GatewayIntegration, UpdatingGatewayIntegrationError> {
        update(
            &self.conn,
            gateway_id,
            payment_method_id,
            user_id,
            gateway_payment_method_id,
        )
        .await
    }
}

impl GatewayIntegrationRepository {
    pub fn new(conn: PgPool) -> Self {
        GatewayIntegrationRepository { conn }
    }
}

#[cfg(test)]
mod test {

    use super::create;
    use crate::{
        config::database::get_connection,
        core::{
            dto::payment_method::{Method, PaymentMethodInfo, PixInfo},
            repository::orm::{
                create_gateway, create_payment_method, create_user, delete_gateway,
                delete_gateway_integration, delete_payment_method, delete_user,
            },
        },
    };
    use fake::{Fake, faker::lorem::en::Word, uuid::UUIDv4};
    use uuid::Uuid;

    #[actix_rt::test]
    async fn should_create_payment_integration() {
        let conn = get_connection().await;
        let user_id: Uuid = UUIDv4.fake();
        let external_id: Uuid = UUIDv4.fake();
        let gateway_id: Uuid = UUIDv4.fake();
        let gateway_name: String = Word().fake();
        let pix_info = PixInfo {
            key: String::from("any@email.com"),
            external_id: String::from("ABCDEFG"),
        };
        let info = PaymentMethodInfo::PixInfo(pix_info);
        let external_user_id: Uuid = UUIDv4.fake();

        let method = Method::Pix;
        create_user(&conn, user_id, external_id)
            .await
            .expect("should_create_payment_integration: user setup went wrong");
        create_gateway(&conn, gateway_id, &gateway_name, 1)
            .await
            .expect("should_create_payment_integration: gateway setup went wrong");
        create_payment_method(&conn, external_user_id, user_id, true, method, info)
            .await
            .expect("should_create_payment_integration: payment_method setup went wrong");
        let result = create(
            &conn,
            gateway_id,
            user_id,
            &external_user_id.clone().to_string(),
        )
        .await;

        match result {
            Ok(gi) => {
                assert!(!gi.id.to_string().is_empty());
                assert_eq!(gi.gateway_user_id, external_user_id.to_string());
                assert_eq!(gi.user_id, user_id);
                assert_eq!(gi.gateway_id, gateway_id);
                delete_gateway_integration(&conn, gi.id).await.expect(
                    "should_create_payment_integration: gateway_integration remove went wrong",
                );
            }
            Err(error) => panic!(
                "should_create_payment_integration test went wrong: {:?}",
                error
            ),
        };

        delete_payment_method(&conn, external_user_id)
            .await
            .expect("should_create_payment_integration: payment_method remove went wrong");
        delete_gateway(&conn, gateway_id)
            .await
            .expect("should_create_payment_integration: gateway remove went wrong");
        delete_user(&conn, user_id)
            .await
            .expect("should_create_payment_integration: user remove went wrong");
    }
}
