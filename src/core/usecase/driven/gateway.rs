use serde::{Deserialize, Serialize};

#[mockall::automock]
#[async_trait::async_trait]
pub trait GatewayAPI {
    fn set_id(&mut self, id: uuid::Uuid) -> Result<bool, GatewayAPIError>;
    fn get_id(&self) -> uuid::Uuid;
    async fn charge(
        &self,
        amount: rust_decimal::Decimal,
        description: &str,
    ) -> Result<GatewayCharge, GatewayAPIError>;
    async fn create_customer(
        &self,
        name: &str,
        email: &str,
    ) -> Result<GatewayUser, GatewayAPIError>;
    async fn create_payment_method(
        &self,
        r#type: &str,
    ) -> Result<GatewayPaymentMethod, GatewayAPIError>;
}

#[derive(Debug, Clone)]
pub enum GatewayAPIError {
    LoginError,
    ChargeError,
    CustomerCreationError,
    PaymentMethodCreationError,
    PaymentMethodTransformError,
    NotSuccessfulReturn,
    UnmappedError,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GatewayUser {
    pub id: String,
    pub name: String,
    pub email: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GatewayPaymentMethod {
    pub id: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GatewayCharge {
    pub id: String,
    pub amount: f32,
    pub currency: String,
}
