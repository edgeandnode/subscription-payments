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
    /// The graph gateway subscription logs topic id
    pub kafka_topic_id: String,
    /// Connection/configuration parameters.
    /// For example, to connect via SSL to the kafka broker, etc.
    /// Thesese key-values are _required_:
    /// - "bootstrap.servers"
    /// - "group.id"
    ///
    /// # Examples
    ///
    /// ```
    /// // instantiate the config, to connect locally, no auth mech
    /// let mut config = BTreeMap::<String, String>::new();
    /// config.insert("bootstrap.servers".to_string(), "PLAINTEXT://127.0.0.1:9092".to_string());
    /// config.insert("group.id".to_string(), "graph-gateway".to_string());
    /// config.insert("message.timeout.ms".to_string(), "3000".to_string());
    /// config.insert("queue.buffering.max.ms".to_string(), "1000".to_string());
    /// config.insert("queue.buffering.max.messages".to_string(), "100000".to_string());
    /// config.insert("enable.partition.eof".to_string(), "false".to_string());
    /// config.insert("enable.auto.commit".to_string(), "false".to_string());
    /// // instantiate the config to connect via ssl
    /// let mut config = BTreeMap::<String, String>::new();
    /// config.insert("bootstrap.servers".to_string(), "PLAINTEXT://127.0.0.1:9092".to_string());
    /// config.insert("group.id".to_string(), "graph-gateway".to_string());
    /// config.insert("security.protocol".to_string(), "sasl_ssl".to_string());
    /// config.insert("sasl.mechanism".to_string(), "SCRAM-SHA-256".to_string());
    /// config.insert("sasl.username".to_string(), "username".to_string());
    /// config.insert("sasl.password".to_string(), "password".to_string());
    /// config.insert("ssl.ca.location".to_string(), "/path/to/ca/cert".to_string());
    /// config.insert("ssl.certificate.location".to_string(), "/path/to/ssl/cert".to_string());
    /// config.insert("ssl.key.location".to_string(), "/path/to/ssl/key".to_string());
    /// config.insert("message.timeout.ms".to_string(), "3000".to_string());
    /// config.insert("queue.buffering.max.ms".to_string(), "1000".to_string());
    /// config.insert("queue.buffering.max.messages".to_string(), "100000".to_string());
    /// config.insert("enable.partition.eof".to_string(), "false".to_string());
    /// config.insert("enable.auto.commit".to_string(), "false".to_string());
    /// ```
    pub kafka_config: BTreeMap<String, String>,
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
            topic_id: args.kafka_topic_id,
            config: args.kafka_config,
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
