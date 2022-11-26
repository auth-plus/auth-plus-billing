use crate::core::dto::user::User;
use uuid::Uuid;

#[mockall::automock]
#[async_trait::async_trait]
pub trait CreatingUser {
    async fn create(&self, external_user_id: &Uuid) -> Result<User, CreatingUserError>;
}

#[derive(Debug, Clone, Copy)]
pub enum CreatingUserError {
    UnmappedError,
}
