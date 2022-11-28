use uuid::Uuid;

use crate::core::{
    dto::payment_method::{Method, PaymentMethod, PaymentMethodInfo},
    usecase::driven::{
        creating_payment_method::{CreatingPaymentMethod, CreatingPaymentMethodError},
        reading_user::{ReadingUser, ReadingUserError},
    },
};

pub struct PaymentMethodCreateUsecase {
    pub reading_user: Box<dyn ReadingUser>,
    pub creating_payment_method: Box<dyn CreatingPaymentMethod>,
}

impl PaymentMethodCreateUsecase {
    pub async fn create(
        &self,
        external_user_id_str: &str,
        is_default: bool,
        method: Method,
        info: PaymentMethodInfo,
    ) -> Result<PaymentMethod, String> {
        let external_user_id = match Uuid::parse_str(external_user_id_str) {
            Ok(id) => id,
            Err(_error) => return Err(String::from("external id provided isn't uuid")),
        };
        let result_user = self.reading_user.list_by_id(&external_user_id).await;
        let user = match result_user {
            Ok(user) => user,
            Err(error) => match error {
                ReadingUserError::UserNotFoundError => return Err(String::from("User Not found")),
                ReadingUserError::UnmappedError => {
                    return Err(String::from(
                        "ReadingUserError::UnmappedError Something wrong happen",
                    ))
                }
            },
        };
        let result_pm = self
            .creating_payment_method
            .create(user.id, is_default, method, &info)
            .await;
        match result_pm {
            Ok(pm) => Ok(pm),
            Err(error) => match error {
                CreatingPaymentMethodError::UnmappedError => Err(String::from(
                    "CreatingPaymentMethodError::UnmappedError Something wrong happen",
                )),
            },
        }
    }
}

#[cfg(test)]
mod test {

    use super::PaymentMethodCreateUsecase;
    use crate::core::{
        dto::{
            payment_method::{Method, PaymentMethod, PaymentMethodInfo, PixInfo},
            user::User,
        },
        usecase::driven::{
            creating_payment_method::{CreatingPaymentMethodError, MockCreatingPaymentMethod},
            reading_user::{MockReadingUser, ReadingUserError},
        },
    };
    use mockall::predicate;
    use uuid::Uuid;

    #[actix_rt::test]
    async fn should_succeed_creating_payment_method() {
        let user_id = Uuid::new_v4();
        let external_id = Uuid::new_v4();
        let is_default = true;
        let method = Method::Pix;
        let pix_info = PixInfo {
            key: String::from("any@email.com"),
            external_id: String::from("ABCDEFG"),
        };
        let info = PaymentMethodInfo::PixInfo(pix_info);
        let user = User {
            id: user_id,
            external_id,
        };
        let pm = PaymentMethod {
            id: Uuid::new_v4(),
            user_id,
            info: info.clone(),
            is_default,
            method,
        };
        let mut mock_ru = MockReadingUser::new();
        mock_ru
            .expect_list_by_id()
            .with(predicate::eq(external_id))
            .times(1)
            .return_const(Ok(user.clone()));
        let mut mock_cpm = MockCreatingPaymentMethod::new();
        mock_cpm
            .expect_create()
            .with(
                predicate::eq(user_id),
                predicate::eq(is_default),
                predicate::eq(method),
                predicate::eq(info.clone()),
            )
            .times(1)
            .return_const(Ok(pm.clone()));
        let payment_gateway_usecase = PaymentMethodCreateUsecase {
            reading_user: Box::new(mock_ru),
            creating_payment_method: Box::new(mock_cpm),
        };
        let result = payment_gateway_usecase
            .create(&external_id.to_string(), is_default, method, info.clone())
            .await;

        match result {
            Ok(resp) => {
                assert!(!resp.id.to_string().is_empty());
                assert_eq!(resp.user_id, user_id);
                assert_eq!(resp.method, method);
                assert_eq!(resp.is_default, is_default);
                assert_eq!(resp.info, info);
            }
            Err(error) => panic!(
                "should_succeed_creating_payment_method test went wrong: {}",
                error
            ),
        }
    }

