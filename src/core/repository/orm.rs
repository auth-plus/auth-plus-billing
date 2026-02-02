use sqlx::{Pool, Postgres, postgres::PgQueryResult};
use uuid::Uuid;

use crate::core::{
    dto::{
        charge::ChargeStatus,
        invoice::InvoiceStatus,
        payment_method::{Method, PaymentMethodInfo},
    },
    gateway::GatewayDAO,
};

pub async fn create_user(
    conn: &Pool<Postgres>,
    user_id: Uuid,
    external_id: Uuid,
) -> Result<PgQueryResult, sqlx::Error> {
    sqlx::query("INSERT INTO \"user\" (id, external_id) VALUES ($1, $2);")
        .bind(user_id)
        .bind(external_id)
        .execute(conn)
        .await
}

pub async fn delete_user(
    conn: &Pool<Postgres>,
    user_id: Uuid,
) -> Result<PgQueryResult, sqlx::Error> {
    sqlx::query("DELETE FROM \"user\" WHERE id::text = $1;")
        .bind(user_id.to_string())
        .execute(conn)
        .await
}

pub async fn create_invoice(
    conn: &Pool<Postgres>,
    invoice_id: Uuid,
    user_id: Uuid,
    status: InvoiceStatus,
) -> Result<PgQueryResult, sqlx::Error> {
    sqlx::query("INSERT INTO invoice (id, user_id, status) VALUES ($1, $2, $3);")
        .bind(invoice_id)
        .bind(user_id)
        .bind(status.to_string())
        .execute(conn)
        .await
}

pub async fn delete_invoice(
    conn: &Pool<Postgres>,
    invoice_id: Uuid,
) -> Result<PgQueryResult, sqlx::Error> {
    sqlx::query("DELETE FROM invoice_item WHERE invoice_id::text = $1;")
        .bind(invoice_id.to_string())
        .execute(conn)
        .await?;
    sqlx::query("DELETE FROM invoice WHERE id::text = $1;")
        .bind(invoice_id.to_string())
        .execute(conn)
        .await
}

pub async fn create_gateway(
    conn: &Pool<Postgres>,
    gateway_id: Uuid,
    gateway_name: &str,
    priority: i32,
) -> Result<PgQueryResult, sqlx::Error> {
    sqlx::query("INSERT INTO gateway (id, name, priority) VALUES ($1, $2, $3);")
        .bind(gateway_id)
        .bind(gateway_name)
        .bind(priority)
        .execute(conn)
        .await
}

pub async fn delete_gateway(
    conn: &Pool<Postgres>,
    gateway_id: Uuid,
) -> Result<PgQueryResult, sqlx::Error> {
    sqlx::query("DELETE FROM gateway WHERE id::text = $1;")
        .bind(gateway_id.to_string())
        .execute(conn)
        .await
}

pub async fn read_main_gateway(conn: &Pool<Postgres>) -> Result<GatewayDAO, sqlx::Error> {
    sqlx::query_as::<_, GatewayDAO>("SELECT * FROM gateway WHERE name = 'stripe' LIMIT 1;")
        .fetch_one(conn)
        .await
}

pub async fn create_payment_method(
    conn: &Pool<Postgres>,
    payment_method_id: Uuid,
    user_id: Uuid,
    is_default: bool,
    method: Method,
    info: PaymentMethodInfo,
) -> Result<PgQueryResult, sqlx::Error> {
    sqlx::query("INSERT INTO payment_method (id, user_id, is_default, method, info) VALUES ($1, $2, $3, $4, $5);")
    .bind(payment_method_id)
    .bind(user_id)
    .bind(is_default)
    .bind(method.to_string())
    .bind(serde_json::to_value(&info)
    .unwrap())
    .execute(conn).await
}

pub async fn delete_payment_method(
    conn: &Pool<Postgres>,
    payment_method_id: Uuid,
) -> Result<PgQueryResult, sqlx::Error> {
    sqlx::query("DELETE FROM payment_method WHERE id::text = $1;")
        .bind(payment_method_id.to_string())
        .execute(conn)
        .await
}

pub async fn create_charge(
    conn: &Pool<Postgres>,
    charge_id: Uuid,
    invoice_id: Uuid,
    payment_method_id: Uuid,
    status: ChargeStatus,
) -> Result<PgQueryResult, sqlx::Error> {
    sqlx::query(
        "INSERT INTO charge (id, invoice_id, payment_method_id, status) VALUES ($1, $2, $3, $4);",
    )
    .bind(charge_id)
    .bind(invoice_id)
    .bind(payment_method_id)
    .bind(status.to_string())
    .execute(conn)
    .await
}

pub async fn delete_charge(
    conn: &Pool<Postgres>,
    charge_id: Uuid,
) -> Result<PgQueryResult, sqlx::Error> {
    sqlx::query("DELETE FROM charge WHERE id::text = $1;")
        .bind(charge_id.to_string())
        .execute(conn)
        .await
}

pub async fn create_gateway_integration(
    conn: &Pool<Postgres>,
    id: Uuid,
    gateway_id: Uuid,
    user_id: Uuid,
    gateway_user_id: String,
) -> Result<PgQueryResult, sqlx::Error> {
    sqlx::query("INSERT INTO gateway_integration (id, gateway_id, user_id, gateway_external_user_id) VALUES ($1, $2, $3, $4);").bind(id).bind(gateway_id).bind(user_id).bind(gateway_user_id).execute(conn).await
}

pub async fn delete_gateway_integration(
    conn: &Pool<Postgres>,
    gateway_integration_id: Uuid,
) -> Result<PgQueryResult, sqlx::Error> {
    sqlx::query("DELETE FROM gateway_integration WHERE id::text = $1;")
        .bind(gateway_integration_id.to_string())
        .execute(conn)
        .await
}

pub async fn delete_gateway_integration_by_pm(
    conn: &Pool<Postgres>,
    payment_method_id: Uuid,
) -> Result<PgQueryResult, sqlx::Error> {
    sqlx::query("DELETE FROM gateway_integration WHERE payment_method_id::text = $1;")
        .bind(payment_method_id.to_string())
        .execute(conn)
        .await
}

#[cfg(test)]
mod test {
    use fake::{Fake, uuid::UUIDv4};
    use uuid::Uuid;

    use crate::config::database::get_connection;

    use super::create_user;

    #[actix_rt::test]
    async fn should_create_user() {
        let conn = get_connection().await;
        let user_id: Uuid = UUIDv4.fake();
        let external_id: Uuid = UUIDv4.fake();
        let result = create_user(&conn, user_id, external_id).await;
        match result {
            Ok(_) => {}
            Err(error) => panic!("should_create_user test went wrong: {:?}", error),
        }
    }
}
