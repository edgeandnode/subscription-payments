use std::collections::HashMap;

use anyhow::{Ok, Result};
use async_trait::async_trait;
use chrono::Utc;
use eventuals::{Eventual, EventualWriter, Ptr};
use futures::TryStreamExt;
use rdkafka::consumer::{DefaultConsumerContext, StreamConsumer};
use rdkafka::error::KafkaError;
use rdkafka::Message;
use toolshed::bytes::{Address, DeploymentId};

use crate::datasource::{Datasource, DatasourceWriter};
use crate::models::*;

/// The in-memory datasource implements both the `Datasource` & `DatasourceWriter` traits.
/// When a message is received on the kafka topic, it will store the data in an `Eventual` in-memory instance to be accessed on read.
pub struct DatasourceInMemory {
    gateway_subscription_query_result_writer:
        EventualWriter<Ptr<HashMap<Address, Vec<GraphSubscriptionQueryResultRecord>>>>,
    gateway_subscription_query_result_tx:
        Eventual<Ptr<HashMap<Address, Vec<GraphSubscriptionQueryResultRecord>>>>,
}

impl DatasourceInMemory {
    pub fn create() -> &'static Self {
        let (gateway_subscription_query_result_writer, gateway_subscription_query_result_tx) =
            Eventual::<Ptr<HashMap<Address, Vec<GraphSubscriptionQueryResultRecord>>>>::new();

        Box::leak(Box::new(Self {
            gateway_subscription_query_result_writer,
            gateway_subscription_query_result_tx,
        }))
    }
}

#[async_trait]
impl Datasource for DatasourceInMemory {
    /// Retrieve the user's unique `RequestTicket` records stored in-memory.
    ///
    /// # Arguments
    ///
    /// - `user` - the user wallet address who performed the stored queries
    /// - `first` - [OPTIONAL:default 100] the number of records, after sorting, to return
    /// - `skip` - [OPTIONAL:default 0] the number of records, after sorting, to skip
    /// - `order_by` - [OPTIONAL] what field on the `RequestTicket` to sort by
    /// - `order_direction` [OPTIONAL] the sort direction
    async fn request_tickets(
        &self,
        user: Address,
        first: Option<i32>,
        skip: Option<i32>,
        order_by: Option<RequestTicketOrderBy>,
        order_direction: Option<OrderDirection>,
    ) -> Result<Vec<RequestTicket>> {
        let results = self
            .gateway_subscription_query_result_tx
            .value_immediate()
            .unwrap_or_default()
            .get(&user)
            .cloned();
        if results.is_none() {
            return Ok(vec![]);
        }
        let mut tickets =
            RequestTicket::build_unique_request_ticket_list(results.unwrap_or_default());
        if order_by.is_some() {
            let direction = order_direction.unwrap_or(OrderDirection::Asc);
            let order = order_by.unwrap();
            tickets.sort_by(|a, b| match order {
                RequestTicketOrderBy::Owner => {
                    if direction == OrderDirection::Asc {
                        a.ticket_user.cmp(&b.ticket_user)
                    } else {
                        a.ticket_user.cmp(&b.ticket_user).reverse()
                    }
                }
                RequestTicketOrderBy::Signer => {
                    if direction == OrderDirection::Asc {
                        a.ticket_signer.cmp(&b.ticket_signer)
                    } else {
                        a.ticket_signer.cmp(&b.ticket_signer).reverse()
                    }
                }
                RequestTicketOrderBy::Name => {
                    if direction == OrderDirection::Asc {
                        a.ticket_name.cmp(&b.ticket_name)
                    } else {
                        a.ticket_name.cmp(&b.ticket_name).reverse()
                    }
                }
            })
        }
        let take = first.unwrap_or(100) as usize;
        let skip = skip.unwrap_or(0) as usize;

        Ok(tickets.into_iter().skip(skip).take(take).collect())
    }

