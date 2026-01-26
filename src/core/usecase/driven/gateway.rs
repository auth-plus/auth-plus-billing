#[mockall::automock]
#[async_trait::async_trait]
pub trait GatewayIntegration {
    async fn charge(&self, amount: f32, description: &str)
    -> Result<bool, GatewayIntegrationError>;
    async fn create_customer(
        &self,
        name: &str,
        email: &str,
    ) -> Result<bool, GatewayIntegrationError>;
    async fn create_payment_method(&self, r#type: &str) -> Result<bool, GatewayIntegrationError>;
}

#[derive(Debug, Clone)]
pub enum GatewayIntegrationError {
    LoginError,
    ChargeError,
    CustomerCreationError,
    PaymentMethodTransformError,
    NotSuccessfulReturn,
    UnmappedError,
}
