use crate::core::dto::gateway_integration::GatewayIntegration;
use uuid::Uuid;

#[mockall::automock]
#[async_trait::async_trait]
pub trait CreatingGatewayIntegration {
    async fn create(
        &self,
        gateway_id: Uuid,
        payment_method_id: Uuid,
    ) -> Result<GatewayIntegration, CreatingGatewayIntegrationError>;
}

#[derive(Debug, Clone, Copy)]
pub enum CreatingGatewayIntegrationError {
    UnmappedError,
}
