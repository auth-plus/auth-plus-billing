pub mod case;
use rdkafka::message::Message;

use crate::config::kafka::get_consumer;
use log::{error, info};

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
            Err(e) => error!("Kafka error: {}", e),
            Ok(m) => {
                let payload = match m.payload_view::<str>() {
                    None => "",
                    Some(Ok(s)) => s,
                    Some(Err(e)) => {
                        error!("Error while deserializing message payload: {:?}", e);
                        ""
                    }
                };
                info!("key: '{:?}', payload: '{}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                      m.key(), payload, m.topic(), m.partition(), m.offset(), m.timestamp());
                match case::switch_case(m.topic(), payload).await {
                    Ok(result) => info!("{:?}", result),
                    Err(error) => error!("{:?}", error),
                }
            }
        };
    }
}
