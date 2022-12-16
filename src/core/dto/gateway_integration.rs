use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize, Clone, Debug)]
pub struct GatewayIntegration {
    pub id: Uuid,
    pub payment_method_id: Uuid,
    pub gateway_id: Uuid,
    pub gateway_external_id: Option<String>,
}
