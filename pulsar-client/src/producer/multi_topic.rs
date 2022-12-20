use std::{collections::BTreeMap, sync::Arc};

use tokio::sync::RwLock;

use crate::{
    client::Client,
    error::Result,
    message::{Message, MessageId},
    producer::{Producer, ProducerConfiguration},
};

pub struct MultiTopicProducer {
    client: Client,
    configuration: ProducerConfiguration,
    producers: Arc<RwLock<BTreeMap<String, Producer>>>,
}

impl MultiTopicProducer {
    pub(crate) fn new(client: &Client, configuration: ProducerConfiguration) -> Self {
        Self { client: client.clone(), configuration, producers: Arc::default() }
    }

    // FIXME: allow: clippy bug https://github.com/rust-lang/rust-clippy/issues/4979
    #[allow(clippy::missing_const_for_fn)]
    /// producer options
    #[must_use]
    pub fn config(&self) -> &ProducerConfiguration { &self.configuration }

    /// list topics currently handled by this producer
    pub async fn topics(&self) -> Vec<String> {
        self.producers.read().await.keys().cloned().collect()
    }

    /// stops the producer
    ///
    /// # Errors
    ///
    /// if can not lookup partitioned topic
    pub async fn close_producer(&mut self, topic: impl Into<String>) -> Result<()> {
        let topic = topic.into();
        let partitions = self.client.lookup_partitioned_topic(&topic).await?;

        {
            let mut producers = self.producers.write().await;
            for topic in partitions {
                producers.remove(&topic);
            }
        }

        Ok(())
    }

    /// sends one message on a topic
    ///
    /// # Errors
    ///
    /// if can not send messages
    pub async fn send<T>(&self, topic: T, message: Message) -> Result<MessageId>
    where
        T: Into<String>,
    {
        let topic = topic.into();

        if let Some(producer) = self.producers.read().await.get(&topic) {
            return producer.send(message).await;
        }

        let producer = self.client.create_producer(&topic, &self.configuration).await?;
        let result = producer.send(message).await;

        // drop the read-write lock immediately after inserting
        {
            self.producers.write().await.insert(topic, producer);
        }
        result
    }
}
