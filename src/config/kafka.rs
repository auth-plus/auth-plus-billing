use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::{Consumer, DefaultConsumerContext};

// A type alias with your custom consumer can be created for convenience.
pub type LoggingConsumer = StreamConsumer<DefaultConsumerContext>;

pub fn get_consumer(topics: &[&str]) -> LoggingConsumer {
    let config = super::env_var::get_config();
    let context = DefaultConsumerContext;
    let consumer: LoggingConsumer = ClientConfig::new()
        .set("group.id", "auth-plus-authentication")
        .set("bootstrap.servers", &config.kafka.url)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .set_log_level(RDKafkaLogLevel::Debug)
        .create_with_context(context)
        .expect("Consumer creation failed");
    consumer
        .subscribe(topics)
        .expect("Can't subscribe to specified topics");
    consumer
}
