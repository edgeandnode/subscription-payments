use std::{collections::BTreeMap, str::FromStr};

use dotenv::dotenv;
use ethers::types::U256;
use toolshed::{bytes::Address, url::Url};

#[derive(Debug)]
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
    pub network_subgraph_url: Url,
    /// Subscriptions contract chain ID
    pub subscriptions_chain_id: U256,
    /// Subscriptions contract address
    pub subscriptions_contract_address: Address,
    /// Subscriptions subgraph url
    pub subscriptions_subgraph_url: Url,
    /// Graph Subscription logs Kafka consumer broker url
    pub graph_subscription_logs_kafka_broker: String,
    /// Graph Subscription logs Kafka group ID
    pub graph_subscription_logs_kafka_group_id: String,
    /// Graph Subscription logs Kafka topic ID
    pub graph_subscription_logs_kafka_topic_id: String,
    /// Additional kafka configuration settings
    pub graph_subscription_logs_kafka_additional_config: Option<BTreeMap<String, String>>,
    /// Postgres database url where the logs are stored.
    /// Uses format: "postgres://{user}:{pwd}@{host}:{port}/{database}"
    pub graph_subscription_logs_db_url: String,
}

pub fn init_config() -> Config {
    dotenv().ok();

    let api_port: u16 = dotenv::var("API_PORT")
        .unwrap_or(String::from("4000"))
        .parse()
        .unwrap();
    let graphql_endpoint = dotenv::var("GRAPHQL_ENDPOINT").unwrap_or(String::from("/graphql"));
    let metrics_port: u16 = dotenv::var("METRICS_PORT")
        .unwrap_or(String::from("9090"))
        .parse()
        .unwrap();
    let log_json: bool = dotenv::var("LOG_JSON")
        .unwrap_or(String::from("true"))
        .parse()
        .unwrap();
    let subscriptions_chain_id: U256 = match dotenv::var("SUBSCRIPTIONS_CONTRACT_CHAIN_ID") {
        Ok(chain_id) => U256::from_dec_str(&chain_id).unwrap_or(U256::from(421613)),
        Err(_) => panic!("SUBSCRIPTIONS_CONTRACT_CHAIN_ID environment variable is required"),
    };
    let subscriptions_contract_address: Address =
        match dotenv::var("SUBSCRIPTIONS_CONTRACT_ADDRESS") {
            Ok(addr) => match Address::from_str(addr.as_str()) {
                Ok(contract_addr) => contract_addr,
                Err(_) => panic!("SUBSCRIPTIONS_CONTRACT_ADDRESS environment variable is invalid"),
            },
            Err(_) => panic!("SUBSCRIPTIONS_CONTRACT_ADDRESS environment variable is required"),
        };
    let subscriptions_subgraph_url: Url = match dotenv::var("SUBSCRIPTIONS_SUBGRAPH_URL") {
        Ok(url) => match Url::from_str(url.as_str()) {
            Ok(url) => url,
            Err(_) => panic!("SUBSCRIPTIONS_SUBGRAPH_URL environment variable is invalid"),
        },
        Err(_) => panic!("SUBSCRIPTIONS_SUBGRAPH_URL environment variable is required"),
    };
    let network_subgraph_url: Url = match dotenv::var("NETWORK_SUBGRAPH_URL") {
        Ok(url) => match Url::from_str(url.as_str()) {
            Ok(url) => url,
            Err(_) => panic!("NETWORK_SUBGRAPH_URL environment variable is invalid"),
        },
        Err(_) => panic!("NETWORK_SUBGRAPH_URL environment variable is required"),
    };
    let graph_subscription_logs_kafka_broker =
        match dotenv::var("GRAPH_SUBSCRIPTION_LOGS_KAFKA_BROKER") {
            Ok(url) => url,
            Err(_) => {
                panic!("GRAPH_SUBSCRIPTION_LOGS_KAFKA_BROKER environment variable is required")
            }
        };
    let graph_subscription_logs_kafka_group_id =
        match dotenv::var("GRAPH_SUBSCRIPTION_LOGS_KAFKA_GROUP_ID") {
            Ok(group_id) => group_id,
            Err(_) => {
                panic!("GRAPH_SUBSCRIPTION_LOGS_KAFKA_GROUP_ID environment variable is required")
            }
        };
    let graph_subscription_logs_kafka_topic_id =
        match dotenv::var("GRAPH_SUBSCRIPTION_LOGS_KAFKA_TOPIC_ID") {
            Ok(group_id) => group_id,
            Err(_) => {
                panic!("GRAPH_SUBSCRIPTION_LOGS_KAFKA_TOPIC_ID environment variable is required")
            }
        };
    // attempt to parse and build the additional kafka config
    let mut kafka_additional_config = BTreeMap::<String, String>::new();
    if let Some(security_protocol) =
        dotenv::var("GRAPH_SUBSCRIPTION_LOGS_KAFKA_SECURITY_PROTOCOL").ok()
    {
        kafka_additional_config.insert("security.protocol".to_string(), security_protocol);
    }
    if let Some(mechanism) = dotenv::var("GRAPH_SUBSCRIPTION_LOGS_KAFKA_SASL_MECHANISM").ok() {
        kafka_additional_config.insert("sasl.mechanism".to_string(), mechanism);
    }
    if let Some(username) = dotenv::var("GRAPH_SUBSCRIPTION_LOGS_KAFKA_SASL_USERNAME").ok() {
        kafka_additional_config.insert("sasl.username".to_string(), username);
    }
    if let Some(password) = dotenv::var("GRAPH_SUBSCRIPTION_LOGS_KAFKA_SASL_PASSWORD").ok() {
        kafka_additional_config.insert("sasl.password".to_string(), password);
    }
    if let Some(ssl_ca_loc) = dotenv::var("GRAPH_SUBSCRIPTION_LOGS_KAFKA_SSL_CA_LOC").ok() {
        kafka_additional_config.insert("ssl.ca.location".to_string(), ssl_ca_loc);
    }
    if let Some(ssl_cert_loc) = dotenv::var("GRAPH_SUBSCRIPTION_LOGS_KAFKA_SSL_CERT_LOC").ok() {
        kafka_additional_config.insert("ssl.certificate.location".to_string(), ssl_cert_loc);
    }
    if let Some(ssl_key_loc) = dotenv::var("GRAPH_SUBSCRIPTION_LOGS_KAFKA_SSL_KEY_LOC").ok() {
        kafka_additional_config.insert("ssl.key.location".to_string(), ssl_key_loc);
    }

    let graph_subscription_logs_db_url = match dotenv::var("GRAPH_SUBSCRIPTION_LOGS_DB_URL") {
        Ok(url) => url,
        Err(_) => panic!("GRAPH_SUBSCRIPTION_LOGS_DB_URL environment variable is required"),
    };

    Config {
        api_port,
        graphql_endpoint,
        metrics_port,
        log_json,
        subscriptions_chain_id,
        subscriptions_contract_address,
        subscriptions_subgraph_url,
        network_subgraph_url,
        graph_subscription_logs_kafka_broker,
        graph_subscription_logs_kafka_group_id,
        graph_subscription_logs_kafka_topic_id,
        graph_subscription_logs_kafka_additional_config: Some(kafka_additional_config),
        graph_subscription_logs_db_url,
    }
}
