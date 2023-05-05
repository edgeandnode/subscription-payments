use anyhow::Result;
use async_trait::async_trait;
use rdkafka::consumer::{DefaultConsumerContext, StreamConsumer};
use toolshed::bytes::{Address, DeploymentId};

use crate::models::*;

#[async_trait]
/// Define an extentable trait that defines common methods to retrieve request ticket, and stats, data.
pub trait Datasource {
    /// Retrieve a list of distinct request tickets for the given user address from the datasource.
    async fn request_tickets(
        &self,
        user: Address,
        first: Option<i32>,
        skip: Option<i32>,
        order_by: Option<RequestTicketOrderBy>,
        order_direction: Option<OrderDirection>,
    ) -> Result<Vec<RequestTicket>>;
    /// Retrieve a list of aggregated request ticket stats,
    /// for all request tickets (made unique by their `ticket_name` and `ticket_payload`),
    /// and across all Subgraph Deployments,
    /// for a `UserSubscription`.
    /// The stats are aggregated on a day-by-day (00:00:00 UTC - 23:59:59 UTC) basis,
    /// over the lifetime of the `UserSubscription`.
    ///
    /// # Arguments
    ///
    /// * `user` - [REQUIRED] the User address who owns the `UserSubscription` and who has been performing the queries with the genrated request tickets.
    /// * `start` - [OPTIONAL] lower-bound timeframe. if specified, returns Stats with a `start` value >= the given value
    /// * `end` - [OPTIONAL] upper-bound timeframe. if specified, returns Stats with a `end` value <= the given value
    /// * `order_by` - [OPTIONAL:default UserSubscriptionStatOrderBy::Start] what to order the stats by
    /// * `order_direction` - [OPTIONAL:default OrderDirection::ASC] direction to order the stats by
    async fn user_subscription_stats(
        &self,
        user: Address,
        start: Option<i64>,
        end: Option<i64>,
        order_by: Option<UserSubscriptionStatOrderBy>,
        order_direction: Option<OrderDirection>,
    ) -> Result<Vec<UserSubscriptionStat>>;
    /// Retrieve a list of request ticket stats, for all subgraph deployments, for the request ticket from the datasource.
    async fn request_ticket_stats(
        &self,
        user: Address,
        ticket_name: String,
        first: Option<i32>,
        skip: Option<i32>,
        order_by: Option<StatOrderBy>,
        order_direction: Option<OrderDirection>,
    ) -> Result<Vec<RequestTicketStat>>;
    /// Retrieve a list of request ticket stats, for a specific subgraph deployment, for the request ticket from the datasource.
    async fn request_ticket_subgraph_stats(
        &self,
        user: Address,
        ticket_name: String,
        subgraph_deployment_qm_hash: DeploymentId,
        first: Option<i32>,
        skip: Option<i32>,
        order_by: Option<StatOrderBy>,
        order_direction: Option<OrderDirection>,
    ) -> Result<Vec<RequestTicketSubgraphStat>>;
}

#[async_trait]
/// Define an extendable trait that exposes a write method that takes the received message
/// from the kafak StreamConsumer and stores the data in the sources data storage mechanism.
pub trait DatasourceWriter {
    /// Use the passed in reference to the `StreamConsumer` to stream/consume messages on the initialized consumer.
    async fn write(&self, consumer: &StreamConsumer<DefaultConsumerContext>);
}
