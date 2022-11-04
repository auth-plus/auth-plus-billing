pub use sqlx::postgres::{PgConnection, PgPool};

pub async fn get_connection() -> PgPool {
    let config = super::env_var::get_config();
    PgPool::connect(&config.database.url)
        .await
        .expect("Could not connect to database")
}
