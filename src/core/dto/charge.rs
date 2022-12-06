use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Debug, Clone, Copy, Deserialize)]
pub enum ChargeStatus {
    Failed,
    Progress,
    Succeed,
    UnmappedStatus,
}

impl From<&str> for ChargeStatus {
    fn from(item: &str) -> Self {
        match item {
            "failed" => ChargeStatus::Failed,
            "progress" => ChargeStatus::Progress,
            "succeed" => ChargeStatus::Succeed,
            _ => ChargeStatus::UnmappedStatus,
        }
    }
}

impl fmt::Display for ChargeStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ChargeStatus::Failed => write!(f, "failed"),
            ChargeStatus::Progress => write!(f, "progress"),
            ChargeStatus::Succeed => write!(f, "succeed"),
            ChargeStatus::UnmappedStatus => write!(f, "unknown"),
        }
    }
}

#[derive(Serialize, Clone, Debug, Deserialize)]
pub struct Charge {
    pub id: Uuid,
    pub invoice_id: Uuid,
    pub status: ChargeStatus,
    pub payment_method_id: Uuid,
}
