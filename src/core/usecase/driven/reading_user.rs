use crate::core::dto::user::User;

#[async_trait::async_trait]
pub trait ReadingUser {
    fn list_by_id(&self, id: &str) -> Result<User, ReadingUserError>;
}

#[derive(Debug, Clone, Copy)]
pub enum ReadingUserError {
    UserNotFoundError,
}
