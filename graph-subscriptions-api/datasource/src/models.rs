use std::io::Cursor;

use graph_subscriptions::TicketPayload;
use prost::Message;
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};
use toolshed::bytes::{Address, DeploymentId};

#[derive(Clone, Deserialize, PartialEq, ::prost::Message)]
pub struct GatewaySubscriptionQueryResult {
    /// Set to the value of the CF-Ray header, otherwise a generated UUID
    #[prost(string, tag = "1")]
    pub query_id: ::prost::alloc::string::String,
    #[prost(enumeration = "StatusCode", tag = "2")]
    pub status_code: i32,
    #[prost(string, tag = "3")]
    pub status_message: ::prost::alloc::string::String,
    #[prost(uint32, tag = "4")]
    pub response_time_ms: u32,
    /// `user` field from ticket payload, 0x-prefixed hex
    #[prost(string, tag = "5")]
    pub ticket_user: ::prost::alloc::string::String,
    /// the ticket payload, JSON map data
    #[prost(string, tag = "6")]
    pub ticket_payload: ::prost::alloc::string::String,
    /// Subgraph Deployment ID, CIDv0 ("Qm" hash)
    #[prost(string, optional, tag = "7")]
    pub deployment: ::core::option::Option<::prost::alloc::string::String>,
    /// Chain name indexed by subgraph deployment
    #[prost(string, optional, tag = "8")]
    pub subgraph_chain: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(uint32, optional, tag = "9")]
    pub query_count: ::core::option::Option<u32>,
    #[prost(float, optional, tag = "10")]
    pub query_budget: ::core::option::Option<f32>,
    #[prost(float, optional, tag = "11")]
    pub indexer_fees: ::core::option::Option<f32>,
}
#[derive(
    Clone, Copy, Debug, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration,
)]
#[repr(i32)]
pub enum StatusCode {
    Success = 0,
    InternalError = 1,
    UserError = 2,
    NotFound = 3,
}
impl StatusCode {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            StatusCode::Success => "SUCCESS",
            StatusCode::InternalError => "INTERNAL_ERROR",
            StatusCode::UserError => "USER_ERROR",
            StatusCode::NotFound => "NOT_FOUND",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "SUCCESS" => Some(Self::Success),
            "INTERNAL_ERROR" => Some(Self::InternalError),
            "USER_ERROR" => Some(Self::UserError),
            "NOT_FOUND" => Some(Self::NotFound),
            _ => None,
        }
    }
}
impl GatewaySubscriptionQueryResult {
    pub fn from_slice(slice: &[u8]) -> anyhow::Result<Self> {
        Self::decode(&mut Cursor::new(slice)).map_err(|err| anyhow::Error::from(err))
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum OrderDirection {
    Asc,
    Desc,
}
impl OrderDirection {
    pub fn as_str(&self) -> &'static str {
        match self {
            OrderDirection::Asc => "ASC",
            OrderDirection::Desc => "DESC",
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RequestTicket {
    /// User-selected, friendly name of the request ticket. Part of the EIP-712 domain signed message.
    /// Parsed from the `ticket_payload` JSON object from the kafka topic.
    pub ticket_name: String,
    /// Wallet address of the user who owns the request ticket.
    /// Pulled directly from the kafka topic log from The Graph Gateway.
    pub ticket_user: Address,
    /// Total count of queries performed, across all deployments, by the request ticket
    pub total_query_count: i64,
    /// Count of _unique_ deployments queried by the request ticket
    pub queried_subgraphs_count: i64,
    /// Unix-timestamp of when the latest query was process, for any deployment, by the request ticket
    pub last_query_timestamp: i64,
    /// The ticket payload value with the signer, allowed_domains, allowed_deployments, allowed_subgraphs, etc
    pub ticket_payload: TicketPayload,
}

#[derive(Debug, PartialEq)]
pub struct UniqRequestTicketDeploymentQmHash {
    pub deployment_qm_hash: DeploymentId,
}

#[derive(Debug, PartialEq, FromQueryResult)]
pub struct UserHasTicketResult {
    pub user_has_ticket: bool,
}
impl Default for UserHasTicketResult {
    fn default() -> Self {
        Self {
            user_has_ticket: false,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RequestTicketOrderBy {
    Owner,
    Signer,
    Name,
}
impl RequestTicketOrderBy {
    pub fn as_str(&self) -> &'static str {
        match self {
            RequestTicketOrderBy::Name => "ticket_name",
            RequestTicketOrderBy::Owner => "ticket_user",
            RequestTicketOrderBy::Signer => "ticker_signer",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
/// Stats pulled and aggergated/derived from queries made from users using The Graph Subscriptions; over all queried Subgraphs.
/// The logs are pushed onto a kafka topic by The Graph Gateway.
pub struct RequestTicketStat {
    /// User-selected, friendly name of the request ticket. Part of the EIP-712 domain signed message.
    /// Pulled directly from the kafka topic log from The Graph Gateway.
    pub ticket_name: String,
    /// Wallet address of the user who owns the request ticket.
    /// Pulled directly from the kafka topic log from The Graph Gateway.
    pub ticket_user: Address,
    /// Timestamp start of the timeframe the stat record aggregates over.
    pub start: i64,
    /// Timestamp end of the timeframe the stat record aggregates over.
    pub end: i64,
    /// An aggregate count of queries performed using the request ticket in the given timeframe.
    /// SUM(`query_count` (from kafka topic)).
    pub query_count: i64,
    /// Percentage of the number of queries that returned successfully compared to the total query count in the given timeframe.
    /// `query_count` WHERE `status_code` == SUCCESS / `query_count`.
    pub success_rate: f32,
    /// An aggregate average of the response time (in ms) of the query responses in the given timeframe.
    /// sum of `response_time_ms` (pulled from kafka topic) / timeframe.
    pub avg_response_time_ms: i32,
    /// An aggregate count of queries performed in the timeframe that were not successful.
    /// SUM(`query_count` (from kafka topic)) WHERE `status_code` != SUCCESS.
    pub failed_query_count: i64,
    /// Count of _unique_ deployments queried by the request ticket
    pub queried_subgraphs_count: i64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RequestTicketStatOrderBy {
    Start,
    End,
    TotalQueryCount,
}
impl RequestTicketStatOrderBy {
    pub fn as_str(&self) -> &'static str {
        match self {
            RequestTicketStatOrderBy::End => "timeframe_end_timestamp",
            RequestTicketStatOrderBy::Start => "timeframe_start_timestamp",
            RequestTicketStatOrderBy::TotalQueryCount => "query_count",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
/// Stats pulled and aggergated/derived from queries made from users using The Graph Subscriptions; for a specific Subgraph Deployment.
/// The logs are pushed onto a kafka topic by The Graph Gateway.
pub struct RequestTicketSubgraphStat {
    /// Qm Hash of the Subgraph Deployment that was queried using the request ticket.
    /// Pulled directly from the kafka topic log from The Graph Gateway.
    pub subgraph_deployment_qm_hash: DeploymentId,
    /// User-selected, friendly name of the request ticket. Part of the EIP-712 domain signed message.
    /// Pulled directly from the kafka topic log from The Graph Gateway.
    pub ticket_name: String,
    /// Wallet address of the user who owns the request ticket.
    /// Pulled directly from the kafka topic log from The Graph Gateway.
    pub ticket_user: Address,
    /// Timestamp start of the timeframe the stat record aggregates over.
    pub start: i64,
    /// Timestamp end of the timeframe the stat record aggregates over.
    pub end: i64,
    /// An aggregate count of queries performed using the request ticket in the given timeframe.
    /// SUM(`query_count` (from kafka topic)).
    pub query_count: i64,
    /// Percentage of the number of queries that returned successfully compared to the total query count in the given timeframe.
    /// `query_count` WHERE `status_code` == SUCCESS / `query_count`.
    pub success_rate: f32,
    /// An aggregate average of the response time (in ms) of the query responses in the given timeframe.
    /// sum of `response_time_ms` (pulled from kafka topic) / timeframe.
    pub avg_response_time_ms: i32,
    /// An aggregate count of queries performed in the timeframe that were not successful.
    /// SUM(`query_count` (from kafka topic)) WHERE `status_code` != SUCCESS.
    pub failed_query_count: i64,
}

#[cfg(test)]
mod tests {
    use prost::Message;
    use serde_json::json;

    use super::*;

    #[test]
    fn should_decode_slice_to_gateway_subscription_query_result() {
        let result = GatewaySubscriptionQueryResult {
            query_id: String::from("1"),
            status_code: 0,
            status_message: String::from("success"),
            response_time_ms: 300,
            ticket_user: String::from("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"),
            ticket_payload: json!({
                "signer": String::from("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"),
                "name": String::from("test_req_ticket__1"),
            })
            .to_string(),
            deployment: Some(String::from(
                "Qmadj8x9km1YEyKmRnJ6EkC2zpJZFCfTyTZpuqC3j6e1QH",
            )),
            subgraph_chain: Some(String::from("mainnet")),
            query_count: Some(100),
            query_budget: Some(0.0003),
            indexer_fees: Some(0.00015),
        };
        // encode as byte vector
        let mut encoded_buf = Vec::new();
        encoded_buf.reserve(result.encoded_len());
        result.encode(&mut encoded_buf).unwrap();
        // convert byte vector -> slice
        let buf = encoded_buf.as_slice();
        // decode back to `GatewaySubscriptionQueryResult`
        let actual = GatewaySubscriptionQueryResult::from_slice(buf);

        assert!(actual.is_ok());
        assert_eq!(actual.unwrap(), result);
    }

    #[test]
    fn should_deserialize_str_to_ticket_payload_no_allowed_fields() {
        let given = r#"{"signer":[193,66,188,240,64,171,249,55,3,192,61,172,240,44,84,180,13,160,237,235],"user":[193,66,188,240,64,171,249,55,3,192,61,172,240,44,84,180,13,160,237,235],"name":"query_key__arb-goerli"}"#;
        let expected = TicketPayload {
            name: Some(String::from("query_key__arb-goerli")),
            signer: "0xc142bcf040AbF93703c03DaCf02c54B40dA0eDEb"
                .parse::<ethers::types::Address>()
                .unwrap(),
            user: Some(
                "0xc142bcf040AbF93703c03DaCf02c54B40dA0eDEb"
                    .parse::<ethers::types::Address>()
                    .unwrap(),
            ),
            allowed_deployments: None,
            allowed_domains: None,
            allowed_subgraphs: None,
        };

        if let Ok(actual) = serde_json::from_str::<TicketPayload>(&given.to_string()) {
            assert_eq!(actual, expected);
        } else {
            assert!(false, "should not throw deserialize error")
        }
    }

    #[test]
    fn should_deserialize_str_to_ticket_payload_with_allowed_fields() {
        let given = json!({
            "name": "req_ticket__1",
            "signer": [193,66,188,240,64,171,249,55,3,192,61,172,240,44,84,180,13,160,237,235],
            "user": [193,66,188,240,64,171,249,55,3,192,61,172,240,44,84,180,13,160,237,235],
            "allowed_domains": "http://localhost:3000,*.thegraph.com",
            "allowed_deployments": "Qmaz1R8vcv9v3gUfksqiS9JUz7K9G8S5By3JYn8kTiiP5K,Qmadj8x9km1YEyKmRnJ6EkC2zpJZFCfTyTZpuqC3j6e1QH",
            "allowed_subgraphs": "3nXfK3RbFrj6mhkGdoKRowEEti2WvmUdxmz73tben6Mb,FDD65maya4xVfPnCjSgDRBz6UBWKAcmGtgY6BmUueJCg"
        });
        let expected = TicketPayload {
            name: Some(String::from("req_ticket__1")),
            signer: "0xc142bcf040AbF93703c03DaCf02c54B40dA0eDEb"
                .parse::<ethers::types::Address>()
                .unwrap(),
            user: Some(
                "0xc142bcf040AbF93703c03DaCf02c54B40dA0eDEb"
                    .parse::<ethers::types::Address>()
                    .unwrap(),
            ),
            allowed_domains: Some(String::from("http://localhost:3000,*.thegraph.com")),
            allowed_deployments: Some(String::from("Qmaz1R8vcv9v3gUfksqiS9JUz7K9G8S5By3JYn8kTiiP5K,Qmadj8x9km1YEyKmRnJ6EkC2zpJZFCfTyTZpuqC3j6e1QH")),
            allowed_subgraphs: Some(String::from("3nXfK3RbFrj6mhkGdoKRowEEti2WvmUdxmz73tben6Mb,FDD65maya4xVfPnCjSgDRBz6UBWKAcmGtgY6BmUueJCg")),
        };

        if let Ok(actual) = serde_json::from_str::<TicketPayload>(&given.to_string()) {
            assert_eq!(actual, expected);
        } else {
            assert!(false, "should not throw deserialize error")
        }
    }
}
