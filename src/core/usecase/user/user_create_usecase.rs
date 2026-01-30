use uuid::Uuid;

use crate::core::{
    dto::user::User,
    usecase::driven::{
        creating_gateway_integration::{
            CreatingGatewayIntegration, CreatingGatewayIntegrationError,
        },
        creating_user::{CreatingUser, CreatingUserError},
        reading_gateway::ReadingGateway,
    },
};

pub struct UserCreateUsecase {
    pub creating_user: Box<dyn CreatingUser>,
    pub reading_gateway: Box<dyn ReadingGateway>,
    pub creating_gateway_integration: Box<dyn CreatingGatewayIntegration>,
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
        let gateway = self
            .reading_gateway
            .get_priority_list()
            .await
            .expect("CreatingUserError::UnmappedError Something wrong happen");
        let gateway_user = gateway
            .create_customer(name, email)
            .await
            .expect("Problem when creating user on gateway");

        let result_user: Result<User, CreatingUserError> =
            self.creating_user.create(&external_user_id).await;
        let internal_user = match result_user {
            Ok(user) => user,
            Err(error) => match error {
                CreatingUserError::UnmappedError => {
                    return Err(String::from(
                        "CreatingUserError::UnmappedError Something wrong happen",
                    ));
                }
            },
        };
        let gateway_id = gateway.get_id();
        match self
            .creating_gateway_integration
            .create(gateway_id, internal_user.id, &gateway_user.id)
            .await
        {
            Ok(_) => {}
            Err(error) => match error {
                CreatingGatewayIntegrationError::UnmappedError => {
                    return Err(String::from(
                        "CreatingGatewayIntegrationError::UnmappedError Something wrong happen",
                    ));
                }
                CreatingGatewayIntegrationError::DuplicateGatewayIntegration => {
                    return Err(String::from(
                        "CreatingGatewayIntegrationError::DuplicateGatewayIntegration Something wrong happen",
                    ));
                }
                CreatingGatewayIntegrationError::NoGatewayIntegration => {
                    return Err(String::from(
                        "CreatingGatewayIntegrationError::NoGatewayIntegration Something wrong happen",
                    ));
                }
            },
        };
        Ok(internal_user)
    }
}

#[cfg(test)]
mod test {

    use super::UserCreateUsecase;
    use crate::core::{
        dto::{gateway_integration::GatewayIntegration, user::User},
        usecase::driven::{
            creating_gateway_integration::MockCreatingGatewayIntegration,
            creating_user::{CreatingUserError, MockCreatingUser},
            gateway::{GatewayAPI, GatewayUser, MockGatewayAPI},
            reading_gateway::MockReadingGateway,
        },
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
        let gateway_id: Uuid = UUIDv4.fake();
        let external_id: Uuid = UUIDv4.fake();
        let name: String = Name().fake();
        let email: String = FreeEmail().fake();
        let user = User {
            id: user_id,
            external_id,
        };
        let gateway_user = GatewayUser {
            id: UUIDv4.fake(),
            name: name.clone(),
            email: email.clone(),
        };
        let gateway_integration = GatewayIntegration {
            id: UUIDv4.fake(),
            gateway_id,
            payment_method_id: None,
            user_id,
            gateway_user_id: UUIDv4.fake(),
            gateway_payment_method_id: None,
        };
        let mut mock_cu = MockCreatingUser::new();
        let mut mock_rg = MockReadingGateway::new();
        let mut mock_cgi = MockCreatingGatewayIntegration::new();
        let mut mock_g = MockGatewayAPI::new();
        mock_g
            .expect_create_customer()
            .with(predicate::eq(name.clone()), predicate::eq(email.clone()))
            .times(1)
            .return_const(Ok(gateway_user.clone()));
        mock_g
            .expect_get_id()
            .with()
            .times(1)
            .return_const(gateway_id.clone());
        mock_cu
            .expect_create()
            .with(predicate::eq(external_id))
            .times(1)
            .return_const(Ok(user.clone()));
        mock_rg
            .expect_get_priority_list()
            .with()
            .times(1)
            .return_once(move || Ok(Box::new(mock_g) as Box<dyn GatewayAPI + Send>));
        mock_cgi
            .expect_create()
            .with(
                predicate::eq(gateway_id.clone()),
                predicate::eq(user_id.clone()),
                predicate::eq(gateway_user.id.clone()),
            )
            .times(1)
            .return_const(Ok(gateway_integration.clone()));
        let user_usecase = UserCreateUsecase {
            creating_user: Box::new(mock_cu),
            reading_gateway: Box::new(mock_rg),
            creating_gateway_integration: Box::new(mock_cgi),
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
        let mut mock_g = MockGatewayAPI::new();
        mock_g
            .expect_create_customer()
            .with(predicate::eq(name.clone()), predicate::eq(email.clone()))
            .times(0);
        let mut mock_cu = MockCreatingUser::new();
        mock_cu.expect_create().times(0);
        let mut mock_rg = MockReadingGateway::new();
        mock_rg.expect_get_priority_list().with().times(0);
        let mut mock_cgi = MockCreatingGatewayIntegration::new();
        mock_cgi.expect_create().times(0);
        let user_usecase = UserCreateUsecase {
            creating_user: Box::new(mock_cu),
            reading_gateway: Box::new(mock_rg),
            creating_gateway_integration: Box::new(mock_cgi),
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
        let gateway_user = GatewayUser {
            id: UUIDv4.fake(),
            name: name.clone(),
            email: email.clone(),
        };
        let mut mock_g = MockGatewayAPI::new();
        mock_g
            .expect_create_customer()
            .with(predicate::eq(name.clone()), predicate::eq(email.clone()))
            .times(1)
            .return_const(Ok(gateway_user.clone()));
        mock_g.expect_get_id().with().times(0);
        let mut mock_cu = MockCreatingUser::new();
        mock_cu
            .expect_create()
            .with(predicate::eq(external_id))
            .times(1)
            .return_const(Err(CreatingUserError::UnmappedError));
        let mut mock_rg = MockReadingGateway::new();
        mock_rg
            .expect_get_priority_list()
            .with()
            .times(1)
            .return_once(move || Ok(Box::new(mock_g) as Box<dyn GatewayAPI + Send>));
        let mut mock_cgi = MockCreatingGatewayIntegration::new();
        mock_cgi.expect_create().times(0);
        let user_usecase = UserCreateUsecase {
            creating_user: Box::new(mock_cu),
            reading_gateway: Box::new(mock_rg),
            creating_gateway_integration: Box::new(mock_cgi),
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
