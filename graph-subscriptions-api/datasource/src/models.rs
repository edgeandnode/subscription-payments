use toolshed::bytes::{Address, DeploymentId};

#[derive(Debug, Clone, PartialEq)]
/// Stats pulled and aggergated/derived from queries made from users using The Graph Subscriptions; over all queried Subgraphs.
/// The logs are pushed onto a kafka topic by The Graph Gateway.
pub struct RequestTicketStat {
    pub id: String,
    /// User-selected, friendly name of the request ticket. Part of the EIP-712 domain signed message.
    /// Pulled directly from the kafka topic log from The Graph Gateway.
    pub ticket_name: String,
    /// Wallet address of the user who owns the request ticket.
    /// Pulled directly from the kafka topic log from The Graph Gateway.
    pub ticket_user: Address,
    /// Wallet address of the signer of the request ticket.
    /// This value will often be the `ticket_user`, but could also be an authorized signer for the subscription owner.
    /// Pulled directly from the kafka topic log from The Graph Gateway.
    pub ticket_signer: Address,
    /// An aggregate count of queries performed using the request ticket in the given timeframe.
    /// SUM(`query_count` (from kafka topic)).
    pub query_count: u32,
    /// Percentage of the number of queries that returned successfully compared to the total query count in the given timeframe.
    /// `query_count` WHERE `status_code` == SUCCESS / `query_count`.
    pub success_rate: f32,
    /// An aggregate average of the response time (in ms) of the query responses in the given timeframe.
    /// sum of `response_time_ms` (pulled from kafka topic) / timeframe.
    pub avg_response_time_ms: u32,
    /// An aggregate count of queries performed in the timeframe that were not successful.
    /// SUM(`query_count` (from kafka topic)) WHERE `status_code` != SUCCESS.
    pub failed_query_count: u32,
}

#[derive(Debug, Clone, PartialEq)]
/// Stats pulled and aggergated/derived from queries made from users using The Graph Subscriptions; for a specific Subgraph Deployment.
/// The logs are pushed onto a kafka topic by The Graph Gateway.
pub struct RequestTicketSubgraphStat {
    pub id: String,
    /// Qm Hash of the Subgraph Deployment that was queried using the request ticket.
    /// Pulled directly from the kafka topic log from The Graph Gateway.
    pub subgraph_deployment_qm_hash: DeploymentId,
    /// User-selected, friendly name of the request ticket. Part of the EIP-712 domain signed message.
    /// Pulled directly from the kafka topic log from The Graph Gateway.
    pub ticket_name: String,
    /// Wallet address of the user who owns the request ticket.
    /// Pulled directly from the kafka topic log from The Graph Gateway.
    pub ticket_user: Address,
    /// Wallet address of the signer of the request ticket.
    /// This value will often be the `ticket_user`, but could also be an authorized signer for the subscription owner.
    /// Pulled directly from the kafka topic log from The Graph Gateway.
    pub ticket_signer: Address,
    /// An aggregate count of queries performed using the request ticket in the given timeframe.
    /// SUM(`query_count` (from kafka topic)).
    pub query_count: u32,
    /// Percentage of the number of queries that returned successfully compared to the total query count in the given timeframe.
    /// `query_count` WHERE `status_code` == SUCCESS / `query_count`.
    pub success_rate: f32,
    /// An aggregate average of the response time (in ms) of the query responses in the given timeframe.
    /// sum of `response_time_ms` (pulled from kafka topic) / timeframe.
    pub avg_response_time_ms: u32,
    /// An aggregate count of queries performed in the timeframe that were not successful.
    /// SUM(`query_count` (from kafka topic)) WHERE `status_code` != SUCCESS.
    pub failed_query_count: u32,
}
