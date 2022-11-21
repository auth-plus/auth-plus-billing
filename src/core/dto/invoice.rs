use std::fmt;

use super::invoice_item::InvoiceItem;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize, Debug, Clone, Copy)]
pub enum InvoiceStatus {
    Draft,
    Pending,
    ChargedWithError,
    Paid,
    Cancelled,
    Uncollectible,
    Refunded,
    InProtest,
    Chargeback,
    UnmappedStatus,
}

impl From<&str> for InvoiceStatus {
    fn from(item: &str) -> Self {
        match item {
            "draft" => InvoiceStatus::Draft,
            "pending" => InvoiceStatus::Pending,
            "charged_with_error" => InvoiceStatus::ChargedWithError,
            "paid" => InvoiceStatus::Paid,
            "cancelled" => InvoiceStatus::Cancelled,
            "uncollectible" => InvoiceStatus::Uncollectible,
            "refunded" => InvoiceStatus::Refunded,
            "in_protest" => InvoiceStatus::InProtest,
            "chargeback" => InvoiceStatus::Chargeback,
            _ => InvoiceStatus::UnmappedStatus,
        }
    }
}

impl fmt::Display for InvoiceStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InvoiceStatus::Draft => write!(f, "draft"),
            InvoiceStatus::Pending => write!(f, "pending"),
            InvoiceStatus::ChargedWithError => write!(f, "charged_with_error"),
            InvoiceStatus::Paid => write!(f, "paid"),
            InvoiceStatus::Cancelled => write!(f, "cancelled"),
            InvoiceStatus::Uncollectible => write!(f, "uncollectible"),
            InvoiceStatus::Refunded => write!(f, "refunded"),
            InvoiceStatus::InProtest => write!(f, "in_protest"),
            InvoiceStatus::Chargeback => write!(f, "chargeback"),
            InvoiceStatus::UnmappedStatus => write!(f, "unknown"),
        }
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct Invoice {
    pub id: Uuid,
    pub user_id: Uuid,
    pub status: InvoiceStatus,
    pub itens: Vec<InvoiceItem>,
}
