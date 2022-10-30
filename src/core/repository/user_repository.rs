use crate::core::dto::user::User;
use crate::core::usecase::driven::reading_user::{ReadingUser, ReadingUserError};

pub struct UserRepository {}

impl ReadingUser for UserRepository {
    fn list_by_id(&self, _id: &str) -> Result<User, ReadingUserError> {
        let usr = User {
            id: String::from("asdasd"),
        };
        Ok(usr)
    }
}
