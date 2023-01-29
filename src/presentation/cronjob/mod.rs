use chrono::Utc;
use cron::Schedule;
use std::str::FromStr;

use crate::core::get_core;

#[tokio::main]
pub async fn start() -> std::io::Result<()> {
    let schedule = Schedule::from_str("0 1 * * * *").unwrap();
    for _ in schedule.upcoming(Utc) {
        let result = retry_charging_invoice_with_error().await;
        result.expect("Error");
    }
    Ok(())
}

async fn retry_charging_invoice_with_error() -> Result<(), String> {
    let core_x = get_core().await;
    let invoices = core_x
        .invoice
        .list
        .get_all_invoices_to_charge()
        .await
        .expect("Selecting invoice with error went wrong");
    let mut error_list = Vec::new();
    for inv in invoices {
        let result = core_x
            .charge
            .create
            .create_charge(&inv.id.to_string())
            .await;
        match result {
            Ok(_) => (),
            Err(error) => error_list.push(error),
        }
    }
    if error_list.len() > 0 {
        let all_errors = error_list.join(";");
        return Err(all_errors);
    }
    Ok(())
}
