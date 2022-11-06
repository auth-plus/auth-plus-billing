use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize, Clone, Debug)]
pub struct Invoice {
    pub id: Uuid,
    pub user_id: Uuid,
    pub status: String,
}
