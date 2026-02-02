use crate::core::dto::payment_method::{Method, PaymentMethod, PaymentMethodInfo};
use crate::core::usecase::driven::creating_payment_method::{
    CreatingPaymentMethod, CreatingPaymentMethodError,
};
use crate::core::usecase::driven::reading_payment_method::{
    ReadingPaymentMethod, ReadingPaymentMethodError,
};
use log::error;
pub use sqlx::postgres::PgPool;
use sqlx::types::Json;
use uuid::Uuid;

#[derive(sqlx::FromRow)]
struct PaymentMethodDAO {
    id: Uuid,
    user_id: Uuid,
    is_default: bool,
    method: String,
    info: sqlx::types::Json<PaymentMethodInfo>,
}

#[derive(Clone)]
pub struct PaymentMethodRepository {
    conn: PgPool,
}

async fn get_default_by_user_id(
    conn: &PgPool,
    user_id: &Uuid,
) -> Result<PaymentMethod, ReadingPaymentMethodError> {
    let result = sqlx::query_as::<_, PaymentMethodDAO>(
        "SELECT * FROM  payment_method WHERE is_default = True AND user_id::text = $1",
    )
    .bind(user_id.to_string())
    .fetch_one(conn)
    .await;

    match result {
        Ok(pm) => {
            let Json(info) = pm.info;
            let item = PaymentMethod {
                id: pm.id,
                user_id: pm.user_id,
                is_default: pm.is_default,
                method: Method::from(pm.method.as_str()),
                info,
            };
            Ok(item)
        }
        Err(err) => {
            println!("{}", err);
            error!("PaymentMethodRepository.get_default_by_user_id :{:?}", err);
            Err(ReadingPaymentMethodError::UnmappedError)
        }
    }
}

async fn create(
    conn: &PgPool,
    user_id: Uuid,
    is_default: bool,
    method: Method,
    info: &PaymentMethodInfo,
) -> Result<PaymentMethod, CreatingPaymentMethodError> {
    let payment_method_id = Uuid::new_v4();
    let inser_info = Json(info);
    let result = sqlx::query("INSERT INTO payment_method (id, user_id, is_default, method, info) VALUES ($1,$2,$3,$4,$5)").bind(payment_method_id).bind(user_id).bind(is_default).bind(method.to_string()).bind(inser_info).execute(conn).await;
    match result {
        Ok(_) => {
            let pm = PaymentMethod {
                id: payment_method_id,
                user_id,
                is_default,
                method,
                info: info.clone(),
            };
            Ok(pm)
        }
        Err(error) => {
            error!("PaymentMethodRepository.create :{:?}", error);
            Err(CreatingPaymentMethodError::UnmappedError)
        }
    }
}

#[async_trait::async_trait]
impl ReadingPaymentMethod for PaymentMethodRepository {
    async fn get_default_by_user_id(
        &self,
        user_id: &Uuid,
    ) -> Result<PaymentMethod, ReadingPaymentMethodError> {
        get_default_by_user_id(&self.conn, user_id).await
    }
}

#[async_trait::async_trait]
impl CreatingPaymentMethod for PaymentMethodRepository {
    async fn create(
        &self,
        user_id: Uuid,
        is_default: bool,
        method: Method,
        info: &PaymentMethodInfo,
    ) -> Result<PaymentMethod, CreatingPaymentMethodError> {
        create(&self.conn, user_id, is_default, method, info).await
    }
}

impl PaymentMethodRepository {
    pub fn new(conn: PgPool) -> Self {
        PaymentMethodRepository { conn }
    }
}

#[cfg(test)]
mod test {

    use super::{create, get_default_by_user_id};
    use crate::{
        config::database::get_connection,
        core::{
            dto::payment_method::{Method, PaymentMethodInfo, PixInfo},
            repository::orm::{
                create_gateway, create_payment_method, create_user, delete_gateway,
                delete_payment_method, delete_user,
            },
        },
    };
    use fake::{Fake, faker::lorem::en::Word, uuid::UUIDv4};
    use uuid::Uuid;

    #[actix_rt::test]
    async fn should_get_default_payment_method() {
        let conn = get_connection().await;
        let user_id: Uuid = UUIDv4.fake();
        let gateway_id: Uuid = UUIDv4.fake();
        let gateway_name: String = Word().fake();
        let pix_info = PixInfo {
            key: String::from("any@email.com"),
            external_id: String::from("ABCDEFG"),
        };
        let info = PaymentMethodInfo::PixInfo(pix_info);
        let payment_method_id: Uuid = UUIDv4.fake();
        let external_id: Uuid = UUIDv4.fake();
        let method = Method::Pix;
        create_user(&conn, user_id, external_id)
            .await
            .expect("should_get_default_payment_method: user setup went wrong");
        create_gateway(&conn, gateway_id, &gateway_name, 1)
            .await
            .expect("should_get_default_payment_method: gateway setup went wrong");
        create_payment_method(&conn, payment_method_id, user_id, true, method, info)
            .await
            .expect("should_get_default_payment_method: payment_method setup went wrong");

        let result = get_default_by_user_id(&conn, &user_id).await;

        match result {
            Ok(pm) => {
                assert_eq!(pm.user_id.to_string(), user_id.to_string());
                assert!(pm.is_default);
                assert!(!pm.id.to_string().is_empty());
                assert_eq!(pm.method.to_string(), method.to_string());
            }
            Err(error) => panic!(
                "should_get_default_payment_method test went wrong: {:?}",
                error
            ),
        };
        delete_payment_method(&conn, payment_method_id)
            .await
            .expect("should_get_default_payment_method: gateway remove went wrong");
        delete_gateway(&conn, gateway_id)
            .await
            .expect("should_get_default_payment_method: gateway remove went wrong");
        delete_user(&conn, user_id)
            .await
            .expect("should_get_default_payment_method: user remove went wrong");
    }

    #[actix_rt::test]
    async fn should_create_payment_method() {
        let conn = get_connection().await;
        let user_id: Uuid = UUIDv4.fake();
        let pix_info = PixInfo {
            key: String::from("any@email.com"),
            external_id: String::from("ABCDEFG"),
        };
        let info = PaymentMethodInfo::PixInfo(pix_info);
        let external_id: Uuid = UUIDv4.fake();
        let method = Method::Pix;
        create_user(&conn, user_id, external_id)
            .await
            .expect("should_create_payment_method: user setup went wrong");

        let result = create(&conn, user_id, true, method, &info).await;

        match result {
            Ok(pm) => {
                assert_eq!(pm.user_id.to_string(), user_id.to_string());
                assert!(pm.is_default);
                assert!(!pm.id.to_string().is_empty());
                assert_eq!(pm.method.to_string(), method.to_string());
                delete_payment_method(&conn, pm.id)
                    .await
                    .expect("should_create_payment_method: gateway remove went wrong");
            }
            Err(error) => panic!("should_create_payment_method test went wrong: {:?}", error),
        };

        delete_user(&conn, user_id)
            .await
            .expect("should_create_payment_method: user remove went wrong");
    }
}
