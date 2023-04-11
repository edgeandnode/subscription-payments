use std::sync::Arc;

use anyhow::{Ok, Result};
use async_trait::async_trait;
use chrono::Utc;
use futures::TryStreamExt;
use rdkafka::consumer::DefaultConsumerContext;
use rdkafka::error::KafkaError;
use rdkafka::{consumer::StreamConsumer, Message};
use redis::JsonAsyncCommands as _;
use serde_json::json;
use tokio::sync::Mutex;
use toolshed::bytes::{Address, DeploymentId};

use crate::datasource::{Datasource, DatasourceWriter};
use crate::models::*;

/// The redis datasource implements both the `Datasource` and `DatasourceWriter` traits.
/// Allows a user to query and store `GatewaySubscriptionQueryResult` records stored in the redis datasource instance.
pub struct DatasourceRedis {
    pub redis_client: redis::Client,
    pub graph_subscriptions_query_result_key: String,
}

impl DatasourceRedis {
    pub fn create(addr: &str) -> &'static Self {
        let client =
            redis::Client::open(addr).expect("Failure establishing redis client connection");

        Box::leak(Box::new(Self {
            redis_client: client,
            graph_subscriptions_query_result_key: String::from(
                "GATEWAY_SUBSCRIPTION_QUERY_RESULTS",
            ),
        }))
    }
}

#[async_trait]
impl Datasource for DatasourceRedis {
    /// Retrieve the user's unique `RequestTicket` records derived from the stored query result records from the redis database.
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
        let mut conn = self.redis_client.get_async_connection().await?;

        let path = format!("$.{}.*", user.to_string().to_lowercase());
        let results: Vec<GraphSubscriptionQueryResultRecord> = conn
            .json_get(&self.graph_subscriptions_query_result_key, path)
            .await?;
        if results.is_empty() {
            return Ok(vec![]);
        }
        let mut tickets = RequestTicket::build_unique_request_ticket_list(results);
        if let Some(order) = order_by {
            let direction = order_direction.unwrap_or(OrderDirection::Asc);
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

    /// Retrieve the user's `RequestTicketStat` records, aggregated over the given timeframe, derived from the stored query result records from the redis database.
    ///
    /// # Arguments
    ///
    /// - `user` - the user wallet address who performed the stored queries
    /// - `ticket_name` - the name of the request ticket to get stats for
    /// - `first` - [OPTIONAL:default 100] the number of records, after sorting, to return
    /// - `skip` - [OPTIONAL:default 0] the number of records, after sorting, to skip
    /// - `order_by` - [OPTIONAL] what field on the `RequestTicketStat` to sort by
    /// - `order_direction` [OPTIONAL] the sort direction
    async fn request_ticket_stats(
        &self,
        user: Address,
        ticket_name: String,
        first: Option<i32>,
        skip: Option<i32>,
        order_by: Option<RequestTicketStatOrderBy>,
        order_direction: Option<OrderDirection>,
    ) -> Result<Vec<RequestTicketStat>> {
        let mut conn = self.redis_client.get_async_connection().await?;

        // build a path to grab all records in the `{user}` key in the redis JSON storage object: `{[user: Address]: Vec<GraphSubscriptionQueryResultRecord>}`,
        // then filters where the `GraphSubscriptionQueryResultRecord.ticket_name` matches the passed value.
        // ex: `'$..0x0000000000000000000000000000000000000000[?(@.ticket_name=="test_req_ticket__1")]'`
        let path = format!(
            "'$..{}[?(@.ticket_name==\"{}\")]'",
            user.to_string().to_lowercase(),
            ticket_name
        );
        let results: Vec<GraphSubscriptionQueryResultRecord> = conn
            .json_get(&self.graph_subscriptions_query_result_key, path)
            .await?;
        if results.is_empty() {
            return Ok(vec![]);
        }

        let mut stats = RequestTicketStat::from_graph_subscription_query_result_records(results);
        if let Some(order) = order_by {
            let direction = order_direction.unwrap_or(OrderDirection::Asc);
            stats.sort_by(|a, b| match order {
                RequestTicketStatOrderBy::Start => {
                    if direction == OrderDirection::Asc {
                        a.start.cmp(&b.start)
                    } else {
                        a.start.cmp(&b.start).reverse()
                    }
                }
                RequestTicketStatOrderBy::End => {
                    if direction == OrderDirection::Asc {
                        a.end.cmp(&b.end)
                    } else {
                        a.end.cmp(&b.end).reverse()
                    }
                }
                RequestTicketStatOrderBy::TotalQueryCount => {
                    if direction == OrderDirection::Asc {
                        a.query_count.cmp(&b.query_count)
                    } else {
                        a.query_count.cmp(&b.query_count).reverse()
                    }
                }
            })
        }
        let take = first.unwrap_or(100) as usize;
        let skip = skip.unwrap_or(0) as usize;

        Ok(stats.into_iter().skip(skip).take(take).collect())
    }

    /// Retrieve the user's `RequestTicketSubgraphStat` records, aggregated over the given timeframe, for a specific subgraph deployment Qm hash, derived from the stored query result records from the redis database.
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
        let mut conn = self.redis_client.get_async_connection().await?;

        // build a path to grab all records in the `{user}` key in the redis JSON storage object: `{[user: Address]: Vec<GraphSubscriptionQueryResultRecord>}`,
        // then filters where the `GraphSubscriptionQueryResultRecord.ticket_name` matches the passed value.
        // ex: `'$..0x0000000000000000000000000000000000000000[?(@.ticket_name=="test_req_ticket__1"&&@.deployment_qm_hash=="Qmadj8x9km1YEyKmRnJ6EkC2zpJZFCfTyTZpuqC3j6e1QH")]'`
        let path = format!(
            "'$..{}[?(@.ticket_name==\"{}\"&&@.deployment_qm_hash==\"{}\")]'",
            user.to_string().to_lowercase(),
            ticket_name,
            subgraph_deployment_qm_hash.ipfs_hash()
        );
        let results: Vec<GraphSubscriptionQueryResultRecord> = conn
            .json_get(&self.graph_subscriptions_query_result_key, path)
            .await?;
        if results.is_empty() {
            return Ok(vec![]);
        }

        let mut stats =
            RequestTicketSubgraphStat::from_graph_subscription_query_result_records(results);
        if let Some(order) = order_by {
            let direction = order_direction.unwrap_or(OrderDirection::Asc);
            stats.sort_by(|a, b| match order {
                RequestTicketStatOrderBy::Start => {
                    if direction == OrderDirection::Asc {
                        a.start.cmp(&b.start)
                    } else {
                        a.start.cmp(&b.start).reverse()
                    }
                }
                RequestTicketStatOrderBy::End => {
                    if direction == OrderDirection::Asc {
                        a.end.cmp(&b.end)
                    } else {
                        a.end.cmp(&b.end).reverse()
                    }
                }
                RequestTicketStatOrderBy::TotalQueryCount => {
                    if direction == OrderDirection::Asc {
                        a.query_count.cmp(&b.query_count)
                    } else {
                        a.query_count.cmp(&b.query_count).reverse()
                    }
                }
            })
        }
        let take = first.unwrap_or(100) as usize;
        let skip = skip.unwrap_or(0) as usize;

        Ok(stats.into_iter().skip(skip).take(take).collect())
    }
}

