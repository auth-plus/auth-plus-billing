use crate::core::dto::gateway_integration::GatewayIntegration;
use crate::core::usecase::driven::creating_gateway_integration::{
    CreatingGatewayIntegration, CreatingGatewayIntegrationError,
};
use log::error;
pub use sqlx::postgres::PgPool;
use uuid::Uuid;
#[derive(Clone)]
pub struct GatewayIntegrationRepository {
    conn: PgPool,
}

async fn create(
    conn: &PgPool,
    gateway_id: Uuid,
    payment_method_id: Uuid,
) -> Result<GatewayIntegration, CreatingGatewayIntegrationError> {
    let gateway_integration_id = Uuid::new_v4();
    let q_gi = format!(
        "INSERT INTO gateway_integration (id, gateway_id, payment_method_id) VALUES ('{}', '{}', '{}');",
        gateway_integration_id,
        gateway_id,
        payment_method_id,
    );
    let result = sqlx::query(&q_gi).execute(conn).await;
    match result {
        Ok(_) => {
            let item = GatewayIntegration {
                id: gateway_integration_id,
                payment_method_id,
                gateway_id,
                gateway_external_id: None,
            };
            Ok(item)
        }
        Err(err) => {
            error!("GatewayIntegration.create :{:?}", err);
            Err(CreatingGatewayIntegrationError::UnmappedError)
        }
    }
}

#[async_trait::async_trait]
impl CreatingGatewayIntegration for GatewayIntegrationRepository {
    async fn create(
        &self,
        gateway_id: Uuid,
        payment_method_id: Uuid,
    ) -> Result<GatewayIntegration, CreatingGatewayIntegrationError> {
        create(&self.conn, gateway_id, payment_method_id).await
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
    use fake::{faker::lorem::en::Word, uuid::UUIDv4, Fake};
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
        let payment_method_id: Uuid = UUIDv4.fake();

        let method = Method::Pix;
        create_user(&conn, user_id, external_id)
            .await
            .expect("should_create_payment_integration: user setup went wrong");
        create_gateway(&conn, gateway_id, &gateway_name, 1)
            .await
            .expect("should_create_payment_integration: gateway setup went wrong");
        create_payment_method(&conn, payment_method_id, user_id, true, method, info)
            .await
            .expect("should_create_payment_integration: payment_method setup went wrong");
        let result = create(&conn, gateway_id, payment_method_id).await;

        match result {
            Ok(gi) => {
                assert!(!gi.id.to_string().is_empty());
                assert_eq!(gi.payment_method_id, payment_method_id);
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

        delete_payment_method(&conn, payment_method_id)
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
