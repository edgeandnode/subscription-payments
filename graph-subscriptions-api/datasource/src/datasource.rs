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
    /// Retrieve a list of request ticket stats, for all subgraph deployments, for the request ticket from the datasource.
    async fn request_ticket_stats(
        &self,
        user: Address,
        ticket_name: String,
        first: Option<i32>,
        skip: Option<i32>,
        order_by: Option<RequestTicketStatOrderBy>,
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
        order_by: Option<RequestTicketStatOrderBy>,
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
