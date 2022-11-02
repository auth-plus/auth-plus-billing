use serde::Serialize;
#[derive(Serialize)]

pub struct User {
    pub id: String,
    pub external_id: String,
}
