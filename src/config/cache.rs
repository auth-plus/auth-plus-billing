use redis::{Client, Connection};

pub fn get_connection() -> Connection {
    let config = super::env_var::get_config();
    let client =
        Client::open(format!("redis://{}", config.cache.url)).expect("Could not connect to redis");
    client.get_connection().expect("Could not connect to redis")
}
