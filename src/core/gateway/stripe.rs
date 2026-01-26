use crate::{
    config::{self},
    core::usecase::driven::gateway::{GatewayIntegration, GatewayIntegrationError},
};
use anyhow::{Error, anyhow};
use log::error;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

const STRIPE_URL: &str = "https://api.stripe.com";

#[derive(Clone)]
pub struct StripeGateway {
    api_key: String,
}

#[derive(Serialize, Deserialize)]
struct Intent {
    amount: String,
    currency: String,
    description: String,
}

async fn charge(
    api_key: &String,
    amount: f32,
    description: String,
) -> Result<bool, GatewayIntegrationError> {
    let client = reqwest::Client::new();
    let payload = Intent {
        amount: amount.to_string(),
        currency: "BRL".into(),
        description,
    };
    let resp: Result<reqwest::Response, reqwest::Error> = client
        .post(format!("{STRIPE_URL}/v1/payment_intents"))
        .basic_auth(api_key, None::<&str>)
        .json(&payload)
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
    api_key: &String,
    name: String,
    email: String,
) -> Result<bool, GatewayIntegrationError> {
    let client = reqwest::Client::new();
    let payload = Customer { name, email };
    let resp: Result<reqwest::Response, reqwest::Error> = client
        .post(format!("{STRIPE_URL}/v1/customers"))
        .basic_auth(api_key, None::<&str>)
        .json(&payload)
        .send()
        .await;

    match resp {
        Ok(body) => {
            if body.status().is_success() {
                Ok(true)
            } else {
                Err(GatewayIntegrationError::CustomerCreationError)
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
    api_key: &String,
    r#type: StripePaymentMethod,
) -> Result<bool, GatewayIntegrationError> {
    let client = reqwest::Client::new();
    let payload = PaymentMethod {
        r#type: r#type.to_string(),
    };
    let resp: Result<reqwest::Response, reqwest::Error> = client
        .post(format!("{STRIPE_URL}/v1/payment_methods"))
        .basic_auth(api_key, None::<&str>)
        .json(&payload)
        .send()
        .await;

    match resp {
        Ok(body) => {
            if body.status().is_success() {
                Ok(true)
            } else {
                Err(GatewayIntegrationError::CustomerCreationError)
            }
        }
        Err(err) => {
            error!("GatewayIntegration.create_customer :{:?}", err);
            Err(GatewayIntegrationError::CustomerCreationError)
        }
    }
}

#[async_trait::async_trait]
impl GatewayIntegration for StripeGateway {
    async fn charge(
        &self,
        amount: f32,
        description: String,
    ) -> Result<bool, GatewayIntegrationError> {
        charge(&self.api_key, amount, description).await
    }
    async fn create_customer(
        &self,
        name: String,
        email: String,
    ) -> Result<bool, GatewayIntegrationError> {
        create_customer(&self.api_key, name, email).await
    }
    async fn create_payment_method(&self, r#type: String) -> Result<bool, GatewayIntegrationError> {
        let converted_method = StripePaymentMethod::from_str(&r#type).expect("asd");
        create_payment_method(&self.api_key, converted_method).await
    }
}

impl StripeGateway {
    pub fn new() -> Self {
        let config = config::env_var::get_config();
        StripeGateway {
            api_key: config.gateway.stripe,
        }
    }
}

impl Default for StripeGateway {
    fn default() -> Self {
        Self::new()
    }
}
