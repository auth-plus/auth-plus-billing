use uuid::Uuid;

use crate::core::{
    dto::user::User,
    usecase::driven::creating_user::{CreatingUser, CreatingUserError},
};

pub struct UserCreateUsecase {
    pub creating_user: Box<dyn CreatingUser>,
}

impl UserCreateUsecase {
    pub async fn create_user(&self, external_user_id_str: &str) -> Result<User, String> {
        let external_user_id = match Uuid::parse_str(external_user_id_str) {
            Ok(id) => id,
            Err(_error) => return Err(String::from("external id provided isn't uuid")),
        };
        let result_user = self.creating_user.create(&external_user_id).await;
        match result_user {
            Ok(user) => Ok(user),
            Err(error) => match error {
                CreatingUserError::UnmappedError => Err(String::from(
                    "CreatingUserError::UnmappedError Something wrong happen",
                )),
            },
        }
    }
}

#[cfg(test)]
mod test {

    use super::UserCreateUsecase;
    use crate::core::{dto::user::User, usecase::driven::creating_user::MockCreatingUser};
    use mockall::predicate;
    use uuid::Uuid;

    #[actix_rt::test]
    async fn should_succeed_creating_user() {
        let user_id = Uuid::new_v4();
        let external_id = Uuid::new_v4();
        let user = User {
            id: user_id,
            external_id,
        };
        let mut mock_cu = MockCreatingUser::new();
        mock_cu
            .expect_create()
            .with(predicate::eq(external_id))
            .times(1)
            .return_const(Ok(user.clone()));
        let user_usecase = UserCreateUsecase {
            creating_user: Box::new(mock_cu),
        };
        let result = user_usecase.create_user(&external_id.to_string()).await;

        match result {
            Ok(resp) => {
                assert_eq!(user_id, resp.id);
                assert_eq!(external_id, resp.external_id);
            }
            Err(error) => panic!("Test went wrong: {}", error),
        }
    }
}
