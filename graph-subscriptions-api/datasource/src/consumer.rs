use std::collections::BTreeMap;

use anyhow::{Ok, Result};
use rdkafka::{
    consumer::{Consumer as _, DefaultConsumerContext, StreamConsumer},
    ClientConfig,
};

#[derive(Debug)]
pub struct ConsumerConfig {
    /// URL to connect to the kafka broker instance
    pub brokers: String,
    /// The kafka logs group id
    pub group_id: String,
    /// The Graph Subscriptions query result logs kafka topic id
    pub topic_id: String,
    /// Additional configuration paramters for the kafka consumer
    /// For example, if connecting to a prod cluster via SSL instead of local
    pub additional_config: Option<BTreeMap<String, String>>,
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
        let client_config = binding
            .set("group.id", &config.group_id)
            .set("bootstrap.servers", &config.brokers)
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "10000")
            .set("enable.auto.commit", "false")
            .set("allow.auto.create.topics", "true");
        if let Some(additional_config) = config.additional_config {
            for (k, v) in additional_config {
                client_config.set(k, v);
            }
        }
        // instantiate the StreamConsumer client isntance
        let consumer: StreamConsumer<DefaultConsumerContext> =
            client_config.create_with_context(DefaultConsumerContext)?;
        // subscribe StreamConsumer to given topic
        consumer.subscribe(&[&config.topic_id])?;

        tracing::info!("LogConsumer::create()::consumer started. listening on topic...");

        Ok(Box::leak(Box::new(Self { consumer })))
    }
}
