use crate::core::usecase::driven::gateway::GatewayAPI;

#[mockall::automock]
#[async_trait::async_trait]
pub trait ReadingGateway {
    async fn get_priority_list(&self) -> Result<Box<dyn GatewayAPI + Send>, ReadingGatewayError>;
}

#[derive(Debug, Clone)]
pub enum ReadingGatewayError {
    UnmappedError,
    NoGatewayFound,
}
