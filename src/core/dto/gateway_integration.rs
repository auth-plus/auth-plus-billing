use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize, Clone, Debug)]
pub struct GatewayIntegration {
    pub id: Uuid,
    pub gateway_id: Uuid,
    pub payment_method_id: Option<Uuid>,
    pub user_id: Uuid,
    pub gateway_user_id: String,
    pub gateway_payment_method_id: Option<String>,
}
