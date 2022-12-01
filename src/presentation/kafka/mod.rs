use rdkafka::message::Message;

use crate::config::kafka::get_consumer;

#[tokio::main]
pub async fn start() -> std::io::Result<()> {
    let topics = &["INVOICE_CHARGE"];
    let consumer = get_consumer(topics);
    loop {
        for msg in consumer.iter() {
            let msg = msg.unwrap();
            let key: &str = msg.key_view().unwrap().unwrap();
            let value = msg.payload().unwrap();
            println!("key: {:?}\nvalue: {:?}", key, value)
        }
    }
}
