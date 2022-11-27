use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Debug, Clone, Copy)]
pub enum Method {
    Pix,
    CreditCard,
    UnmappedMethod,
}

impl From<&str> for Method {
    fn from(item: &str) -> Self {
        match item {
            "pix" => Method::Pix,
            "credit_card" => Method::CreditCard,
            _ => Method::UnmappedMethod,
        }
    }
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Method::Pix => write!(f, "pix"),
            Method::CreditCard => write!(f, "credit_card"),
            Method::UnmappedMethod => write!(f, "unknown"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PixInfo {
    pub key: String,
    pub external_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreditCardInfo {
    last4digit: String,
    flag: String,
    expire_data: String,
    external_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum PaymentMethodInfo {
    CreditCardInfo(CreditCardInfo),
    PixInfo(PixInfo),
}
#[derive(Serialize, Clone, Debug)]
pub struct PaymentMethod {
    pub id: Uuid,
    pub user_id: Uuid,
    pub is_default: bool,
    pub method: Method,
    pub info: PaymentMethodInfo,
}
