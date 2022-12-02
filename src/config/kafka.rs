use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::{BaseConsumer, Consumer};

pub fn get_consumer(topics: &[&str]) -> BaseConsumer {
    let config = super::env_var::get_config();

    let consumer: BaseConsumer = ClientConfig::new()
        .set("group.id", "billing_consumer")
        .set("bootstrap.servers", &config.kafka.url)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .set_log_level(RDKafkaLogLevel::Debug)
        .create()
        .expect("Consumer creation failed");

    consumer
        .subscribe(&topics.to_vec())
        .expect("Can't subscribe to specified topics");

    consumer
}
