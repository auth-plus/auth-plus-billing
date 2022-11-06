use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize, Clone)]
pub struct User {
    pub id: Uuid,
    pub external_id: Uuid,
}
