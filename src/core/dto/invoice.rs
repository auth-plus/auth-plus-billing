use serde::Serialize;

#[derive(Serialize)]
pub struct Invoice {
    pub id: String,
    pub user_id: String,
    pub status: String,
}