    /// Retrieve the user's `RequestTicketStat` records, aggregated over the given timeframe, stored in-memory.
    ///
    /// # Arguments
    ///
    /// - `user` - the user wallet address who performed the stored queries
    /// - `ticket_name` - the name of the request ticket to get stats for
    /// - `first` - [OPTIONAL:default 100] the number of records, after sorting, to return
    /// - `skip` - [OPTIONAL:default 0] the number of records, after sorting, to skip
    /// - `order_by` - [OPTIONAL:default start] what field on the `RequestTicketStat` to sort by
    /// - `order_direction` [OPTIONAL: default asc] the sort direction
    async fn request_ticket_stats(
        &self,
        user: Address,
        ticket_name: String,
        first: Option<i32>,
        skip: Option<i32>,
        order_by: Option<RequestTicketStatOrderBy>,
        order_direction: Option<OrderDirection>,
    ) -> Result<Vec<RequestTicketStat>> {
        let results = self
            .gateway_subscription_query_result_tx
            .value_immediate()
            .unwrap_or_default()
            .get(&user)
            .cloned();
        if results.is_none() {
            return Ok(vec![]);
        }
        // filter out results that don't match the `ticket_name`
        let filtered: Vec<GraphSubscriptionQueryResultRecord> = results
            .unwrap_or_default()
            .into_iter()
            .filter(|result| result.ticket_name == ticket_name)
            .collect();

        let mut stats = RequestTicketStat::from_graph_subscription_query_result_records(filtered);
        let order_by = order_by.unwrap_or(RequestTicketStatOrderBy::Start);
        let order_direction = order_direction.unwrap_or(OrderDirection::Asc);
        stats.sort_by(|a, b| match order_by {
            RequestTicketStatOrderBy::Start => {
                if order_direction == OrderDirection::Asc {
                    a.start.cmp(&b.start)
                } else {
                    a.start.cmp(&b.start).reverse()
                }
            }
            RequestTicketStatOrderBy::End => {
                if order_direction == OrderDirection::Asc {
                    a.end.cmp(&b.end)
                } else {
                    a.end.cmp(&b.end).reverse()
                }
            }
            RequestTicketStatOrderBy::TotalQueryCount => {
                if order_direction == OrderDirection::Asc {
                    a.query_count.cmp(&b.query_count)
                } else {
                    a.query_count.cmp(&b.query_count).reverse()
                }
            }
        });
        let take = first.unwrap_or(100) as usize;
        let skip = skip.unwrap_or(0) as usize;

        Ok(stats.into_iter().skip(skip).take(take).collect())
    }

    /// Retrieve the user's `RequestTicketSubgraphStat` records, aggregated over the given timeframe, for a specific Subgraph deployment Qm hash, stored in-memory.
    ///
    /// # Arguments
    ///
    /// - `user` - the user wallet address who performed the stored queries
    /// - `ticket_name` - the name of the request ticket to get stats for
    /// - `subgraph_deployment_qm_hash` - the Subgraph deployment Qm hash
    /// - `first` - [OPTIONAL:default 100] the number of records, after sorting, to return
    /// - `skip` - [OPTIONAL:default 0] the number of records, after sorting, to skip
    /// - `order_by` - [OPTIONAL] what field on the `RequestTicketStat` to sort by
    /// - `order_direction` [OPTIONAL] the sort direction
    async fn request_ticket_subgraph_stats(
        &self,
        user: Address,
        ticket_name: String,
        subgraph_deployment_qm_hash: DeploymentId,
        first: Option<i32>,
        skip: Option<i32>,
        order_by: Option<RequestTicketStatOrderBy>,
        order_direction: Option<OrderDirection>,
    ) -> Result<Vec<RequestTicketSubgraphStat>> {
        let results = self
            .gateway_subscription_query_result_tx
            .value_immediate()
            .unwrap_or_default()
            .get(&user)
            .cloned();
        if results.is_none() {
            return Ok(vec![]);
        }
        // filter out results that don't match the `ticket_name` & `subgraph_deployment_qm_hash`
        let filtered: Vec<GraphSubscriptionQueryResultRecord> = results
            .unwrap_or_default()
            .into_iter()
            .filter(|result| {
                result.ticket_name == ticket_name
                    && result.deployment_qm_hash == subgraph_deployment_qm_hash.ipfs_hash()
            })
            .collect();

        let mut stats =
            RequestTicketSubgraphStat::from_graph_subscription_query_result_records(filtered);
        let order_by = order_by.unwrap_or(RequestTicketStatOrderBy::Start);
        let order_direction = order_direction.unwrap_or(OrderDirection::Asc);
        stats.sort_by(|a, b| match order_by {
            RequestTicketStatOrderBy::Start => {
                if order_direction == OrderDirection::Asc {
                    a.start.cmp(&b.start)
                } else {
                    a.start.cmp(&b.start).reverse()
                }
            }
            RequestTicketStatOrderBy::End => {
                if order_direction == OrderDirection::Asc {
                    a.end.cmp(&b.end)
                } else {
                    a.end.cmp(&b.end).reverse()
                }
            }
            RequestTicketStatOrderBy::TotalQueryCount => {
                if order_direction == OrderDirection::Asc {
                    a.query_count.cmp(&b.query_count)
                } else {
                    a.query_count.cmp(&b.query_count).reverse()
                }
            }
        });
        let take = first.unwrap_or(100) as usize;
        let skip = skip.unwrap_or(0) as usize;

        Ok(stats.into_iter().skip(skip).take(take).collect())
    }
}

