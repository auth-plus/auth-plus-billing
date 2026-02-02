use std::time::Duration;

pub use sqlx::postgres::{PgPool, PgPoolOptions};

pub async fn get_connection() -> PgPool {
    let config = super::env_var::get_config();
    PgPoolOptions::new()
        .max_connections(20) // Tune based on load
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(300))
        .max_lifetime(Duration::from_secs(1800))
        .connect(&config.database.url)
        .await
        .expect("Could not connect to database")
}
