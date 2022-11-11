use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InvoiceItem {
    pub id: Uuid,
    pub description: String,
    pub quantity: i32,
    pub amount: Decimal,
    pub currency: String,
}
