use crate::core::dto::payment_method::PaymentMethod;
use uuid::Uuid;

#[mockall::automock]
#[async_trait::async_trait]
pub trait ReadingPaymentMethod {
    async fn get_default_by_user_id(
        &self,
        user_id: &Uuid,
    ) -> Result<PaymentMethod, ReadingPaymentMethodError>;
}

#[derive(Debug, Clone, Copy)]
pub enum ReadingPaymentMethodError {
    MethodUnexpectederror,
    UnmappedError,
}
