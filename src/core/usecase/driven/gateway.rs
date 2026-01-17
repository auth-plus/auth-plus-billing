#[mockall::automock]
#[async_trait::async_trait]
pub trait GatewayIntegration {
    async fn charge(
        &self,
        amount: f32,
        description: String,
    ) -> Result<bool, GatewayIntegrationError>;
    async fn create_customer(
        &self,
        name: String,
        email: String,
    ) -> Result<bool, GatewayIntegrationError>;
    async fn create_payment_method(&self, r#type: String) -> Result<bool, GatewayIntegrationError>;
}

#[derive(Debug, Clone)]
pub enum GatewayIntegrationError {
    LoginError,
    ChargeError,
    CustomerCreationError,
    PaymentMethodTransformError,
    UnmappedError,
}
