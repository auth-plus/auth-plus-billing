use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Clone, Debug, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub external_id: Uuid,
}
