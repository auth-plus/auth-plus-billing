use std::env;

#[derive(Clone)]
pub struct App {
    pub env: String,
    pub name: String,
    pub port: u16,
}

#[derive(Clone)]
pub struct Database {
    pub url: String,
}

#[derive(Clone)]
pub struct Kafka {
    pub url: String,
}

#[derive(Clone)]
pub struct Config {
    pub app: App,
    pub database: Database,
    pub kafka: Kafka,
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
    let db_url = env::var("DATABASE_URL").expect("RUST_ENV is not set");
    let database = Database { url: db_url };
    let kafka = Kafka {
        url: env::var("KAFKA_HOST").expect("KAFKA_HOST is not set"),
    };
    Config {
        app,
        database,
        kafka,
    }
}