#[async_trait]
impl DatasourceWriter for DatasourceInMemory {
    async fn write(&self, consumer: &StreamConsumer<DefaultConsumerContext>) {
        let stream_processor = consumer.stream().try_for_each(|borrowed_msg| async move {
            let msg = borrowed_msg.detach();
            // convert the `OwnedMessage`
            let query_result_msg = match GatewaySubscriptionQueryResult::from_slice(
                msg.payload().unwrap_or_default(),
            ) {
                Err(err) => {
                    tracing::warn!("DatasourceRedis.write()::cannot deserialize message. skipping offset: [{}]. {}", msg.offset(), err);
                    return Result::<(), KafkaError>::Ok(());
                }
                Result::Ok(payload) => payload,
            };
            // build a `GraphSubscriptionQueryResultRecord`
            let timestamp = msg
                .timestamp()
                .to_millis()
                .and_then(|ms| Some(ms / 1000))
                .unwrap_or(Utc::now().timestamp());
            let offset = msg.offset();
            let key = String::from_utf8_lossy(msg.key().unwrap_or_default()).to_string();
            let subscription_query_result_record = GraphSubscriptionQueryResultRecord::from_query_result_msg(query_result_msg, timestamp, offset, key);
            let _user = match subscription_query_result_record.ticket_user.parse::<Address>() {
                Result::Ok(user_addr) => user_addr,
                Err(err) => {
                    tracing::warn!("DatasourceRedis.write()::failure parsing record ticket_user as an `Address` instance. skipping offset: [{}]. {}", msg.offset(), err);
                    return Result::<(), KafkaError>::Ok(());
                }
            };
            // TODO: Update retrieved records in `Self` instance

            Result::<(), KafkaError>::Ok(())
        });

        tracing::info!(
            "InMemoryDatasource.write()::initializing message stream consumer processing..."
        );
        stream_processor
            .await
            .expect("Failure processing the DatasourceInMemory stream writer");
        tracing::info!("InMemoryDatasource.write()::message stream consumer terminated");
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::utils::*;

    use super::*;

    impl DatasourceInMemory {
        fn create_with_initial_data(
            initial_data: HashMap<Address, Vec<GraphSubscriptionQueryResultRecord>>,
        ) -> &'static Self {
            let (gateway_subscription_query_result_writer, _) =
                Eventual::<Ptr<HashMap<Address, Vec<GraphSubscriptionQueryResultRecord>>>>::new();

            Box::leak(Box::new(Self {
                gateway_subscription_query_result_writer,
                gateway_subscription_query_result_tx: Eventual::from_value(Ptr::new(initial_data)),
            }))
        }
    }

