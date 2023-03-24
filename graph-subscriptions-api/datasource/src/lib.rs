mod consumer;
mod models;

use anyhow::{ensure, Ok, Result};
pub use models::*;

use consumer::{ConsumerConfig, LogConsumer};
use toolshed::url::Url;

pub struct GraphSubscriptionsDatasource;

impl GraphSubscriptionsDatasource {
    async fn create(
        // Flag to determine if the implementer of the datasource instance needs to instantiate and consume from a Kafka Consumer
        instantiate_kafka_consumer: bool,
        kafka_broker: Option<Url>,
        kafka_subscription_logs_group_id: Option<String>,
        kafka_subscription_logs_topic_id: Option<String>,
    ) -> Result<()> {
        if instantiate_kafka_consumer {
            ensure!(
                kafka_broker.is_some(),
                "Must provide a Kafka Broker URL if instantiating the consumer instance"
            );
            ensure!(kafka_subscription_logs_group_id.is_some(), "Must provide a Subscription Query Logs Group ID if instantiating the consumer instance");
            ensure!(kafka_subscription_logs_topic_id.is_some(), "Must provide a Subscription Query Logs Topic ID if instantiating the consumer instance");
            // instantiate the consumer instance
            let _log_consumer = LogConsumer::create(ConsumerConfig {
                brokers: kafka_broker.unwrap(),
                group_id: kafka_subscription_logs_group_id.unwrap(),
                topic_id: kafka_subscription_logs_topic_id.unwrap(),
            });
            // TODO: buildout dynamic message consumer
        }

        Ok(())
    }

    pub async fn create_with_consumer(
        kafka_broker: Option<Url>,
        kafka_subscription_logs_group_id: Option<String>,
        kafka_subscription_logs_topic_id: Option<String>,
    ) -> Result<()> {
        GraphSubscriptionsDatasource::create(
            true,
            kafka_broker,
            kafka_subscription_logs_group_id,
            kafka_subscription_logs_topic_id,
        )
        .await
    }
}
