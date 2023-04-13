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
        // instantiate the StreamConsumer client isntance
        let consumer: StreamConsumer<DefaultConsumerContext> = ClientConfig::new()
            .set("group.id", &config.group_id)
            .set("bootstrap.servers", &config.brokers)
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "10000")
            .set("enable.auto.commit", "false")
            .set("allow.auto.create.topics", "true")
            .create_with_context(DefaultConsumerContext)?;
        // subscribe StreamConsumer to given topic
        consumer.subscribe(&[&config.topic_id])?;

        tracing::info!("LogConsumer::create()::consumer started. listening on topic...");

        Ok(Box::leak(Box::new(Self { consumer })))
    }
}