#[async_trait]
impl DatasourceWriter for DatasourceRedis {
    async fn write(&self, consumer: &StreamConsumer<DefaultConsumerContext>) {
        let conn = Arc::new(Mutex::new(
            self.redis_client
                .get_async_connection()
                .await
                .expect("Did not successfully instantiate the redis client connection"),
        ));

        let stream_processor = consumer.stream().try_for_each(move |borrowed_msg| {
            let conn = conn.clone();
            async move {
                let mut conn = conn.lock().await;
                let msg = borrowed_msg.detach();
                // convert the `OwnedMessage`
                let query_result_msg = match GatewaySubscriptionQueryResult::from_slice(
                    msg.payload().unwrap_or_default(),
                ) {
                    Err(err) => {
                        tracing::warn!("DatasourceRedis.store_subscription_query_result_record()::cannot deserialize message. skipping offset: [{}]. {}", msg.offset(), err);
                        return Result::<(), KafkaError>::Ok(());
                    }
                    std::result::Result::Ok(payload) => payload,
                };

                let query_path = format!("$.{}.*", query_result_msg.ticket_user.to_lowercase());
                let existing: Vec<GraphSubscriptionQueryResultRecord> = conn
                    .json_get(&self.graph_subscriptions_query_result_key, query_path)
                    .await.map_err(|_| KafkaError::MessageConsumption(rdkafka::types::RDKafkaErrorCode::BadMessage))?;

                let path = format!("$.{}", query_result_msg.ticket_user.to_lowercase());

                // build a `GraphSubscriptionQueryResultRecord`
                let timestamp = msg
                    .timestamp()
                    .to_millis()
                    .map(|ms| ms / 1000)
                    .unwrap_or(Utc::now().timestamp());
                let offset = msg.offset();
                let key = String::from_utf8_lossy(msg.key().unwrap_or_default()).to_string();
                let record = GraphSubscriptionQueryResultRecord::from_query_result_msg(
                    query_result_msg,
                    timestamp,
                    offset,
                    key,
                );

                if existing.is_empty() {
                    // set the initial value in the map for the user address
                    conn.json_set(
                        &self.graph_subscriptions_query_result_key,
                        path,
                        &json!(vec![record]),
                    )
                    .await.map_err(|_| KafkaError::MessageConsumption(rdkafka::types::RDKafkaErrorCode::BadMessage))?;
                } else {
                    // append the object to the existing array
                    conn.json_arr_append(
                        &self.graph_subscriptions_query_result_key,
                        path,
                        &json!(record),
                    )
                    .await.map_err(|_| KafkaError::MessageConsumption(rdkafka::types::RDKafkaErrorCode::BadMessage))?;
                }

                Result::<(), KafkaError>::Ok(())
            }
        });

        tracing::info!(
            "DatasourceRedis.write()::initializing message stream consumer processing..."
        );
        stream_processor
            .await
            .expect("DatasourceRedis.write()::failure processing the stream messages");
        tracing::info!("DatasourceRedis.write()::message stream consumer terminated");
    }
}
