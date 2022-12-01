use std::time::Duration;

use crate::config::kafka::get_producer;
use crate::core::dto::charge::{Charge, ChargeStatus};
use crate::core::usecase::driven::creating_charge::{CreatingCharge, CreatingChargeError};
use rdkafka::producer::FutureRecord;
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
) -> Result<Charge, CreatingChargeError> {
    let producer = get_producer();
    let charge_id = Uuid::new_v4();
    let status = ChargeStatus::Progress;
    let q_charge = format!(
        "INSERT INTO charge (id, invoice_id, payment_method_id, status) VALUES ('{}', '{}', '{}', '{}');",
        charge_id,
        invoice_id,
        payment_method_id,
        status
    );
    let result_charge = sqlx::query(&q_charge).execute(conn).await;
    let charge = match result_charge {
        Ok(_) => Charge {
            id: charge_id,
            invoice_id,
            payment_method_id,
            status,
        },
        Err(_) => return Err(CreatingChargeError::UnmappedError),
    };
    let message = format!("{:?}", charge);
    let send = producer
        .send(
            FutureRecord::to("INVOICE_CHARGE")
                .payload(&message)
                .key(&format!("Key {}", 0)),
            Duration::from_secs(0),
        )
        .await;
    match send {
        Ok(_) => Ok(charge),
        Err(_) => Err(CreatingChargeError::KafkaProducerError),
    }
}

#[async_trait::async_trait]
impl CreatingCharge for ChargeRepository {
    async fn create_charge(
        &self,
        invoice_id: Uuid,
        payment_method_id: Uuid,
    ) -> Result<Charge, CreatingChargeError> {
        create(&self.conn, invoice_id, payment_method_id).await
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
    use crate::{config::database::get_connection, core::dto::charge::ChargeStatus};
    use fake::{uuid::UUIDv4, Fake};
    use uuid::Uuid;

    #[actix_rt::test]
    async fn should_create_user() {
        let conn = get_connection().await;
        let invoice_id: Uuid = UUIDv4.fake();
        let payment_method_id: Uuid = UUIDv4.fake();

        let result = create(&conn, invoice_id, payment_method_id).await;

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
            }
            Err(error) => panic!("should_create_user test went wrong: {:?}", error),
        }
    }
}
