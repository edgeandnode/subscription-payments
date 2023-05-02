use std::collections::BTreeMap;

use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};
use toolshed::{bytes::Address, url::Url};

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Port to run the graph-subscriptions-api on. default: 4000
    pub api_port: u16,
    /// Graphql api/graphiql endpoint. default: /graphql
    pub graphql_endpoint: String,
    /// Port to run the graph-subscriptions-api _metrics_ endpoint on. default: 9090
    pub metrics_port: u16,
    /// Format log output as JSON
    pub log_json: bool,
    /// The Graph Network Subgraph URL. For querying subgraphs published to the network
    #[serde_as(as = "DisplayFromStr")]
    pub network_subgraph_url: Url,
    /// Subscriptions contract chain ID
    pub subscriptions_chain_id: u64,
    /// Subscriptions contract address
    pub subscriptions_contract_address: Address,
    /// Subscriptions subgraph url
    #[serde_as(as = "DisplayFromStr")]
    pub subscriptions_subgraph_url: Url,
    /// See https://github.com/confluentinc/librdkafka/blob/master/CONFIGURATION.md
    ///
    /// # Examples
    ///
    ///
    /// ```
    /// // builds default, basic auth settings for connecting to a local kafka instance
    /// {
    ///     "kafka": {
    ///         "bootstrap.servers": "PLAINTEXT://127.0.0.1:9092",
    ///         "group.id": "graph-gateway",
    ///         "message.timeout.ms": "3000",
    ///         "queue.buffering.max.ms": "1000",
    ///         "queue.buffering.max.messages": "100000",
    ///         "enable.partition.eof": "false",
    ///         "enable.auto.commit": "false",
    ///     }
    /// }
    /// // with SSL/SASL authentication mechanism configured as well
    /// {
    ///     "kafka": {
    ///         "bootstrap.servers": "PLAINTEXT://127.0.0.1:9092",
    ///         "security.protocol": "sasl_ssl",
    ///         "sasl.mechanism": "SCRAM-SHA-256",
    ///         "sasl.username": "username",
    ///         "sasl.password": "pwd",
    ///         "ssl.ca.location": "/path/to/ca.crt",
    ///         "ssl.certificate.location": "/path/to/ssl.crt",
    ///         "ssl.key.location": "/path/to/ssl.key",
    ///         "group.id": "graph-gateway",
    ///         "message.timeout.ms": "3000",
    ///         "queue.buffering.max.ms": "1000",
    ///         "queue.buffering.max.messages": "100000",
    ///         "enable.partition.eof": "false",
    ///         "enable.auto.commit": "false",
    ///     }
    /// }
    /// ```
    #[serde(default)]
    pub kafka: KafkaConfig,
    /// The Kafka topic the gateway GSP query logs will be published to
    pub kafka_topic_id: String,
    /// Postgres database url where the logs are stored.
    /// Uses format: "postgres://{user}:{pwd}@{host}:{port}/{database}"
    pub db_url: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct KafkaConfig(BTreeMap<String, String>);

impl KafkaConfig {
    pub fn build(mut conf: KafkaConfig) -> BTreeMap<String, String> {
        let mut settings = conf.0.clone();
        settings.append(&mut conf.0);

        settings
    }
}

impl Default for KafkaConfig {
    fn default() -> Self {
        let settings = [
            ("bootstrap.servers", "PLAINTEXT://127.0.0.1:9092"),
            ("group.id", "graph-gateway"),
            ("message.timeout.ms", "3000"),
            ("queue.buffering.max.ms", "1000"),
            ("queue.buffering.max.messages", "100000"),
            ("enable.partition.eof", "false"),
            ("enable.auto.commit", "false"),
        ];
        Self(
            settings
                .into_iter()
                .map(|(k, v)| (k.to_owned(), v.to_owned()))
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_json_config_into_config_instance() {
        let config_raw = r#"
        {
            "api_port": 4000,
            "graphql_endpoint": "/graphql",
            "metrics_port": 9090,
            "log_json": true,
            "network_subgraph_url": "https://api.thegraph.com/subgraphs/name/graphprotocol/graph-network-mainnet/graphql",
            "subscriptions_chain_id": 421613,
            "subscriptions_contract_address": "0x29f49a438c747e7Dd1bfe7926b03783E47f9447B",
            "subscriptions_subgraph_url": "https://api.thegraph.com/subgraphs/name/graphprotocol/subscriptions-arbitrum-goerli",
            "kafka": {
                "bootstrap.servers": "PLAINTEXT://127.0.0.1:9092",
                "group.id": "graph-gateway",
                "message.timeout.ms": "3000",
                "queue.buffering.max.ms": "1000",
                "queue.buffering.max.messages": "100000",
                "enable.partition.eof": "false",
                "enable.auto.commit": "false"
            },
            "kafka_topic_id": "gateway_subscription_query_results",
            "db_url": "postgres://dev:dev@localhost:5432/gateway_subscription_query_results"
        }
        "#;
        let expected_kafka_config = KafkaConfig::default();
        let expected = Config {
            api_port: 4000,
            graphql_endpoint: "/graphql".to_string(),
            metrics_port: 9090,
            log_json: true,
            network_subgraph_url: "https://api.thegraph.com/subgraphs/name/graphprotocol/graph-network-mainnet/graphql".parse::<Url>().unwrap(),
            subscriptions_chain_id: 421613,
            subscriptions_contract_address: "0x29f49a438c747e7Dd1bfe7926b03783E47f9447B".parse::<Address>().unwrap(),
            subscriptions_subgraph_url: "https://api.thegraph.com/subgraphs/name/graphprotocol/subscriptions-arbitrum-goerli".parse::<Url>().unwrap(),
            kafka_topic_id: "gateway_subscription_query_results".to_string(),
            db_url: "postgres://dev:dev@localhost:5432/gateway_subscription_query_results".to_string(),
            kafka: expected_kafka_config
        };

        match serde_json::from_str::<Config>(&config_raw) {
            Ok(actual) => {
                // spot check
                assert_eq!(actual.api_port, expected.api_port);
                assert_eq!(actual.kafka_topic_id, expected.kafka_topic_id);
                assert_eq!(
                    actual.subscriptions_chain_id,
                    expected.subscriptions_chain_id
                );
                assert_eq!(actual.db_url, expected.db_url);
                assert_eq!(actual.kafka.clone(), expected.kafka.clone());
            }
            Err(err) => {
                assert!(false, "Failure parsing JSON -> Config {:#?}", err);
            }
        }
    }
}
