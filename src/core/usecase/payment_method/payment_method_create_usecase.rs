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
                    "ReadingUserError::UnmappedError Something wrong happen",
                )),
            },
        }
    }
}
