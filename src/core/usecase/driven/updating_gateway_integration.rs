use crate::core::dto::gateway_integration::GatewayIntegration;
use uuid::Uuid;

#[mockall::automock]
#[async_trait::async_trait]
pub trait UpdatingGatewayIntegration {
    async fn update(
        &self,
        gateway_id: Uuid,
        user_id: Uuid,
        payment_method_id: Uuid,
        gateway_payment_method_id: &str,
    ) -> Result<GatewayIntegration, UpdatingGatewayIntegrationError>;
}

#[derive(Debug, Clone, Copy)]
pub enum UpdatingGatewayIntegrationError {
    UnmappedError,
    DuplicateGatewayIntegration,
    NoGatewayIntegration,
}
