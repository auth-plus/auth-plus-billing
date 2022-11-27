use crate::core::dto::payment_method::{Method, PaymentMethod, PaymentMethodInfo};
use crate::core::usecase::driven::creating_payment_method::{
    CreatingPaymentMethod, CreatingPaymentMethodError,
};
use crate::core::usecase::driven::reading_payment_method::{
    ReadingPaymentMethod, ReadingPaymentMethodError,
};
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
        "SELECT * FROM  payment_method WHERE is_default = True AND user_id :: text = $1",
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
            tracing::error!("PaymentMethodRepository.get_default_by_user_id :{:?}", err);
            return Err(ReadingPaymentMethodError::UnmappedError);
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
    let q_invoice = format!(
        "INSERT INTO payment_method (id, user_id, is_default, method, info) VALUES ('{}','{}', '{}','{}','{:?}');",
        payment_method_id, user_id, is_default, method, info
    );
    let r_invoice = sqlx::query(&q_invoice).execute(conn).await;
    match r_invoice {
        Ok(_) => {
            let pm = PaymentMethod {
                id: payment_method_id,
                user_id: user_id,
                is_default: is_default,
                method: method,
                info: info.clone(),
            };
            Ok(pm)
        }
        Err(error) => {
            tracing::error!("{:?}", error);
            return Err(CreatingPaymentMethodError::UnmappedError);
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

    use super::get_default_by_user_id;
    use crate::{
        config::database::get_connection,
        core::dto::payment_method::{Method, PaymentMethodInfo, PixInfo},
    };
    use fake::{uuid::UUIDv4, Fake};
    use uuid::Uuid;

    #[actix_rt::test]
    async fn should_get_default_payment_method() {
        let conn = get_connection().await;
        let user_id: Uuid = UUIDv4.fake();
        let pix_info = PixInfo {
            key: String::from("any@email.com"),
            external_id: String::from("ABCDEFG"),
        };
        let info = PaymentMethodInfo::PixInfo(pix_info);
        let payment_method_id: Uuid = UUIDv4.fake();
        let external_id: Uuid = UUIDv4.fake();
        let method = Method::Pix;
        let q_user = format!(
            "INSERT INTO \"user\" (id, external_id) VALUES ('{}', '{}');",
            user_id.to_string(),
            external_id.to_string()
        );
        sqlx::query(&q_user)
            .execute(&conn)
            .await
            .expect("should_get_default_payment_method: user setup went wrong");
        let q_payment_method = format!(
                "INSERT INTO payment_method (id, user_id, is_default, method, info) VALUES ('{}','{}', '{}','{}','{}');",
                payment_method_id,
                user_id,
                true,
                method,
                serde_json::to_string(&info).unwrap()
            );
        print!("{:?}", q_payment_method);
        sqlx::query(&q_payment_method)
            .execute(&conn)
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
    }
}
