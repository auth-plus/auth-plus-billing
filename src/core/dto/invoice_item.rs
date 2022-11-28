use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct InvoiceItem {
    pub id: Option<Uuid>,
    pub description: String,
    pub quantity: u16,
    pub amount: Decimal,
    pub currency: String,
}
