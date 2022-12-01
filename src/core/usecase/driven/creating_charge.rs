use uuid::Uuid;

use crate::core::dto::charge::Charge;

#[mockall::automock]
#[async_trait::async_trait]
pub trait CreatingCharge {
    async fn create_charge(
        &self,
        invoice_id: Uuid,
        payment_method_id: Uuid,
    ) -> Result<Charge, CreatingChargeError>;
}

#[derive(Debug, Clone, Copy)]
pub enum CreatingChargeError {
    KafkaProducerError,
    UnmappedError,
}
