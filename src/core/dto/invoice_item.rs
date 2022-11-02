use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Serialize)]
pub struct InvoiceItem {
    pub id: String,
    user_id: String,
    value: Decimal,
}
