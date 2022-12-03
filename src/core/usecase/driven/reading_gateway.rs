use crate::core::dto::gateway::Gateway;
#[mockall::automock]
#[async_trait::async_trait]
pub trait ReadingGateway {
    async fn get_priority_list(&self) -> Result<Vec<Gateway>, ReadingGatewayError>;
}

#[derive(Debug, Clone, Copy)]
pub enum ReadingGatewayError {
    UnmappedError,
}
