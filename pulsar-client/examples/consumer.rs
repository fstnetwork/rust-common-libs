use pulsar_client::{
    client::{Client, ClientConfiguration},
    consumer::{ConsumerConfiguration, ConsumerType},
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
    let mut config = ConsumerConfiguration::new();
    config.set_consumer_type(ConsumerType::Shared);

    let mut consumer = client
        .multi_subscribe(
            vec![
                "persistent://loc-qa-test2/data-process-task/\
                 9494e6de-8cfe-44db-9669-f69d64b38236-12",
                "persistent://public/default/my-topic",
            ],
            "default2",
            &config,
        )
        .await
        .unwrap();

    loop {
        let message = consumer.receive().await.unwrap();
        tracing::info!("id: {}", message.get_message_id(),);

        consumer.acknowledge_id(&message.get_message_id()).await.unwrap();
    }
}