    #[actix_rt::test]
    async fn should_fail_when_uuid_is_wrong() {
        let is_default = true;
        let method = Method::Pix;
        let pix_info = PixInfo {
            key: String::from("any@email.com"),
            external_id: String::from("ABCDEFG"),
        };
        let info = PaymentMethodInfo::PixInfo(pix_info);
        let mut mock_ru = MockReadingUser::new();
        mock_ru.expect_list_by_id().times(0);
        let mut mock_cpm = MockCreatingPaymentMethod::new();
        mock_cpm.expect_create().times(0);
        let payment_gateway_usecase = PaymentMethodCreateUsecase {
            reading_user: Box::new(mock_ru),
            creating_payment_method: Box::new(mock_cpm),
        };
        let result = payment_gateway_usecase
            .create("any-hash-not-uuid", is_default, method, info.clone())
            .await;

        match result {
            Ok(_) => panic!("should_fail_when_uuid_is_wrong test went wrong"),
            Err(error) => assert_eq!(error, String::from("external id provided isn't uuid")),
        }
    }

    #[actix_rt::test]
    async fn should_fail_when_user_provider_went_wrong() {
        let external_id = Uuid::new_v4();
        let is_default = true;
        let method = Method::Pix;
        let pix_info = PixInfo {
            key: String::from("any@email.com"),
            external_id: String::from("ABCDEFG"),
        };
        let info = PaymentMethodInfo::PixInfo(pix_info);
        let mut mock_ru = MockReadingUser::new();
        mock_ru
            .expect_list_by_id()
            .with(predicate::eq(external_id))
            .times(1)
            .return_const(Err(ReadingUserError::UserNotFoundError));
        let mut mock_cpm = MockCreatingPaymentMethod::new();
        mock_cpm.expect_create().times(0);
        let payment_gateway_usecase = PaymentMethodCreateUsecase {
            reading_user: Box::new(mock_ru),
            creating_payment_method: Box::new(mock_cpm),
        };
        let result = payment_gateway_usecase
            .create(&external_id.to_string(), is_default, method, info.clone())
            .await;

        match result {
            Ok(_) => panic!("should_fail_when_user_provider_went_wrong test went wrong"),
            Err(error) => assert_eq!(error, String::from("User Not found")),
        }
    }

    #[actix_rt::test]
    async fn should_fail_when_payment_method_provider_went_wrong() {
        let user_id = Uuid::new_v4();
        let external_id = Uuid::new_v4();
        let is_default = true;
        let method = Method::Pix;
        let pix_info = PixInfo {
            key: String::from("any@email.com"),
            external_id: String::from("ABCDEFG"),
        };
        let info = PaymentMethodInfo::PixInfo(pix_info);
        let user = User {
            id: user_id,
            external_id,
        };
        let mut mock_ru = MockReadingUser::new();
        mock_ru
            .expect_list_by_id()
            .with(predicate::eq(external_id))
            .times(1)
            .return_const(Ok(user.clone()));
        let mut mock_cpm = MockCreatingPaymentMethod::new();
        mock_cpm
            .expect_create()
            .with(
                predicate::eq(user_id),
                predicate::eq(is_default),
                predicate::eq(method),
                predicate::eq(info.clone()),
            )
            .times(1)
            .return_const(Err(CreatingPaymentMethodError::UnmappedError));
        let payment_gateway_usecase = PaymentMethodCreateUsecase {
            reading_user: Box::new(mock_ru),
            creating_payment_method: Box::new(mock_cpm),
        };
        let result = payment_gateway_usecase
            .create(&external_id.to_string(), is_default, method, info.clone())
            .await;

        match result {
            Ok(_) => panic!("should_fail_when_payment_method_provider_went_wrong test went wrong"),
            Err(error) => assert_eq!(
                error,
                String::from("CreatingPaymentMethodError::UnmappedError Something wrong happen")
            ),
        }
    }
}
