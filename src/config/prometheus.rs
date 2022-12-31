use lazy_static::lazy_static;
use prometheus::{Encoder, IntCounter, Registry, TextEncoder};

pub struct Prometheus {}

lazy_static! {
    static ref REGISTRY: Registry =
        Registry::new_custom(Some(String::from("auth_plus_billing")), None).unwrap();
    pub static ref C_HTTP_SUCCESS: IntCounter =
        IntCounter::new("success_counter", "counter 20X").unwrap();
    pub static ref C_HTTP_FAIL: IntCounter =
        IntCounter::new("failed_counter", "counter 50X").unwrap();
}

impl Prometheus {
    pub fn init() {
        REGISTRY.register(Box::new(C_HTTP_SUCCESS.clone())).unwrap();
        REGISTRY.register(Box::new(C_HTTP_FAIL.clone())).unwrap();
    }

    pub fn export() -> String {
        let encoder = TextEncoder::new();
        let mut buffer = vec![];
        if let Err(e) = encoder.encode(&REGISTRY.gather(), &mut buffer) {
            eprintln!("could not encode prometheus metrics: {}", e);
        };
        let mut response =
            String::from_utf8(buffer.clone()).expect("Failed to convert bytes to string");
        buffer.clear();

        let mut buffer2 = vec![];
        if let Err(e) = encoder.encode(&prometheus::gather(), &mut buffer2) {
            eprintln!("could not encode prometheus metrics: {}", e);
        };
        let response_default =
            String::from_utf8(buffer2.clone()).expect("Failed to convert bytes to string");
        buffer2.clear();
        response.push_str(&response_default);
        response
    }
}
