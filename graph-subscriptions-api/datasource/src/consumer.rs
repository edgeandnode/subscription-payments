use std::collections::BTreeMap;

use anyhow::{Ok, Result};
use rdkafka::{
    consumer::{Consumer as _, DefaultConsumerContext, StreamConsumer},
    ClientConfig,
};

#[derive(Debug)]
pub struct ConsumerConfig {
    /// The Graph Subscriptions query result logs kafka topic id
    pub topic_id: String,
    /// Kafka consumer configuration paramaters
    pub config: BTreeMap<String, String>,
}

pub struct LogConsumer {
    /// The created Kafka Consumer Client
    pub consumer: StreamConsumer<DefaultConsumerContext>,
}

impl LogConsumer {
    /// Initialize the kafka StreamConsumer client instance for async subscribing of messages on the given topic.
    /// NOTE: this does not begin listening on the stream instance
    pub fn create(config: ConsumerConfig) -> Result<&'static Self> {
        tracing::info!(
            "LogConsumer::create()::initializing Kafka Stream Consumer... [{:?}]",
            config
        );
        let mut binding = ClientConfig::new();
        for (k, v) in config.config {
            binding.set(k, v);
        }
        // instantiate the StreamConsumer client isntance
        let consumer: StreamConsumer<DefaultConsumerContext> =
            binding.create_with_context(DefaultConsumerContext)?;
        // subscribe StreamConsumer to given topic
        consumer.subscribe(&[&config.topic_id])?;

        tracing::info!("LogConsumer::create()::consumer started. listening on topic...");

        Ok(Box::leak(Box::new(Self { consumer })))
    }
}
