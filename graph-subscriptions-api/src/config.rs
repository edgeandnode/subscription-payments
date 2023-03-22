use dotenv::dotenv;

#[derive(Debug)]
pub struct Config {
    /// Port to run the graph-subscriptions-api on. default: 4000
    pub api_port: u16,
    /// Graphql api/graphiql endpoint. default: /graphql
    pub graphql_endpoint: String,
    /// Format log output as JSON
    pub log_json: bool,
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

    Config {
        api_port,
        graphql_endpoint,
        log_json,
    }
}
