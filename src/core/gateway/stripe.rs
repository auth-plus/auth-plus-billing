use crate::{
    config::{self},
    core::usecase::driven::gateway::{GatewayIntegration, GatewayIntegrationError},
};
use anyhow::{Error, anyhow};
use log::error;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr, time::Duration};

#[derive(Clone)]
pub struct StripeGateway {
    api_key: String,
    url: String,
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
) -> Result<bool, GatewayIntegrationError> {
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
                Err(GatewayIntegrationError::ChargeError)
            }
        }
        Err(err) => {
            error!("GatewayIntegration.charge :{:?}", err);
            Err(GatewayIntegrationError::ChargeError)
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Customer {
    name: String,
    email: String,
}

async fn create_customer(
    host: &String,
    api_key: &String,
    name: &str,
    email: &str,
) -> Result<bool, GatewayIntegrationError> {
    let client = reqwest::Client::new();
    let payload = Customer {
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

    match resp {
        Ok(body) => {
            eprintln!("{}", body.status());
            eprintln!("{}", body.url());
            if body.status().is_success() {
                Ok(true)
            } else {
                error!("GatewayIntegration.create_customer: Stripe return not 2XX");
                Err(GatewayIntegrationError::NotSuccessfulReturn)
            }
        }
        Err(err) => {
            error!("GatewayIntegration.create_customer :{:?}", err);
            Err(GatewayIntegrationError::CustomerCreationError)
        }
    }
}

#[derive(Serialize, Debug, Clone, Copy, PartialEq, Deserialize)]
enum StripePaymentMethod {
    Boleto,
    Card,
    Paypal,
    Pix,
    SamsungPay,
}
// Implement the Display trait
impl fmt::Display for StripePaymentMethod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StripePaymentMethod::Boleto => write!(f, "boleto"),
            StripePaymentMethod::Card => write!(f, "card"),
            StripePaymentMethod::Paypal => write!(f, "paypal"),
            StripePaymentMethod::Pix => write!(f, "pix"),
            StripePaymentMethod::SamsungPay => write!(f, "samsung_pay"),
        }
    }
}
impl FromStr for StripePaymentMethod {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "boleto" => Ok(StripePaymentMethod::Boleto),
            "card" => Ok(StripePaymentMethod::Card),
            "paypal" => Ok(StripePaymentMethod::Paypal),
            "pix" => Ok(StripePaymentMethod::Pix),
            "samsung_pay" => Ok(StripePaymentMethod::SamsungPay),
            _ => Err(anyhow!("Unknown StripePaymentMethod: {}", s)),
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
) -> Result<bool, GatewayIntegrationError> {
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

    match resp {
        Ok(body) => {
            if body.status().is_success() {
                Ok(true)
            } else {
                Err(GatewayIntegrationError::PaymentMethodTransformError)
            }
        }
        Err(err) => {
            error!("GatewayIntegration.create_payment_method :{:?}", err);
            Err(GatewayIntegrationError::PaymentMethodTransformError)
        }
    }
}

#[async_trait::async_trait]
impl GatewayIntegration for StripeGateway {
    async fn charge(
        &self,
        amount: f32,
        description: &str,
    ) -> Result<bool, GatewayIntegrationError> {
        charge(&self.url, &self.api_key, amount, description).await
    }
    async fn create_customer(
        &self,
        name: &str,
        email: &str,
    ) -> Result<bool, GatewayIntegrationError> {
        create_customer(&self.url, &self.api_key, name, email).await
    }
    async fn create_payment_method(&self, r#type: &str) -> Result<bool, GatewayIntegrationError> {
        let converted_method = StripePaymentMethod::from_str(r#type)
            .map_err(|_| GatewayIntegrationError::PaymentMethodTransformError)?;
        create_payment_method(&self.url, &self.api_key, converted_method).await
    }
}

impl StripeGateway {
    pub fn new() -> Self {
        let config = config::env_var::get_config();
        StripeGateway {
            api_key: config.gateway.stripe.key,
            url: config.gateway.stripe.url,
        }
    }
}

impl Default for StripeGateway {
    fn default() -> Self {
        Self::new()
    }
}
