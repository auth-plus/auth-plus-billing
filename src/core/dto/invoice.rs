use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Serialize)]
pub struct Invoice {
    pub id: String,
    pub value: Decimal,
}
