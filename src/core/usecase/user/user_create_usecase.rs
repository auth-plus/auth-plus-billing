use uuid::Uuid;

use crate::core::{
    dto::user::User,
    usecase::driven::{
        creating_user::{CreatingUser, CreatingUserError},
        gateway::GatewayIntegration,
    },
};

pub struct UserCreateUsecase {
    pub creating_user: Box<dyn CreatingUser>,
    pub gateway: Box<dyn GatewayIntegration>,
}

impl UserCreateUsecase {
    pub async fn create_user(
        &self,
        external_user_id_str: &str,
        name: &str,
        email: &str,
    ) -> Result<User, String> {
        let external_user_id = match Uuid::parse_str(external_user_id_str) {
            Ok(id) => id,
            Err(_) => return Err(String::from("external id provided isn't uuid")),
        };
        self.gateway
            .create_customer(name, email)
            .await
            .expect("CreatingUserError::GatewayError Something wrong happen");
        let result_user: Result<User, CreatingUserError> =
            self.creating_user.create(&external_user_id).await;
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
    use crate::core::{
        dto::user::User,
        usecase::driven::creating_user::{CreatingUserError, MockCreatingUser},
        usecase::driven::gateway::MockGatewayIntegration,
    };
    use fake::{
        Fake,
        faker::{internet::en::FreeEmail, name::en::Name},
        uuid::UUIDv4,
    };
    use mockall::predicate;
    use uuid::Uuid;

    #[actix_rt::test]
    async fn should_succeed_creating_user() {
        let user_id: Uuid = UUIDv4.fake();
        let external_id: Uuid = UUIDv4.fake();
        let name: String = Name().fake();
        let email: String = FreeEmail().fake();
        let user = User {
            id: user_id,
            external_id,
        };
        let mut mock_cu = MockCreatingUser::new();
        let mut mock_gw = MockGatewayIntegration::new();
        mock_cu
            .expect_create()
            .with(predicate::eq(external_id))
            .times(1)
            .return_const(Ok(user.clone()));
        mock_gw
            .expect_create_customer()
            .with(predicate::eq(name.clone()), predicate::eq(email.clone()))
            .times(1)
            .return_const(Ok(true));
        let user_usecase = UserCreateUsecase {
            creating_user: Box::new(mock_cu),
            gateway: Box::new(mock_gw),
        };
        let result = user_usecase
            .create_user(&external_id.to_string(), &name, &email)
            .await;

        match result {
            Ok(resp) => {
                assert_eq!(user_id, resp.id);
                assert_eq!(external_id, resp.external_id);
            }
            Err(error) => panic!("should_succeed_creating_user test went wrong: {}", error),
        }
    }

    #[actix_rt::test]
    async fn should_fail_when_uuid_is_wrong() {
        let name: String = Name().fake();
        let email: String = FreeEmail().fake();
        let mut mock_cu = MockCreatingUser::new();
        mock_cu.expect_create().times(0);
        let mut mock_gw = MockGatewayIntegration::new();
        mock_gw.expect_create_customer().times(0);
        let user_usecase = UserCreateUsecase {
            creating_user: Box::new(mock_cu),
            gateway: Box::new(mock_gw),
        };
        let result: Result<User, String> = user_usecase
            .create_user("any-hash-that-is-not-uuid", &name, &email)
            .await;

        match result {
            Ok(_) => panic!("should_fail_when_uuid_is_wrong test went wrong"),
            Err(error) => {
                assert_eq!(error, String::from("external id provided isn't uuid"));
            }
        }
    }

    #[actix_rt::test]
    async fn should_fail_when_user_provider_went_wrong() {
        let external_id: Uuid = UUIDv4.fake();
        let name: String = Name().fake();
        let email: String = FreeEmail().fake();
        let mut mock_cu = MockCreatingUser::new();
        mock_cu
            .expect_create()
            .with(predicate::eq(external_id))
            .times(1)
            .return_const(Err(CreatingUserError::UnmappedError));
        let mut mock_gw = MockGatewayIntegration::new();
        mock_gw
            .expect_create_customer()
            .with(predicate::eq(name.clone()), predicate::eq(email.clone()))
            .times(1)
            .return_const(Ok(true));
        let user_usecase = UserCreateUsecase {
            creating_user: Box::new(mock_cu),
            gateway: Box::new(mock_gw),
        };
        let result = user_usecase
            .create_user(&external_id.to_string(), &name, &email)
            .await;

        match result {
            Ok(_) => panic!("should_fail_when_user_provider_went_wrong test went wrong"),
            Err(error) => {
                assert_eq!(
                    error,
                    String::from("CreatingUserError::UnmappedError Something wrong happen")
                );
            }
        }
    }
}
