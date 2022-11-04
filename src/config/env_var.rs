use std::env;

pub struct App {
    pub env: String,
    pub name: String,
    pub port: u16,
}

pub struct Database {
    pub url: String,
}

pub struct Config {
    pub app: App,
    pub database: Database,
}

pub fn get_config() -> Config {
    let app_env = env::var("RUST_ENV").expect("RUST_ENV is not set");
    let app_name = env::var("APP_NAME").expect("APP_NAME is not set");
    let app_port_string: String = env::var("PORT").expect("PORT is not set");
    let app_port: u16 = app_port_string.parse::<u16>().unwrap();
    let app = App {
        env: app_env,
        name: app_name,
        port: app_port,
    };
    let db_host = env::var("DATABASE_HOST").expect("RUST_ENV is not set");
    let db_user = env::var("DATABASE_USER").expect("APP_NAME is not set");
    let db_pw: String = env::var("DATABASE_PASSWORD").expect("PORT is not set");
    let db_name: String = env::var("DATABASE_DATABASE").expect("PORT is not set");
    let db_port: String = env::var("DATABASE_PORT").expect("PORT is not set");
    let url = format!(
        "postgres://{}:{}@{}:{}/{}",
        db_user, db_pw, db_host, db_port, db_name
    );
    let database = Database { url };
    Config { app, database }
}
