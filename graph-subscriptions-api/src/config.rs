use std::str::FromStr;

use dotenv::dotenv;
use toolshed::bytes::Address;

#[derive(Debug)]
pub struct Config {
    /// Port to run the graph-subscriptions-api on. default: 4000
    pub api_port: u16,
    /// Graphql api/graphiql endpoint. default: /graphql
    pub graphql_endpoint: String,
    /// Format log output as JSON
    pub log_json: bool,
    /// Subscriptions contract chain ID
    pub subscriptions_chain_id: u64,
    /// Subscriptions contract address
    pub subscriptions_contract_address: Address,
}

pub fn init_config() -> Config {
    dotenv().ok();

    let api_port: u16 = dotenv::var("API_PORT")
        .unwrap_or(String::from("4000"))
        .parse()
        .unwrap();
    let graphql_endpoint = dotenv::var("GRAPHQL_ENDPOINT").unwrap_or(String::from("/graphql"));
    let log_json: bool = dotenv::var("LOG_JSON")
        .unwrap_or(String::from("true"))
        .parse()
        .unwrap();
    let subscriptions_chain_id: u64 = match dotenv::var("SUBSCRIPTIONS_CONTRACT_CHAIN_ID") {
        Ok(chain_id) => chain_id.parse().unwrap(),
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

    Config {
        api_port,
        graphql_endpoint,
        log_json,
        subscriptions_chain_id,
        subscriptions_contract_address,
    }
}
