use crate::core::dto::payment_method::{Method, PaymentMethod, PaymentMethodInfo};
use uuid::Uuid;

#[mockall::automock]
#[async_trait::async_trait]
pub trait CreatingPaymentMethod {
    async fn create(
        &self,
        user_id: Uuid,
        gateway_id: Uuid,
        is_default: bool,
        method: Method,
        info: &PaymentMethodInfo,
    ) -> Result<PaymentMethod, CreatingPaymentMethodError>;
}

#[derive(Debug, Clone, Copy)]
pub enum CreatingPaymentMethodError {
    UnmappedError,
}
