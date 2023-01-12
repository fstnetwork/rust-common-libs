use pulsar_client::{
    client::{Client, ClientConfiguration},
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
            tracing::info!("send to `my-topic`");
            let message_id = producer.send("my-topic", format!("doge-cry: {i}")).await.unwrap();
            tracing::info!("id: {}", message_id);
        }

        {
            tracing::info!("send to `my-topic2`");
            let message_id = producer.send("my-topic2", format!("doge-cry: {i}")).await.unwrap();
            tracing::info!("id: {}", message_id);
        }
    }
}
