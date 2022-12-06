use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize, Clone, Debug)]
pub struct Gateway {
    pub id: Uuid,
    pub name: String,
}
