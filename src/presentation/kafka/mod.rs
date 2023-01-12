use rdkafka::message::Message;

use crate::config::kafka::get_consumer;

#[tokio::main]
pub async fn start() -> std::io::Result<()> {
    let topics = &[
        "2FA_EMAIL_CREATED",
        "2FA_PHONE_CREATED",
        "2FA_EMAIL_SENT",
        "2FA_PHONE_SENT",
        "USER_CREATED",
        "ORGANIZATION_CREATED",
    ];
    let consumer = get_consumer(topics);
    loop {
        match consumer.recv().await {
            Err(e) => println!("Kafka error: {}", e),
            Ok(m) => {
                let payload = match m.payload_view::<str>() {
                    None => "",
                    Some(Ok(s)) => s,
                    Some(Err(e)) => {
                        println!("Error while deserializing message payload: {:?}", e);
                        ""
                    }
                };
                println!("key: '{:?}', payload: '{}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                      m.key(), payload, m.topic(), m.partition(), m.offset(), m.timestamp());
            }
        };
    }
}
