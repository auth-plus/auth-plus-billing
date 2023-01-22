use crate::core::dto::charge::{Charge, ChargeStatus};
use crate::core::usecase::driven::creating_charge::{CreatingCharge, CreatingChargeError};
pub use sqlx::postgres::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct ChargeRepository {
    conn: PgPool,
}

async fn create(
    conn: &PgPool,
    invoice_id: Uuid,
    payment_method_id: Uuid,
    external_id: String,
) -> Result<Charge, CreatingChargeError> {
    let charge_id = Uuid::new_v4();
    let status = ChargeStatus::Progress;
    let q_charge = format!(
        "INSERT INTO charge (id, invoice_id, external_id, payment_method_id, status) VALUES ('{}', '{}', '{}', '{}', '{}');",
        charge_id,
        invoice_id,
        external_id,
        payment_method_id,
        status
    );
    let result_charge = sqlx::query(&q_charge).execute(conn).await;
    match result_charge {
        Ok(_) => {
            let item = Charge {
                id: charge_id,
                invoice_id,
                payment_method_id,
                status,
            };
            Ok(item)
        }
        Err(error) => {
            tracing::error!("ChargeRepository.create :{:?}", error);
            Err(CreatingChargeError::UnmappedError)
        }
    }
}

#[async_trait::async_trait]
impl CreatingCharge for ChargeRepository {
    async fn create_charge(
        &self,
        invoice_id: Uuid,
        payment_method_id: Uuid,
    ) -> Result<Charge, CreatingChargeError> {
        let external_id = String::from("GET THIS FROM GATEWAY");
        create(&self.conn, invoice_id, payment_method_id, external_id).await
    }
}

impl ChargeRepository {
    pub fn new(conn: PgPool) -> Self {
        ChargeRepository { conn }
    }
}

#[cfg(test)]
mod test {

    use super::create;
    use crate::{
        config::database::get_connection,
        core::{
            dto::{
                charge::ChargeStatus,
                invoice::InvoiceStatus,
                payment_method::{CreditCardInfo, Method, PaymentMethodInfo},
            },
            repository::orm::{
                create_gateway, create_invoice, create_payment_method, create_user, delete_charge,
                delete_gateway, delete_invoice, delete_payment_method, delete_user,
            },
        },
    };
    use fake::{faker::lorem::en::Word, uuid::UUIDv4, Fake};
    use uuid::Uuid;

    #[actix_rt::test]
    async fn should_create_charge() {
        let conn = get_connection().await;
        let invoice_external_id: String = Word().fake();
        let gateway_id: Uuid = UUIDv4.fake();
        let gateway_name: String = Word().fake();
        let invoice_id: Uuid = UUIDv4.fake();
        let payment_method_id: Uuid = UUIDv4.fake();
        let user_id: Uuid = UUIDv4.fake();
        let external_id: Uuid = UUIDv4.fake();
        let method = Method::Pix;
        let cc_info = CreditCardInfo {
            last4digit: String::from("1234"),
            flag: String::from("visa"),
            expire_data: String::from("01/01/1970"),
            external_id: String::from("ABCDEFG"),
        };
        let info = PaymentMethodInfo::CreditCardInfo(cc_info);
        create_user(&conn, user_id, external_id)
            .await
            .expect("should_create_charge: user setup went wrong");
        create_invoice(&conn, invoice_id, user_id, InvoiceStatus::Draft)
            .await
            .expect("should_create_charge: invoice setup went wrong");
        create_gateway(&conn, gateway_id, &gateway_name, 1)
            .await
            .expect("should_create_charge: gateway setup went wrong");
        create_payment_method(&conn, payment_method_id, user_id, true, method, info)
            .await
            .expect("should_create_charge: payment_method setup went wrong");

        let result = create(&conn, invoice_id, payment_method_id, invoice_external_id).await;

        match result {
            Ok(charge) => {
                assert_eq!(charge.id.to_string().is_empty(), false);
                assert_eq!(charge.invoice_id.to_string(), invoice_id.to_string());
                assert_eq!(
                    charge.payment_method_id.to_string(),
                    payment_method_id.to_string()
                );
                assert_eq!(
                    charge.status.to_string(),
                    ChargeStatus::Progress.to_string()
                );
                delete_charge(&conn, charge.id)
                    .await
                    .expect("should_create_charge: charge remove went wrong");
            }
            Err(error) => panic!("should_create_charge test went wrong: {:?}", error),
        };

        delete_payment_method(&conn, payment_method_id)
            .await
            .expect("should_create_charge: gateway remove went wrong");
        delete_gateway(&conn, gateway_id)
            .await
            .expect("should_create_charge: gateway remove went wrong");
        delete_invoice(&conn, invoice_id)
            .await
            .expect("should_create_charge: invoice remove went wrong");
        delete_user(&conn, user_id)
            .await
            .expect("should_create_charge: user remove went wrong");
    }
}
