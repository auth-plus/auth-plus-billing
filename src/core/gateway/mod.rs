use uuid::Uuid;

use crate::core::gateway::stripe::StripeGateway;

pub mod stripe;
pub mod stripe_models;

#[derive(sqlx::FromRow)]
pub struct GatewayDAO {
    pub id: Uuid,
    pub name: String,
    pub priority: i32,
}

#[derive(Clone)]
pub struct GatewayMap {
    pub stripe: StripeGateway,
}
