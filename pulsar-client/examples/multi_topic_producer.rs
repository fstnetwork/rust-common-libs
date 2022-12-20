use pulsar_client::{
    client::{Client, ClientConfiguration},
    message::Message,
    producer::ProducerConfiguration,
};
use tracing::Level;
use tracing_subscriber::{filter::LevelFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(LevelFilter::from_level(Level::TRACE))
        .with(fmt::layer())
        .init();

    let config = ClientConfiguration::new();
    let client = Client::new(&"pulsar://localhost:6650".parse().unwrap(), &config).unwrap();
    let mut config = ProducerConfiguration::new();
    config.set_producer_name("doge-producer").unwrap();

    let producer = client.multi_topic_producer(config);

    for i in 0..10 {
        {
            let message = Message::with_content(format!("doge-cry: {i}").as_bytes());

            tracing::info!("send to `my-topic`");
            let message_id = producer.send("my-topic", message).await.unwrap();
            tracing::info!("id: {}", message_id);
        }

        {
            let message = Message::with_content(format!("doge-cry: {i}").as_bytes());

            tracing::info!("send to `my-topic2`");
            let message_id = producer.send("my-topic2", message).await.unwrap();
            tracing::info!("id: {}", message_id);
        }
    }
}
