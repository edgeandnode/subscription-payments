use std::{collections::HashMap, io::Cursor};

use prost::Message;
use redis_derive::FromRedisValue;
use serde::{Deserialize, Serialize};
use toolshed::bytes::{Address, DeploymentId};

use crate::utils::build_timerange_timestamp;

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
    /// `signer` field from ticket payload, 0x-prefixed hex
    #[prost(string, tag = "6")]
    pub ticket_signer: ::prost::alloc::string::String,
    /// `name` field from ticket payload
    #[prost(string, optional, tag = "7")]
    pub ticket_name: ::core::option::Option<::prost::alloc::string::String>,
    /// Subgraph Deployment ID, CIDv0 ("Qm" hash)
    #[prost(string, optional, tag = "8")]
    pub deployment: ::core::option::Option<::prost::alloc::string::String>,
    /// Chain name indexed by subgraph deployment
    #[prost(string, optional, tag = "9")]
    pub subgraph_chain: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(uint32, optional, tag = "10")]
    pub query_count: ::core::option::Option<u32>,
    #[prost(float, optional, tag = "11")]
    pub query_budget: ::core::option::Option<f32>,
    #[prost(float, optional, tag = "12")]
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

#[derive(Clone, Debug, PartialEq, Eq, FromRedisValue, Serialize)]
pub enum QueryResultStatus {
    Success,
    UserError,
    IndexerError,
    InternalError,
    Error,
}
impl QueryResultStatus {
    pub fn from_i32(val: i32) -> Self {
        match val {
            0 => QueryResultStatus::Success,
            1 => QueryResultStatus::UserError,
            2 => QueryResultStatus::IndexerError,
            3 => QueryResultStatus::InternalError,
            _ => QueryResultStatus::Error,
        }
    }
}
#[derive(Clone, Debug, FromRedisValue, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct GraphSubscriptionQueryResultRecord {
    pub query_id: String,
    pub deployment_qm_hash: String,
    pub subgraph_chain: Option<String>,
    pub ticket_user: String,
    pub ticket_signer: String,
    pub ticket_name: String,
    pub query_count: u32,
    pub status_code: QueryResultStatus,
    pub status_message: String,
    pub response_time_ms: u32,
    pub query_budget: f32,
    pub indexer_fees: f32,
    /// unix-timestamp when the message was received
    pub timestamp: i64,
    /// kafka message offset
    pub offset: i64,
    /// kafka message key
    pub key: String,
}

impl GraphSubscriptionQueryResultRecord {
    pub fn from_query_result_msg(
        msg: GatewaySubscriptionQueryResult,
        timestamp: i64,
        offset: i64,
        key: String,
    ) -> Self {
        Self {
            query_id: msg.query_id,
            deployment_qm_hash: msg.deployment.unwrap_or_default(),
            subgraph_chain: msg.subgraph_chain,
            ticket_user: msg.ticket_user,
            ticket_signer: msg.ticket_signer,
            ticket_name: msg.ticket_name.unwrap_or_default(),
            query_count: msg.query_count.unwrap_or_default(),
            status_code: QueryResultStatus::from_i32(msg.status_code),
            status_message: msg.status_message,
            response_time_ms: msg.response_time_ms,
            query_budget: msg.query_budget.unwrap_or_default(),
            indexer_fees: msg.indexer_fees.unwrap_or_default(),
            timestamp,
            offset,
            key,
        }
    }

    pub fn calc_total_query_count(records_in_timeframe: &Vec<Self>) -> u32 {
        if records_in_timeframe.is_empty() {
            return 0;
        }
        records_in_timeframe
            .iter()
            .map(|query_result_record| query_result_record.query_count)
            .sum()
    }

    pub fn calc_success_rate_in_timeframe(
        total_query_count: u32,
        records_in_timeframe: &Vec<Self>,
    ) -> f32 {
        if total_query_count == 0 || records_in_timeframe.is_empty() {
            return 0.0;
        }
        let success_query_count: u32 = records_in_timeframe
            .iter()
            .filter(|r| r.status_code == QueryResultStatus::Success)
            .map(|query_result_record| query_result_record.query_count)
            .sum();

        success_query_count as f32 / total_query_count as f32
    }

    pub fn calc_avg_response_time_ms_in_timeframe(records_in_timeframe: &Vec<Self>) -> u32 {
        if records_in_timeframe.is_empty() {
            return 0;
        }
        let total_response_time_ms: u32 = records_in_timeframe
            .iter()
            .map(|query_result_record| query_result_record.response_time_ms)
            .sum();

        total_response_time_ms / records_in_timeframe.len() as u32
    }

