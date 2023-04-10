use anyhow::{Ok, Result};
use futures::{stream::FuturesUnordered, StreamExt};
use toolshed::url::Url;

mod consumer;
mod datasource;
mod datasource_redis;
mod models;
mod utils;

pub use datasource::{Datasource, DatasourceWriter};
pub use datasource_redis::DatasourceRedis;
pub use models::*;

use consumer::{ConsumerConfig, LogConsumer};

pub struct GraphSubscriptionsDatasource<'a, T>
where
    T: Datasource,
{
    pub datasource: &'a T,
}

impl<T: Datasource> GraphSubscriptionsDatasource<'_, T> {
    pub async fn create_with_datasource_redis(
        kafka_broker: Url,
        kafka_subscription_logs_group_id: String,
        kafka_subscription_logs_topic_id: String,
        redis_url: String,
        num_workers: Option<usize>,
    ) -> Result<GraphSubscriptionsDatasource<'static, DatasourceRedis>> {
        // instantiate the consumer instance
        let log_consumer = LogConsumer::create(ConsumerConfig {
            brokers: kafka_broker,
            group_id: kafka_subscription_logs_group_id,
            topic_id: kafka_subscription_logs_topic_id,
        })?;
        // instantiate the redis datasource instance and begin consuming messages
        let datasource_redis = DatasourceRedis::create(&redis_url);

        (0..num_workers.unwrap_or(1))
            .map(|_| tokio::spawn(datasource_redis.write(&log_consumer.consumer)))
            .collect::<FuturesUnordered<_>>()
            .for_each(|_| async { () })
            .await;

        Ok(GraphSubscriptionsDatasource {
            datasource: datasource_redis,
        })
    }
}
