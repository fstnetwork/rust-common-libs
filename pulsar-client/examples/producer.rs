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

    let producer =
        client.create_producer("persistent://public/default/my-topic2", &config).await.unwrap();

    for i in 0..10 {
        let message_id = producer.send(format!("doge-cry: {i}")).await.unwrap();
        tracing::info!("id: {}", message_id);
    }
}