    #[tokio::test]
    async fn req_tickets_should_return_empty_if_no_results_inmemory() {
        let client = DatasourceInMemory::create();
        let tickets = client
            .request_tickets(
                Address::from_str("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462").unwrap(),
                None,
                None,
                None,
                None,
            )
            .await
            .unwrap_or_default();

        assert!(tickets.is_empty());
    }
    #[tokio::test]
    async fn req_tickets_should_return_unique_list() {
        let user = "0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"
            .parse::<Address>()
            .unwrap();

        let query_results: Vec<GraphSubscriptionQueryResultRecord> = vec![
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
        let mut query_results_map =
            HashMap::<Address, Vec<GraphSubscriptionQueryResultRecord>>::new();
        query_results_map.insert(user, query_results);

        let mut expected = vec![
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

        let client = DatasourceInMemory::create_with_initial_data(query_results_map);

        let actual_no_filters = client
            .request_tickets(user, None, None, None, None)
            .await
            .unwrap_or_default();
        assert_eq!(actual_no_filters, expected);

        // only return 1
        let actual_first = client
            .request_tickets(user, Some(1), None, None, None)
            .await
            .unwrap_or_default();
        let expected_first: Vec<RequestTicket> = vec![RequestTicket {
            ticket_name: String::from("test_req_ticket__1"),
            ticket_user: Address::from_str("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462").unwrap(),
            ticket_signer: Address::from_str("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462").unwrap(),
        }];
        assert_eq!(actual_first.len(), 1);
        assert_eq!(actual_first, expected_first);

        // only return 1, but skip 1
        let actual_skip = client
            .request_tickets(user, Some(1), Some(1), None, None)
            .await
            .unwrap_or_default();
        let expected_skip: Vec<RequestTicket> = vec![RequestTicket {
            ticket_name: String::from("test_req_ticket__2"),
            ticket_user: Address::from_str("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462").unwrap(),
            ticket_signer: Address::from_str("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462").unwrap(),
        }];
        assert_eq!(actual_skip.len(), 1);
        assert_eq!(actual_skip, expected_skip);

        // order by ticket_name, desc
        let actual_ordered_desc = client
            .request_tickets(
                user,
                None,
                None,
                Some(RequestTicketOrderBy::Name),
                Some(OrderDirection::Desc),
            )
            .await
            .unwrap_or_default();
        expected.sort_by(|a, b| a.ticket_name.cmp(&b.ticket_name).reverse());
        assert_eq!(actual_ordered_desc, expected);
    }

    #[tokio::test]
    async fn req_ticket_stats_should_return_empty_if_no_results_inmemory() {
        let client = DatasourceInMemory::create();
        let stats = client
            .request_ticket_stats(
                Address::from_str("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462").unwrap(),
                String::from("test_req_ticket_1"),
                None,
                None,
                None,
                None,
            )
            .await
            .unwrap_or_default();

        assert!(stats.is_empty());
    }
    #[tokio::test]
    async fn req_tickets_stats_should_return_list() {
        let user = "0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"
            .parse::<Address>()
            .unwrap();

        let timestamp_1 = 1679791065; // Sunday, March 26, 2023 12:37:45 AM UTC
        let (start_1, end_1) = build_timerange_timestamp(timestamp_1);
        let timestamp_3 = 1679963865; // Tuesday, March 28, 2023 12:37:45 AM UTC
        let (start_3, end_3) = build_timerange_timestamp(timestamp_3);
        let query_results = vec![
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
            GraphSubscriptionQueryResultRecord {
                query_id: String::from("4"),
                ticket_name: String::from("test_req_ticket__2"),
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
                offset: 4,
                key: String::from("msg::4"),
            },
        ];
        let mut query_results_map =
            HashMap::<Address, Vec<GraphSubscriptionQueryResultRecord>>::new();
        query_results_map.insert(user, query_results);

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

        let client = DatasourceInMemory::create_with_initial_data(query_results_map);

        let actual_no_filters = client
            .request_ticket_stats(
                user,
                String::from("test_req_ticket__1"),
                None,
                None,
                None,
                None,
            )
            .await
            .unwrap_or_default();
        assert_eq!(actual_no_filters, expected);
        // should not return any stats for ticket: `test_req_ticket__2`
        assert!(!actual_no_filters
            .into_iter()
            .any(|stat| stat.ticket_name == String::from("test_req_ticket__2")));

        // only return 1
        let actual_first = client
            .request_ticket_stats(
                user,
                String::from("test_req_ticket__1"),
                Some(1),
                None,
                None,
                None,
            )
            .await
            .unwrap_or_default();
        let expected_first: Vec<RequestTicketStat> = vec![RequestTicketStat {
            ticket_name: String::from("test_req_ticket__1"),
            ticket_user: Address::from_str("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462").unwrap(),
            ticket_signer: Address::from_str("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462").unwrap(),
            start: start_1,
            end: end_1,
            query_count: 2,
            avg_response_time_ms: (300 + 400) / 2 as u32,
            success_rate: 1.0,
            failed_query_count: 0,
        }];
        assert_eq!(actual_first.len(), 1);
        assert_eq!(actual_first, expected_first);

        // only return 1, but skip 1
        let actual_skip = client
            .request_ticket_stats(
                user,
                String::from("test_req_ticket__1"),
                Some(1),
                Some(1),
                None,
                None,
            )
            .await
            .unwrap_or_default();
        let expected_skip = vec![RequestTicketStat {
            ticket_name: String::from("test_req_ticket__1"),
            ticket_user: Address::from_str("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462").unwrap(),
            ticket_signer: Address::from_str("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462").unwrap(),
            start: start_3,
            end: end_3,
            query_count: 100,
            avg_response_time_ms: 300,
            success_rate: 0.0,
            failed_query_count: 100,
        }];
        assert_eq!(actual_skip.len(), 1);
        assert_eq!(actual_skip, expected_skip);

        // order by ticket_name, desc
        let actual_ordered_desc = client
            .request_ticket_stats(
                user,
                String::from("test_req_ticket__1"),
                None,
                None,
                Some(RequestTicketStatOrderBy::Start),
                Some(OrderDirection::Desc),
            )
            .await
            .unwrap_or_default();
        expected.sort_by(|a, b| a.start.cmp(&b.start).reverse());
        assert_eq!(actual_ordered_desc, expected);
    }

    #[tokio::test]
    async fn req_ticket_subgraph_stats_should_return_empty_if_no_results_inmemory() {
        let client = DatasourceInMemory::create();
        let stats = client
            .request_ticket_subgraph_stats(
                Address::from_str("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462").unwrap(),
                String::from("test_req_ticket_1"),
                DeploymentId::from_ipfs_hash("Qmadj8x9km1YEyKmRnJ6EkC2zpJZFCfTyTZpuqC3j6e1QH")
                    .unwrap(),
                None,
                None,
                None,
                None,
            )
            .await
            .unwrap_or_default();

        assert!(stats.is_empty());
    }
    #[tokio::test]
    async fn req_tickets_subgraph_stats_should_return_list() {
        let user = "0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"
            .parse::<Address>()
            .unwrap();
        let deployment_id =
            DeploymentId::from_str("Qmadj8x9km1YEyKmRnJ6EkC2zpJZFCfTyTZpuqC3j6e1QH").unwrap();

        let timestamp_1 = 1679791065; // Sunday, March 26, 2023 12:37:45 AM UTC
        let (start_1, end_1) = build_timerange_timestamp(timestamp_1);
        let timestamp_3 = 1679963865; // Tuesday, March 28, 2023 12:37:45 AM UTC
        let (start_3, end_3) = build_timerange_timestamp(timestamp_3);
        let query_results = vec![
            GraphSubscriptionQueryResultRecord {
                query_id: String::from("1"),
                ticket_name: String::from("test_req_ticket__1"),
                ticket_user: String::from("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"),
                ticket_signer: String::from("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"),
                deployment_qm_hash: deployment_id.ipfs_hash(),
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
                deployment_qm_hash: deployment_id.ipfs_hash(),
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
                deployment_qm_hash: deployment_id.ipfs_hash(),
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
            GraphSubscriptionQueryResultRecord {
                query_id: String::from("4"),
                ticket_name: String::from("test_req_ticket__2"),
                ticket_user: String::from("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"),
                ticket_signer: String::from("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462"),
                deployment_qm_hash: String::from("QmWmyoMoctfbAaiEs2G46gpeUmhqFRDW6KWo64y5r581Vz"),
                subgraph_chain: Some(String::from("mainnet")),
                query_count: 100,
                status_code: QueryResultStatus::InternalError,
                status_message: String::from("success"),
                response_time_ms: 300,
                query_budget: 0.0003,
                indexer_fees: 0.0001,
                timestamp: timestamp_3,
                offset: 4,
                key: String::from("msg::4"),
            },
        ];
        let mut query_results_map =
            HashMap::<Address, Vec<GraphSubscriptionQueryResultRecord>>::new();
        query_results_map.insert(user, query_results);

        let mut expected = vec![
            RequestTicketSubgraphStat {
                subgraph_deployment_qm_hash: deployment_id,
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
                subgraph_deployment_qm_hash: deployment_id,
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

        let client = DatasourceInMemory::create_with_initial_data(query_results_map);

        let actual_no_filters = client
            .request_ticket_subgraph_stats(
                user,
                String::from("test_req_ticket__1"),
                deployment_id,
                None,
                None,
                None,
                None,
            )
            .await
            .unwrap_or_default();
        assert_eq!(actual_no_filters, expected);
        // should not return any stats for ticket: `test_req_ticket__2` || qm hash `QmWmyoMoctfbAaiEs2G46gpeUmhqFRDW6KWo64y5r581Vz`
        assert!(!actual_no_filters.into_iter().any(|stat| stat.ticket_name
            == String::from("test_req_ticket__2")
            || stat.subgraph_deployment_qm_hash
                == DeploymentId::from_ipfs_hash("QmWmyoMoctfbAaiEs2G46gpeUmhqFRDW6KWo64y5r581Vz")
                    .unwrap()));

        // only return 1
        let actual_first = client
            .request_ticket_subgraph_stats(
                user,
                String::from("test_req_ticket__1"),
                deployment_id,
                Some(1),
                None,
                None,
                None,
            )
            .await
            .unwrap_or_default();
        let expected_first = vec![RequestTicketSubgraphStat {
            subgraph_deployment_qm_hash: deployment_id,
            ticket_name: String::from("test_req_ticket__1"),
            ticket_user: Address::from_str("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462").unwrap(),
            ticket_signer: Address::from_str("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462").unwrap(),
            start: start_1,
            end: end_1,
            query_count: 2,
            avg_response_time_ms: (300 + 400) / 2 as u32,
            success_rate: 1.0,
            failed_query_count: 0,
        }];
        assert_eq!(actual_first.len(), 1);
        assert_eq!(actual_first, expected_first);

        // only return 1, but skip 1
        let actual_skip = client
            .request_ticket_subgraph_stats(
                user,
                String::from("test_req_ticket__1"),
                deployment_id,
                Some(1),
                Some(1),
                None,
                None,
            )
            .await
            .unwrap_or_default();
        let expected_skip = vec![RequestTicketSubgraphStat {
            subgraph_deployment_qm_hash: deployment_id,
            ticket_name: String::from("test_req_ticket__1"),
            ticket_user: Address::from_str("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462").unwrap(),
            ticket_signer: Address::from_str("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462").unwrap(),
            start: start_3,
            end: end_3,
            query_count: 100,
            avg_response_time_ms: 300,
            success_rate: 0.0,
            failed_query_count: 100,
        }];
        assert_eq!(actual_skip.len(), 1);
        assert_eq!(actual_skip, expected_skip);

        // order by ticket_name, desc
        let actual_ordered_desc = client
            .request_ticket_subgraph_stats(
                user,
                String::from("test_req_ticket__1"),
                deployment_id,
                None,
                None,
                Some(RequestTicketStatOrderBy::Start),
                Some(OrderDirection::Desc),
            )
            .await
            .unwrap_or_default();
        expected.sort_by(|a, b| a.start.cmp(&b.start).reverse());
        assert_eq!(actual_ordered_desc, expected);
    }
}
