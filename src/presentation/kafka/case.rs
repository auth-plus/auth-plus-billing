use crate::{core, presentation::http::routes::user::CreateUserInputSchema};
use log::error;

pub async fn switch_case(topic: &str, data: &str) -> Result<(), String> {
    let core_x = core::get_core().await;
    match topic {
        "2FA_EMAIL_CREATED" | "2FA_PHONE_CREATED" | "2FA_EMAIL_SENT" | "2FA_PHONE_SENT" => {
            error!("Sorry I don't intend to charge this action yet");
            Ok(())
        }
        "USER_CREATED" => {
            let json: CreateUserInputSchema =
                serde_json::from_str(data).expect("data on Kafka was no CreateUserInputSchema");
            match core_x.user.create.create_user(&json.external_id).await {
                Ok(_) => Ok(()),
                Err(error) => {
                    let resp = format!("Something wrong happen: {}", error);
                    Err(resp)
                }
            }
        }
        "ORGANIZATION_CREATED" => {
            error!("Sorry I don't intend to charge this action yet");
            Ok(())
        }
        _ => Err(String::from("UnmappedError")),
    }
}
