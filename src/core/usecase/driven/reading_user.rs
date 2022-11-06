use crate::core::dto::user::User;
use uuid::Uuid;

#[mockall::automock]
#[async_trait::async_trait]
pub trait ReadingUser {
    async fn list_by_id(&self, external_id: &Uuid) -> Result<User, ReadingUserError>;
}

#[derive(Debug, Clone, Copy)]
pub enum ReadingUserError {
    UserNotFoundError,
    UnmappedError,
    InvalidUuidError,
}