    pub fn calc_failed_query_count_in_timeframe(records_in_timeframe: &Vec<Self>) -> u32 {
        if records_in_timeframe.is_empty() {
            return 0;
        }
        records_in_timeframe
            .iter()
            .filter(|r| r.status_code != QueryResultStatus::Success)
            .map(|query_result_record| query_result_record.query_count)
            .sum()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum OrderDirection {
    Asc,
    Desc,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RequestTicket {
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
}

impl RequestTicket {
    pub fn from_graph_subscription_query_result(
        result: GraphSubscriptionQueryResultRecord,
    ) -> Self {
        Self {
            ticket_name: result.ticket_name,
            ticket_user: result.ticket_user.parse::<Address>().unwrap_or_default(),
            ticket_signer: result.ticket_signer.parse::<Address>().unwrap_or_default(),
        }
    }
    /// Convert a list of `GraphSubscriptionQueryResultRecord` to a unique set of `RequestTicket` records.
    /// Records should be unique by `GraphSubscriptionQueryResultRecord.ticket_name`
    pub fn build_unique_request_ticket_list(
        results: Vec<GraphSubscriptionQueryResultRecord>,
    ) -> Vec<Self> {
        results.into_iter().fold(
            Vec::<RequestTicket>::new(),
            |mut tickets, query_result_record| {
                if !tickets
                    .iter()
                    .any(|t| t.ticket_name == query_result_record.ticket_name)
                {
                    tickets.push(RequestTicket::from_graph_subscription_query_result(
                        query_result_record,
                    ));
                }
                tickets
            },
        )
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RequestTicketOrderBy {
    Owner,
    Signer,
    Name,
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
    /// Wallet address of the signer of the request ticket.
    /// This value will often be the `ticket_user`, but could also be an authorized signer for the subscription owner.
    /// Pulled directly from the kafka topic log from The Graph Gateway.
    pub ticket_signer: Address,
    /// Timestamp start of the timeframe the stat record aggregates over.
    pub start: i64,
    /// Timestamp end of the timeframe the stat record aggregates over.
    pub end: i64,
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

impl RequestTicketStat {
    /// Convert a list of `GraphSubscriptionQueryResultRecord` to a unique set of `RequestTicketStat` records.
    /// Records should be unique in the given `start` and `end` timeframe for the `RequestTicket`.
    /// Aggregate the `query_count`, success_rate`, `avg_response_time_ms` and `failed_query_count` values.
    ///
    /// # Arguments
    ///
    /// - `results` - a vector of `GraphSubscriptionQueryResultRecord` retrieved from the redis database and filtered by the `GraphSubscriptionQueryResultRecord.ticket_name`
    pub fn from_graph_subscription_query_result_records(
        results: Vec<GraphSubscriptionQueryResultRecord>,
    ) -> Vec<Self> {
        if results.is_empty() {
            return vec![];
        }
        // map into a chunked map of `HashMap<String,GraphSubscriptionQueryResultRecord>` by the start and end timestamp
        let chunked_results = results.iter().fold(
            HashMap::new(),
            |mut map: HashMap<(i64, i64), Vec<GraphSubscriptionQueryResultRecord>>,
             query_result_record| {
                let (start, end) = build_timerange_timestamp(query_result_record.timestamp);
                map.entry((start, end))
                    .or_insert_with(Vec::<GraphSubscriptionQueryResultRecord>::new)
                    .push(query_result_record.clone());

                map
            },
        );
        // iterate over the map entries to build a set of `RequestTicketStat` records unique by timestamp
        chunked_results.iter().fold(
            Vec::<RequestTicketStat>::new(),
            |mut stats, ((start, end), records_in_timeframe)| {
                let query_count = GraphSubscriptionQueryResultRecord::calc_total_query_count(
                    records_in_timeframe,
                );
                let success_rate =
                    GraphSubscriptionQueryResultRecord::calc_success_rate_in_timeframe(
                        query_count,
                        records_in_timeframe,
                    );
                let avg_response_time_ms =
                    GraphSubscriptionQueryResultRecord::calc_avg_response_time_ms_in_timeframe(
                        records_in_timeframe,
                    );
                let failed_query_count =
                    GraphSubscriptionQueryResultRecord::calc_failed_query_count_in_timeframe(
                        records_in_timeframe,
                    );
                let first_record = records_in_timeframe.first().unwrap();
                // push RequestTicketStat onto the stats array
                stats.push(Self {
                    ticket_name: first_record.ticket_name.to_string(),
                    ticket_user: first_record
                        .ticket_user
                        .parse::<Address>()
                        .unwrap_or_default(),
                    ticket_signer: first_record
                        .ticket_signer
                        .parse::<Address>()
                        .unwrap_or_default(),
                    start: *start,
                    end: *end,
                    query_count,
                    success_rate,
                    avg_response_time_ms,
                    failed_query_count,
                });

                stats
            },
        )
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RequestTicketStatOrderBy {
    Start,
    End,
    TotalQueryCount,
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
    /// Wallet address of the signer of the request ticket.
    /// This value will often be the `ticket_user`, but could also be an authorized signer for the subscription owner.
    /// Pulled directly from the kafka topic log from The Graph Gateway.
    pub ticket_signer: Address,
    /// Timestamp start of the timeframe the stat record aggregates over.
    pub start: i64,
    /// Timestamp end of the timeframe the stat record aggregates over.
    pub end: i64,
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

impl RequestTicketSubgraphStat {
    /// Convert a list of `GraphSubscriptionQueryResultRecord` to a unique set of `RequestTicketSubgraphStat` records.
    /// Records should be unique in the given `start` and `end` timeframe for the `RequestTicket` and deployment qm hash.
    /// Aggregate the `query_count`, success_rate`, `avg_response_time_ms` and `failed_query_count` values.
    ///
    /// # Arguments
    ///
    /// - `results` - a vector of `GraphSubscriptionQueryResultRecord` retrieved from the redis database and filtered by the `GraphSubscriptionQueryResultRecord.ticket_name` & `GraphSubscriptionQueryResultRecord.deployment_qm_hash`
    pub fn from_graph_subscription_query_result_records(
        results: Vec<GraphSubscriptionQueryResultRecord>,
    ) -> Vec<Self> {
        if results.is_empty() {
            return vec![];
        }
        // map into a chunked map of `HashMap<String,GraphSubscriptionQueryResultRecord>` by the start and end timestamp
        let chunked_results = results.iter().fold(
            HashMap::new(),
            |mut map: HashMap<(i64, i64), Vec<GraphSubscriptionQueryResultRecord>>,
             query_result_record| {
                let (start, end) = build_timerange_timestamp(query_result_record.timestamp);
                map.entry((start, end))
                    .or_insert_with(Vec::<GraphSubscriptionQueryResultRecord>::new)
                    .push(query_result_record.clone());

                map
            },
        );
        // iterate over the map entries to build a set of `RequestTicketStat` records unique by timestamp
        chunked_results.iter().fold(
            Vec::<RequestTicketSubgraphStat>::new(),
            |mut stats, ((start, end), records_in_timeframe)| {
                let query_count = GraphSubscriptionQueryResultRecord::calc_total_query_count(
                    records_in_timeframe,
                );
                let success_rate =
                    GraphSubscriptionQueryResultRecord::calc_success_rate_in_timeframe(
                        query_count,
                        records_in_timeframe,
                    );
                let avg_response_time_ms =
                    GraphSubscriptionQueryResultRecord::calc_avg_response_time_ms_in_timeframe(
                        records_in_timeframe,
                    );
                let failed_query_count =
                    GraphSubscriptionQueryResultRecord::calc_failed_query_count_in_timeframe(
                        records_in_timeframe,
                    );
                // grab the first record as the deployment_qm_hash, ticket_name, ticket_user and ticket_signer value will be the same for each record in the timeframe
                let first_record = records_in_timeframe.first().unwrap();
                // push RequestTicketStat onto the stats array
                stats.push(Self {
                    subgraph_deployment_qm_hash: first_record
                        .deployment_qm_hash
                        .parse::<DeploymentId>()
                        .unwrap_or_default(),
                    ticket_name: first_record.ticket_name.to_string(),
                    ticket_user: first_record
                        .ticket_user
                        .parse::<Address>()
                        .unwrap_or_default(),
                    ticket_signer: first_record
                        .ticket_signer
                        .parse::<Address>()
                        .unwrap_or_default(),
                    start: *start,
                    end: *end,
                    query_count,
                    success_rate,
                    avg_response_time_ms,
                    failed_query_count,
                });

                stats
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use prost::Message;
    use std::str::FromStr;

    use super::*;

    #[test]
    fn should_decode_slice_to_gateway_subscription_query_result() {
        let result = GatewaySubscriptionQueryResult {
            query_id: String::from("1"),
            status_code: 0,
            status_message: String::from("success"),
            response_time_ms: 300,
            ticket_name: Some(String::from("test_req_ticket__1")),
            ticket_user: String::from("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"),
            ticket_signer: String::from("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"),
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
    fn should_map_graph_subscription_query_result_record_to_request_ticket() {
        let given = GraphSubscriptionQueryResultRecord {
            query_id: String::from("1"),
            ticket_name: String::from("test_req_ticket__1"),
            ticket_user: String::from("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"),
            ticket_signer: String::from("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"),
            deployment_qm_hash: String::from("Qmadj8x9km1YEyKmRnJ6EkC2zpJZFCfTyTZpuqC3j6e1QH"),
            subgraph_chain: Some(String::from("mainnet")),
            query_count: 1,
            status_code: QueryResultStatus::Success,
            status_message: String::from("success"),
            response_time_ms: 300,
            query_budget: 0.0003,
            indexer_fees: 0.0001,
            timestamp: 1680038176,
            offset: 1,
            key: String::from("msg::1"),
        };
        let expected = RequestTicket {
            ticket_name: String::from("test_req_ticket__1"),
            ticket_user: Address::from_str("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462").unwrap(),
            ticket_signer: Address::from_str("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462").unwrap(),
        };
        let actual = RequestTicket::from_graph_subscription_query_result(given);

        assert_eq!(actual, expected);
    }

    #[test]
    fn build_unique_request_ticket_list_should_return_empty_vec_if_given_empty() {
        assert!(RequestTicket::build_unique_request_ticket_list(vec![]).is_empty());
    }
    #[test]
    fn build_unique_request_ticket_list_should_return_unique_list() {
        let given = vec![
            GraphSubscriptionQueryResultRecord {
                query_id: String::from("1"),
                ticket_name: String::from("test_req_ticket__1"),
                ticket_user: String::from("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"),
                ticket_signer: String::from("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"),
                deployment_qm_hash: String::from("Qmadj8x9km1YEyKmRnJ6EkC2zpJZFCfTyTZpuqC3j6e1QH"),
                subgraph_chain: Some(String::from("mainnet")),
                query_count: 1,
                status_code: QueryResultStatus::Success,
                status_message: String::from("success"),
                response_time_ms: 300,
                query_budget: 0.0003,
                indexer_fees: 0.0001,
                timestamp: 1680038176,
                offset: 1,
                key: String::from("msg::1"),
            },
            GraphSubscriptionQueryResultRecord {
                query_id: String::from("2"),
                ticket_name: String::from("test_req_ticket__1"),
                ticket_user: String::from("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"),
                ticket_signer: String::from("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"),
                deployment_qm_hash: String::from("Qmadj8x9km1YEyKmRnJ6EkC2zpJZFCfTyTZpuqC3j6e1QH"),
                subgraph_chain: Some(String::from("mainnet")),
                query_count: 1,
                status_code: QueryResultStatus::Success,
                status_message: String::from("success"),
                response_time_ms: 300,
                query_budget: 0.0003,
                indexer_fees: 0.0001,
                timestamp: 1680040000,
                offset: 2,
                key: String::from("msg::2"),
            },
            GraphSubscriptionQueryResultRecord {
                query_id: String::from("3"),
                ticket_name: String::from("test_req_ticket__2"),
                ticket_user: String::from("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"),
                ticket_signer: String::from("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"),
                deployment_qm_hash: String::from("Qmadj8x9km1YEyKmRnJ6EkC2zpJZFCfTyTZpuqC3j6e1QH"),
                subgraph_chain: Some(String::from("mainnet")),
                query_count: 1,
                status_code: QueryResultStatus::Success,
                status_message: String::from("success"),
                response_time_ms: 300,
                query_budget: 0.0003,
                indexer_fees: 0.0001,
                timestamp: 1680040000,
                offset: 3,
                key: String::from("msg::3"),
            },
        ];
        let expected = vec![
            RequestTicket {
                ticket_name: String::from("test_req_ticket__1"),
                ticket_user: Address::from_str("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462")
                    .unwrap(),
                ticket_signer: Address::from_str("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462")
                    .unwrap(),
            },
            RequestTicket {
                ticket_name: String::from("test_req_ticket__2"),
                ticket_user: Address::from_str("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462")
                    .unwrap(),
                ticket_signer: Address::from_str("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462")
                    .unwrap(),
            },
        ];

        let actual = RequestTicket::build_unique_request_ticket_list(given);

        assert_eq!(actual, expected);
    }

    #[test]
    fn subgraph_ticket_stat_from_graph_subscription_query_result_records_return_empty() {
        assert!(RequestTicketStat::from_graph_subscription_query_result_records(vec![]).is_empty())
    }
    #[test]
    fn subgraph_ticket_stat_from_graph_subscription_query_result_records() {
        let timestamp_1 = 1679791065; // Sunday, March 26, 2023 12:37:45 AM UTC
        let (start_1, end_1) = build_timerange_timestamp(timestamp_1);
        let timestamp_3 = 1679963865; // Tuesday, March 28, 2023 12:37:45 AM UTC
        let (start_3, end_3) = build_timerange_timestamp(timestamp_3);
        let given = vec![
            GraphSubscriptionQueryResultRecord {
                query_id: String::from("1"),
                ticket_name: String::from("test_req_ticket__1"),
                ticket_user: String::from("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"),
                ticket_signer: String::from("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"),
                deployment_qm_hash: String::from("Qmadj8x9km1YEyKmRnJ6EkC2zpJZFCfTyTZpuqC3j6e1QH"),
                subgraph_chain: Some(String::from("mainnet")),
                query_count: 1,
                status_code: QueryResultStatus::Success,
                status_message: String::from("success"),
                response_time_ms: 300,
                query_budget: 0.0003,
                indexer_fees: 0.0001,
                timestamp: timestamp_1,
                offset: 1,
                key: String::from("msg::1"),
            },
            // should have same start and end as first record, and map to same `RequestTicketStat`
            GraphSubscriptionQueryResultRecord {
                query_id: String::from("2"),
                ticket_name: String::from("test_req_ticket__1"),
                ticket_user: String::from("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"),
                ticket_signer: String::from("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"),
                deployment_qm_hash: String::from("Qmadj8x9km1YEyKmRnJ6EkC2zpJZFCfTyTZpuqC3j6e1QH"),
                subgraph_chain: Some(String::from("mainnet")),
                query_count: 1,
                status_code: QueryResultStatus::Success,
                status_message: String::from("success"),
                response_time_ms: 400,
                query_budget: 0.0003,
                indexer_fees: 0.0001,
                timestamp: 1679834265, // Sunday, March 26, 2023 12:37:45 PM UTC
                offset: 2,
                key: String::from("msg::2"),
            },
            GraphSubscriptionQueryResultRecord {
                query_id: String::from("3"),
                ticket_name: String::from("test_req_ticket__1"),
                ticket_user: String::from("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"),
                ticket_signer: String::from("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"),
                deployment_qm_hash: String::from("Qmadj8x9km1YEyKmRnJ6EkC2zpJZFCfTyTZpuqC3j6e1QH"),
                subgraph_chain: Some(String::from("mainnet")),
                query_count: 100,
                status_code: QueryResultStatus::InternalError,
                status_message: String::from("success"),
                response_time_ms: 300,
                query_budget: 0.0003,
                indexer_fees: 0.0001,
                timestamp: timestamp_3,
                offset: 3,
                key: String::from("msg::3"),
            },
        ];
        let mut expected = vec![
            RequestTicketStat {
                ticket_name: String::from("test_req_ticket__1"),
                ticket_user: Address::from_str("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462")
                    .unwrap(),
                ticket_signer: Address::from_str("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462")
                    .unwrap(),
                start: start_1,
                end: end_1,
                query_count: 2,
                avg_response_time_ms: (300 + 400) / 2 as u32,
                success_rate: 1.0,
                failed_query_count: 0,
            },
            RequestTicketStat {
                ticket_name: String::from("test_req_ticket__1"),
                ticket_user: Address::from_str("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462")
                    .unwrap(),
                ticket_signer: Address::from_str("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462")
                    .unwrap(),
                start: start_3,
                end: end_3,
                query_count: 100,
                avg_response_time_ms: 300,
                success_rate: 0.0,
                failed_query_count: 100,
            },
        ];

        let mut actual = RequestTicketStat::from_graph_subscription_query_result_records(given);

        actual.sort_by(|a, b| a.start.cmp(&b.start));
        expected.sort_by(|a, b| a.start.cmp(&b.start));
        assert_eq!(actual, expected);
    }

    #[test]
    fn subgraph_ticket_subgraph_stat_from_graph_subscription_query_result_records_return_empty() {
        assert!(
            RequestTicketSubgraphStat::from_graph_subscription_query_result_records(vec![])
                .is_empty()
        )
    }
    #[test]
    fn subgraph_ticket_subgraph_stat_from_graph_subscription_query_result_records() {
        let timestamp_1 = 1679791065; // Sunday, March 26, 2023 12:37:45 AM UTC
        let (start_1, end_1) = build_timerange_timestamp(timestamp_1);
        let timestamp_3 = 1679963865; // Tuesday, March 28, 2023 12:37:45 AM UTC
        let (start_3, end_3) = build_timerange_timestamp(timestamp_3);
        let given = vec![
            GraphSubscriptionQueryResultRecord {
                query_id: String::from("1"),
                ticket_name: String::from("test_req_ticket__1"),
                ticket_user: String::from("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"),
                ticket_signer: String::from("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"),
                deployment_qm_hash: String::from("Qmadj8x9km1YEyKmRnJ6EkC2zpJZFCfTyTZpuqC3j6e1QH"),
                subgraph_chain: Some(String::from("mainnet")),
                query_count: 1,
                status_code: QueryResultStatus::Success,
                status_message: String::from("success"),
                response_time_ms: 300,
                query_budget: 0.0003,
                indexer_fees: 0.0001,
                timestamp: timestamp_1,
                offset: 1,
                key: String::from("msg::1"),
            },
            // should have same start and end as first record, and map to same `RequestTicketStat`
            GraphSubscriptionQueryResultRecord {
                query_id: String::from("2"),
                ticket_name: String::from("test_req_ticket__1"),
                ticket_user: String::from("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"),
                ticket_signer: String::from("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"),
                deployment_qm_hash: String::from("Qmadj8x9km1YEyKmRnJ6EkC2zpJZFCfTyTZpuqC3j6e1QH"),
                subgraph_chain: Some(String::from("mainnet")),
                query_count: 1,
                status_code: QueryResultStatus::Success,
                status_message: String::from("success"),
                response_time_ms: 400,
                query_budget: 0.0003,
                indexer_fees: 0.0001,
                timestamp: 1679834265, // Sunday, March 26, 2023 12:37:45 PM UTC
                offset: 2,
                key: String::from("msg::2"),
            },
            GraphSubscriptionQueryResultRecord {
                query_id: String::from("3"),
                ticket_name: String::from("test_req_ticket__1"),
                ticket_user: String::from("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"),
                ticket_signer: String::from("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"),
                deployment_qm_hash: String::from("Qmadj8x9km1YEyKmRnJ6EkC2zpJZFCfTyTZpuqC3j6e1QH"),
                subgraph_chain: Some(String::from("mainnet")),
                query_count: 100,
                status_code: QueryResultStatus::InternalError,
                status_message: String::from("success"),
                response_time_ms: 300,
                query_budget: 0.0003,
                indexer_fees: 0.0001,
                timestamp: timestamp_3,
                offset: 3,
                key: String::from("msg::3"),
            },
        ];
        let mut expected = vec![
            RequestTicketSubgraphStat {
                subgraph_deployment_qm_hash: DeploymentId::from_str(
                    "Qmadj8x9km1YEyKmRnJ6EkC2zpJZFCfTyTZpuqC3j6e1QH",
                )
                .unwrap(),
                ticket_name: String::from("test_req_ticket__1"),
                ticket_user: Address::from_str("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462")
                    .unwrap(),
                ticket_signer: Address::from_str("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462")
                    .unwrap(),
                start: start_1,
                end: end_1,
                query_count: 2,
                avg_response_time_ms: (300 + 400) / 2 as u32,
                success_rate: 1.0,
                failed_query_count: 0,
            },
            RequestTicketSubgraphStat {
                subgraph_deployment_qm_hash: DeploymentId::from_str(
                    "Qmadj8x9km1YEyKmRnJ6EkC2zpJZFCfTyTZpuqC3j6e1QH",
                )
                .unwrap(),
                ticket_name: String::from("test_req_ticket__1"),
                ticket_user: Address::from_str("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462")
                    .unwrap(),
                ticket_signer: Address::from_str("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462")
                    .unwrap(),
                start: start_3,
                end: end_3,
                query_count: 100,
                avg_response_time_ms: 300,
                success_rate: 0.0,
                failed_query_count: 100,
            },
        ];

        let mut actual =
            RequestTicketSubgraphStat::from_graph_subscription_query_result_records(given);
        actual.sort_by(|a, b| a.start.cmp(&b.start));
        expected.sort_by(|a, b| a.start.cmp(&b.start));
        assert_eq!(actual, expected);
    }

    #[test]
    fn calc_total_query_count_return_default_if_given_empty() {
        assert_eq!(
            GraphSubscriptionQueryResultRecord::calc_total_query_count(&vec![]),
            0
        );
    }
    #[test]
    fn calc_total_query_count_return_query_count_sum() {
        let given = vec![
            GraphSubscriptionQueryResultRecord {
                query_id: String::from("1"),
                ticket_name: String::from("test_req_ticket__1"),
                ticket_user: String::from("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"),
                ticket_signer: String::from("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"),
                deployment_qm_hash: String::from("Qmadj8x9km1YEyKmRnJ6EkC2zpJZFCfTyTZpuqC3j6e1QH"),
                subgraph_chain: Some(String::from("mainnet")),
                query_count: 1,
                status_code: QueryResultStatus::Success,
                status_message: String::from("success"),
                response_time_ms: 300,
                query_budget: 0.0003,
                indexer_fees: 0.0001,
                timestamp: 1679791065,
                offset: 1,
                key: String::from("msg::1"),
            },
            GraphSubscriptionQueryResultRecord {
                query_id: String::from("2"),
                ticket_name: String::from("test_req_ticket__1"),
                ticket_user: String::from("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"),
                ticket_signer: String::from("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"),
                deployment_qm_hash: String::from("Qmadj8x9km1YEyKmRnJ6EkC2zpJZFCfTyTZpuqC3j6e1QH"),
                subgraph_chain: Some(String::from("mainnet")),
                query_count: 1,
                status_code: QueryResultStatus::Success,
                status_message: String::from("success"),
                response_time_ms: 400,
                query_budget: 0.0003,
                indexer_fees: 0.0001,
                timestamp: 1679834265, // Sunday, March 26, 2023 12:37:45 PM UTC
                offset: 2,
                key: String::from("msg::2"),
            },
        ];
        let expected = 2;
        let actual = GraphSubscriptionQueryResultRecord::calc_total_query_count(&given);

        assert_eq!(actual, expected);
    }

    #[test]
    fn calc_success_rate_in_timeframe_return_default_if_given_0_query_count() {
        assert_eq!(
            GraphSubscriptionQueryResultRecord::calc_success_rate_in_timeframe(0, &vec![]),
            0.0
        );
    }
    #[test]
    fn calc_success_rate_in_timeframe_return_default_if_given_empty_vec() {
        assert_eq!(
            GraphSubscriptionQueryResultRecord::calc_success_rate_in_timeframe(100, &vec![]),
            0.0
        );
    }
    #[test]
    fn calc_success_rate_in_timeframe() {
        let given = vec![
            GraphSubscriptionQueryResultRecord {
                query_id: String::from("1"),
                ticket_name: String::from("test_req_ticket__1"),
                ticket_user: String::from("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"),
                ticket_signer: String::from("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"),
                deployment_qm_hash: String::from("Qmadj8x9km1YEyKmRnJ6EkC2zpJZFCfTyTZpuqC3j6e1QH"),
                subgraph_chain: Some(String::from("mainnet")),
                query_count: 1,
                status_code: QueryResultStatus::Success,
                status_message: String::from("success"),
                response_time_ms: 300,
                query_budget: 0.0003,
                indexer_fees: 0.0001,
                timestamp: 1679791065,
                offset: 1,
                key: String::from("msg::1"),
            },
            GraphSubscriptionQueryResultRecord {
                query_id: String::from("2"),
                ticket_name: String::from("test_req_ticket__1"),
                ticket_user: String::from("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"),
                ticket_signer: String::from("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"),
                deployment_qm_hash: String::from("Qmadj8x9km1YEyKmRnJ6EkC2zpJZFCfTyTZpuqC3j6e1QH"),
                subgraph_chain: Some(String::from("mainnet")),
                query_count: 1,
                status_code: QueryResultStatus::InternalError,
                status_message: String::from("big bad error"),
                response_time_ms: 400,
                query_budget: 0.0003,
                indexer_fees: 0.0001,
                timestamp: 1679834265, // Sunday, March 26, 2023 12:37:45 PM UTC
                offset: 2,
                key: String::from("msg::2"),
            },
        ];
        let expected = 0.5;
        let actual = GraphSubscriptionQueryResultRecord::calc_success_rate_in_timeframe(2, &given);

        assert_eq!(actual, expected);
    }

    #[test]
    fn calc_avg_response_time_ms_in_timeframe_return_def_if_given_empty() {
        assert_eq!(
            GraphSubscriptionQueryResultRecord::calc_avg_response_time_ms_in_timeframe(&vec![]),
            0
        );
    }
    #[test]
    fn calc_avg_response_time_ms_in_timeframe() {
        let given = vec![
            GraphSubscriptionQueryResultRecord {
                query_id: String::from("1"),
                ticket_name: String::from("test_req_ticket__1"),
                ticket_user: String::from("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"),
                ticket_signer: String::from("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"),
                deployment_qm_hash: String::from("Qmadj8x9km1YEyKmRnJ6EkC2zpJZFCfTyTZpuqC3j6e1QH"),
                subgraph_chain: Some(String::from("mainnet")),
                query_count: 1,
                status_code: QueryResultStatus::Success,
                status_message: String::from("success"),
                response_time_ms: 300,
                query_budget: 0.0003,
                indexer_fees: 0.0001,
                timestamp: 1679791065,
                offset: 1,
                key: String::from("msg::1"),
            },
            GraphSubscriptionQueryResultRecord {
                query_id: String::from("2"),
                ticket_name: String::from("test_req_ticket__1"),
                ticket_user: String::from("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"),
                ticket_signer: String::from("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"),
                deployment_qm_hash: String::from("Qmadj8x9km1YEyKmRnJ6EkC2zpJZFCfTyTZpuqC3j6e1QH"),
                subgraph_chain: Some(String::from("mainnet")),
                query_count: 1,
                status_code: QueryResultStatus::InternalError,
                status_message: String::from("big bad error"),
                response_time_ms: 400,
                query_budget: 0.0003,
                indexer_fees: 0.0001,
                timestamp: 1679834265, // Sunday, March 26, 2023 12:37:45 PM UTC
                offset: 2,
                key: String::from("msg::2"),
            },
        ];
        let expected: u32 = (300 + 400) / 2;
        let actual =
            GraphSubscriptionQueryResultRecord::calc_avg_response_time_ms_in_timeframe(&given);

        assert_eq!(actual, expected);
    }

    #[test]
    fn calc_failed_query_count_in_timeframe_return_def_if_given_empty_vec() {
        assert_eq!(
            GraphSubscriptionQueryResultRecord::calc_failed_query_count_in_timeframe(&vec![]),
            0
        );
    }
    #[test]
    fn calc_failed_query_count_in_timeframe_return_0_if_no_failures() {
        let given = vec![GraphSubscriptionQueryResultRecord {
            query_id: String::from("1"),
            ticket_name: String::from("test_req_ticket__1"),
            ticket_user: String::from("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"),
            ticket_signer: String::from("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"),
            deployment_qm_hash: String::from("Qmadj8x9km1YEyKmRnJ6EkC2zpJZFCfTyTZpuqC3j6e1QH"),
            subgraph_chain: Some(String::from("mainnet")),
            query_count: 1,
            status_code: QueryResultStatus::Success,
            status_message: String::from("success"),
            response_time_ms: 300,
            query_budget: 0.0003,
            indexer_fees: 0.0001,
            timestamp: 1679791065,
            offset: 1,
            key: String::from("msg::1"),
        }];
        let actual =
            GraphSubscriptionQueryResultRecord::calc_failed_query_count_in_timeframe(&given);

        assert_eq!(actual, 0);
    }
    #[test]
    fn calc_failed_query_count_in_timeframe_return_failure_count() {
        let given = vec![
            GraphSubscriptionQueryResultRecord {
                query_id: String::from("1"),
                ticket_name: String::from("test_req_ticket__1"),
                ticket_user: String::from("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"),
                ticket_signer: String::from("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"),
                deployment_qm_hash: String::from("Qmadj8x9km1YEyKmRnJ6EkC2zpJZFCfTyTZpuqC3j6e1QH"),
                subgraph_chain: Some(String::from("mainnet")),
                query_count: 1,
                status_code: QueryResultStatus::Success,
                status_message: String::from("success"),
                response_time_ms: 300,
                query_budget: 0.0003,
                indexer_fees: 0.0001,
                timestamp: 1679791065,
                offset: 1,
                key: String::from("msg::1"),
            },
            GraphSubscriptionQueryResultRecord {
                query_id: String::from("2"),
                ticket_name: String::from("test_req_ticket__1"),
                ticket_user: String::from("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"),
                ticket_signer: String::from("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"),
                deployment_qm_hash: String::from("Qmadj8x9km1YEyKmRnJ6EkC2zpJZFCfTyTZpuqC3j6e1QH"),
                subgraph_chain: Some(String::from("mainnet")),
                query_count: 1,
                status_code: QueryResultStatus::InternalError,
                status_message: String::from("big bad error"),
                response_time_ms: 400,
                query_budget: 0.0003,
                indexer_fees: 0.0001,
                timestamp: 1679834265, // Sunday, March 26, 2023 12:37:45 PM UTC
                offset: 2,
                key: String::from("msg::2"),
            },
        ];
        let actual =
            GraphSubscriptionQueryResultRecord::calc_failed_query_count_in_timeframe(&given);

        assert_eq!(actual, 1);
    }
}
