use crate::{
    config::{self},
    core::{
        gateway::stripe_models::{
            CustomerInput, CustomerOutput, PaymentMethodOutput, StripePaymentMethod,
        },
        usecase::driven::gateway::{
            GatewayAPI, GatewayAPIError, GatewayPaymentMethod, GatewayUser,
        },
    },
};
use log::error;
use serde::{Deserialize, Serialize};
use std::{str::FromStr, time::Duration};
use uuid::Uuid;

#[derive(Clone)]
pub struct StripeGateway {
    api_key: String,
    url: String,
    id: Option<Uuid>,
}

#[derive(Serialize, Deserialize)]
struct Intent {
    amount: String,
    currency: String,
    description: String,
}

async fn charge(
    host: &String,
    api_key: &String,
    amount: f32,
    description: &str,
) -> Result<bool, GatewayAPIError> {
    let client = reqwest::Client::new();
    let amount_cents = (amount * 100.0).round() as i64;
    let payload = Intent {
        amount: amount_cents.to_string(),
        currency: "BRL".into(),
        description: description.into(),
    };
    let url: String = format!("{host}/v1/payment_intents");
    let resp: Result<reqwest::Response, reqwest::Error> = client
        .post(url)
        .basic_auth(api_key, None::<&str>)
        .json(&payload)
        .timeout(Duration::from_secs(30))
        .send()
        .await;

    match resp {
        Ok(body) => {
            if body.status().is_success() {
                Ok(true)
            } else {
                Err(GatewayAPIError::ChargeError)
            }
        }
        Err(err) => {
            error!("GatewayIntegration.charge :{:?}", err);
            Err(GatewayAPIError::ChargeError)
        }
    }
}

async fn create_customer(
    host: &String,
    api_key: &String,
    name: &str,
    email: &str,
) -> Result<GatewayUser, GatewayAPIError> {
    let client = reqwest::Client::new();
    let payload = CustomerInput {
        name: name.into(),
        email: email.into(),
    };
    let resp: Result<reqwest::Response, reqwest::Error> = client
        .post(format!("{host}/v1/customers"))
        .basic_auth(api_key, None::<&str>)
        .json(&payload)
        .timeout(Duration::from_secs(30))
        .send()
        .await;

    let body = match resp {
        Ok(resp) => {
            if resp.status().is_success() {
                resp.json::<CustomerOutput>().await
            } else {
                error!("GatewayIntegration.create_customer: Stripe return not 2XX");
                return Err(GatewayAPIError::NotSuccessfulReturn);
            }
        }
        Err(err) => {
            error!("GatewayIntegration.create_customer :{:?}", err);
            return Err(GatewayAPIError::CustomerCreationError);
        }
    };
    match body {
        Ok(body) => {
            let gateway_user: GatewayUser = GatewayUser {
                id: body.id,
                name: body.name,
                email: body.email.unwrap_or_else(|| email.into()),
            };
            Ok(gateway_user)
        }
        Err(err) => {
            error!("GatewayIntegration.create_customer :{:?}", err);
            Err(GatewayAPIError::CustomerCreationError)
        }
    }
}

#[derive(Serialize, Deserialize)]
struct PaymentMethod {
    r#type: String,
}

async fn create_payment_method(
    host: &String,
    api_key: &String,
    r#type: StripePaymentMethod,
) -> Result<GatewayPaymentMethod, GatewayAPIError> {
    let client = reqwest::Client::new();
    let payload = PaymentMethod {
        r#type: r#type.to_string(),
    };
    let resp: Result<reqwest::Response, reqwest::Error> = client
        .post(format!("{host}/v1/payment_methods"))
        .basic_auth(api_key, None::<&str>)
        .json(&payload)
        .timeout(Duration::from_secs(30))
        .send()
        .await;
    let body = match resp {
        Ok(resp) => {
            if resp.status().is_success() {
                resp.json::<PaymentMethodOutput>().await
            } else {
                error!("GatewayIntegration.create_payment_method: Stripe return not 2XX");
                return Err(GatewayAPIError::NotSuccessfulReturn);
            }
        }
        Err(err) => {
            error!("GatewayIntegration.create_payment_method :{:?}", err);
            return Err(GatewayAPIError::PaymentMethodCreationError);
        }
    };

    match body {
        Ok(body) => {
            let pm = GatewayPaymentMethod { id: body.id };
            Ok(pm)
        }
        Err(err) => {
            error!("GatewayIntegration.create_payment_method :{:?}", err);
            Err(GatewayAPIError::PaymentMethodCreationError)
        }
    }
}

#[async_trait::async_trait]
impl GatewayAPI for StripeGateway {
    fn set_id(&mut self, id: uuid::Uuid) -> Result<bool, GatewayAPIError> {
        self.id = Some(id);
        Ok(true)
    }
    fn get_id(&self) -> uuid::Uuid {
        match self.id {
            Some(uid) => uid,
            None => panic!("Trying to get a gateway_id without setting"),
        }
    }
    async fn charge(&self, amount: f32, description: &str) -> Result<bool, GatewayAPIError> {
        charge(&self.url, &self.api_key, amount, description).await
    }
    async fn create_customer(
        &self,
        name: &str,
        email: &str,
    ) -> Result<GatewayUser, GatewayAPIError> {
        create_customer(&self.url, &self.api_key, name, email).await
    }
    async fn create_payment_method(
        &self,
        r#type: &str,
    ) -> Result<GatewayPaymentMethod, GatewayAPIError> {
        let converted_method = StripePaymentMethod::from_str(r#type)
            .map_err(|_| GatewayAPIError::PaymentMethodTransformError)?;
        create_payment_method(&self.url, &self.api_key, converted_method).await
    }
}

impl StripeGateway {
    pub fn new() -> Self {
        let config = config::env_var::get_config();
        StripeGateway {
            api_key: config.gateway.stripe.key,
            url: config.gateway.stripe.url,
            id: None,
        }
    }
}

impl Default for StripeGateway {
    fn default() -> Self {
        Self::new()
    }
}
