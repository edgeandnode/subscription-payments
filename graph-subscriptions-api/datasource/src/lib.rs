use anyhow::{Ok, Result};

mod consumer;
mod datasource;
mod datsource_postgres;
mod models;
mod utils;

pub use datasource::{Datasource, DatasourceWriter};
pub use datsource_postgres::DatasourcePostgres;
pub use models::*;

use consumer::{ConsumerConfig, LogConsumer};

pub struct GraphSubscriptionsDatasource<'a, T>
where
    T: Datasource,
{
    pub datasource: &'a T,
}

impl<T: Datasource> GraphSubscriptionsDatasource<'_, T> {
    pub async fn create_with_datasource_pg(
        kafka_broker: String,
        kafka_subscription_logs_group_id: String,
        kafka_subscription_logs_topic_id: String,
        postgres_db_url: String,
        num_workers: Option<usize>,
    ) -> Result<GraphSubscriptionsDatasource<'static, DatasourcePostgres>> {
        // instantiate the consumer instance
        let log_consumer = LogConsumer::create(ConsumerConfig {
            brokers: kafka_broker,
            group_id: kafka_subscription_logs_group_id,
            topic_id: kafka_subscription_logs_topic_id,
        })?;
        // instantiate the postgres datasource instance and begin consuming messages
        let datasource_pg = DatasourcePostgres::create(postgres_db_url).await?;

        for _ in 0..num_workers.unwrap_or(1) {
            tokio::spawn(datasource_pg.write(&log_consumer.consumer));
        }

        Ok(GraphSubscriptionsDatasource::<DatasourcePostgres> {
            datasource: datasource_pg,
        })
    }
}
