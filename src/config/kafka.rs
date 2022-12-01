use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::{BaseConsumer, Consumer};
use rdkafka::producer::FutureProducer;

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
        .subscribe(topics)
        .expect("Can't subscribe to specified topics");

    consumer
}

pub fn get_producer() -> FutureProducer {
    let config = super::env_var::get_config();

    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", &config.kafka.url)
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Producer creation error");

    producer
}
