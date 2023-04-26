use std::collections::BTreeMap;

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

pub struct CreateWithDatasourcePgArgs {
    /// Kafka broker url.
    /// Format: {kafka_protocol}://{ip address}:{port}
    ///
    /// # Examples
    ///
    /// ```
    /// let kafka_broker = String::from("PLAINTEXT://127.0.0.1:9092");
    /// ```
    pub kafka_broker: String,
    /// The graph gateway subscription logs group id
    pub kafka_subscription_logs_group_id: String,
    /// The graph gateway subscription logs topic id
    pub kafka_subscription_logs_topic_id: String,
    /// Additional connection/configuration parameters.
    /// For example, to connect via SSL to the kafka broker, etc.
    ///
    /// # Examples
    ///
    /// ```
    /// // instantiate the additional config to connect via ssl
    /// let mut additional_config = BTreeMap::<String, String>::new();
    /// additiona_config.insert("security.protocol".to_string(), "sasl_ssl".to_string());
    /// additiona_config.insert("sasl.mechanism".to_string(), "SCRAM-SHA-256".to_string());
    /// additiona_config.insert("sasl.username".to_string(), "username".to_string());
    /// additiona_config.insert("sasl.password".to_string(), "password".to_string());
    /// additiona_config.insert("ssl.ca.location".to_string(), "/path/to/ca/cert".to_string());
    /// additiona_config.insert("ssl.certificate.location".to_string(), "/path/to/ssl/cert".to_string());
    /// additiona_config.insert("ssl.key.location".to_string(), "/path/to/ssl/key".to_string());
    /// ```
    pub kafka_additional_config: Option<BTreeMap<String, String>>,
    /// Postgres db url.
    /// Format: `postgres://{user}:{password}@{host}:{port}/{database}
    ///
    /// # Examples
    ///
    /// ```
    /// let postgres_db_url = String::from("postgres://dev:password1@0.0.0.0:5432/logs");
    /// ```
    pub postgres_db_url: String,
    /// Number of work threads to spin up which listen on the kafka message consumer and write to the db.
    /// Default value is: 1
    pub num_workers: Option<usize>,
}

impl<T: Datasource> GraphSubscriptionsDatasource<'_, T> {
    pub async fn create_with_datasource_pg(
        args: CreateWithDatasourcePgArgs,
    ) -> Result<GraphSubscriptionsDatasource<'static, DatasourcePostgres>> {
        // instantiate the consumer instance
        let log_consumer = LogConsumer::create(ConsumerConfig {
            brokers: args.kafka_broker,
            group_id: args.kafka_subscription_logs_group_id,
            topic_id: args.kafka_subscription_logs_topic_id,
            additional_config: None,
        })?;
        // instantiate the postgres datasource instance and begin consuming messages
        let datasource_pg = DatasourcePostgres::create(args.postgres_db_url).await?;

        for _ in 0..args.num_workers.unwrap_or(1) {
            tokio::spawn(datasource_pg.write(&log_consumer.consumer));
        }

        Ok(GraphSubscriptionsDatasource::<DatasourcePostgres> {
            datasource: datasource_pg,
        })
    }
}
