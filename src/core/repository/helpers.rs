use sqlx::{postgres::PgQueryResult, Pool, Postgres};
use uuid::Uuid;

use crate::core::dto::{
    invoice::InvoiceStatus,
    payment_method::{Method, PaymentMethodInfo},
};

pub async fn create_user(
    conn: &Pool<Postgres>,
    user_id: Uuid,
    external_id: Uuid,
) -> Result<PgQueryResult, sqlx::Error> {
    let q_user = format!(
        "INSERT INTO \"user\" (id, external_id) VALUES ('{}', '{}');",
        user_id, external_id
    );
    sqlx::query(&q_user).execute(conn).await
}

pub async fn delete_user(
    conn: &Pool<Postgres>,
    user_id: Uuid,
) -> Result<PgQueryResult, sqlx::Error> {
    let q_user = format!("DELETE FROM \"user\" WHERE id :: text = '{}';", user_id);
    sqlx::query(&q_user).execute(conn).await
}

pub async fn create_invoice(
    conn: &Pool<Postgres>,
    invoice_id: Uuid,
    user_id: Uuid,
    status: InvoiceStatus,
) -> Result<PgQueryResult, sqlx::Error> {
    let q_invoice = format!(
        "INSERT INTO invoice (id, user_id, status) VALUES ('{}', '{}', '{}');",
        invoice_id, user_id, status
    );
    sqlx::query(&q_invoice).execute(conn).await
}

pub async fn delete_invoice(
    conn: &Pool<Postgres>,
    invoice_id: Uuid,
) -> Result<PgQueryResult, sqlx::Error> {
    let q_invoice_item = format!(
        "DELETE FROM invoice_item WHERE invoice_id :: text = '{}';",
        invoice_id
    );
    sqlx::query(&q_invoice_item).execute(conn).await?;
    let q_invoice = format!("DELETE FROM invoice WHERE id :: text = '{}';", invoice_id);
    sqlx::query(&q_invoice).execute(conn).await
}

pub async fn create_gateway(
    conn: &Pool<Postgres>,
    gateway_id: Uuid,
    gateway_name: &str,
) -> Result<PgQueryResult, sqlx::Error> {
    let q_gateway = format!(
        "INSERT INTO gateway (id, name) VALUES ('{}', '{}');",
        gateway_id, gateway_name
    );
    sqlx::query(&q_gateway).execute(conn).await
}

pub async fn delete_gateway(
    conn: &Pool<Postgres>,
    gateway_id: Uuid,
) -> Result<PgQueryResult, sqlx::Error> {
    let q_gateway = format!("DELETE FROM gateway WHERE id :: text = '{}';", gateway_id);
    sqlx::query(&q_gateway).execute(conn).await
}

pub async fn create_payment_method(
    conn: &Pool<Postgres>,
    payment_method_id: Uuid,
    user_id: Uuid,
    gateway_id: Uuid,
    is_default: bool,
    method: Method,
    info: PaymentMethodInfo,
) -> Result<PgQueryResult, sqlx::Error> {
    let q_payment_method = format!(
        "INSERT INTO payment_method (id, user_id, gateway_id, is_default, method, info) VALUES ('{}','{}', '{}','{}','{}','{}');",
        payment_method_id,
        user_id,
        gateway_id,
        is_default,
        method,
        serde_json::to_string(&info).unwrap()
    );
    sqlx::query(&q_payment_method).execute(conn).await
}

pub async fn delete_payment_method(
    conn: &Pool<Postgres>,
    payment_method_id: Uuid,
) -> Result<PgQueryResult, sqlx::Error> {
    let q_payment_method = format!(
        "DELETE FROM payment_method WHERE id :: text = '{}';",
        payment_method_id
    );
    sqlx::query(&q_payment_method).execute(conn).await
}

pub async fn delete_charge(
    conn: &Pool<Postgres>,
    charge_id: Uuid,
) -> Result<PgQueryResult, sqlx::Error> {
    let q_charge = format!("DELETE FROM charge WHERE id :: text = '{}';", charge_id);
    sqlx::query(&q_charge).execute(conn).await
}
