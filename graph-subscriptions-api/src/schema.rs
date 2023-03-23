use anyhow::{Ok, Result};
use async_graphql::{Context, EmptyMutation, EmptySubscription, Object, Schema};
use chrono::Utc;
use toolshed::bytes::{Address, Bytes32};

use crate::network_subgraph::{GraphAccount, Subgraph, SubgraphDeployments};

pub struct GraphSubscriptionsSchemaCtx {
    pub subgraph_deployments: SubgraphDeployments,
}

pub type GraphSubscriptionsSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

#[Object]
impl GraphAccount {
    async fn id(&self) -> String {
        self.id.to_string()
    }
    async fn image(&self) -> &Option<String> {
        &self.image
    }
    async fn default_display_name(&self) -> &Option<String> {
        &self.default_display_name
    }
}

#[Object]
impl Subgraph {
    async fn id(&self) -> String {
        self.id.to_string()
    }
    async fn display_name(&self) -> &Option<String> {
        &self.display_name
    }
    async fn image(&self) -> &Option<String> {
        &self.image
    }
    async fn owner(&self) -> &GraphAccount {
        &self.owner
    }
}

pub struct RequestTicket {
    pub id: Bytes32,
    pub owner: Address,
    pub display_name: String,
}

#[Object]
/// The RequestTicket is a derived structure that represents fields derived from a user querying a Subgraph on The Graph Network.
/// After a user subscribes to The Graph Subscriptions Contract, they can then sign an EIP-712 domain message and use this to query Subgraphs on The Graph Network.
/// When a gateway receives the query, with this request ticket, it pushes data about the query to logs.
/// This api then queries the data from those logs to build this structure.
impl RequestTicket {
    /// Generated Bytes32 ID from the: {user wallet address}:{request ticket name}
    async fn id(&self) -> String {
        self.id.to_string()
    }
    /// The wallet address of the user who owns the request ticket/signed the message
    async fn owner(&self) -> String {
        self.owner.to_string()
    }
    /// The user-chosen, friendly, name of the request ticket.
    /// This value is not stored on-chain. It is selected filled out by the user when they sign the EIP-712 message.
    async fn display_name(&self) -> &String {
        &self.display_name
    }
    /// Count of all of the `Subgraphs` queried by the request ticket.
    async fn queried_subgraphs_count<'ctx>(&self, _ctx: &Context<'ctx>) -> i64 {
        0
    }
    /// List of `Subgraph` records that this request ticket queried.
    async fn queried_subgraphs<'ctx>(
        &self,
        _ctx: &Context<'ctx>,
        _first: Option<i32>,
        _skip: Option<i32>,
    ) -> Option<Vec<Subgraph>> {
        None
    }
    /// Total count of queries performed, across all `Subgraphs`, using this request ticket
    async fn total_query_count<'ctx>(&self, _ctx: &Context<'ctx>) -> u64 {
        0
    }
    /// Percentage of queries used for the user's active subscription.
    /// An active subscription stores the start and end block timestamp as well as a query rate that the user is paying for on-chain (in the Subscriptions contract).
    /// As the user queries `Subgraphs` using their request ticket, they "use up" part of their paid for rate (which is more of a way to rate-limit querying),
    /// in the given time-period.
    /// This value represents the percentage (from 0.00 -> 1.00) of the rate that has been used by the amount of queries made with the request ticket.
    async fn query_rate_used_percentage<'ctx>(&self, _ctx: &Context<'ctx>) -> f32 {
        0.00
    }
    /// Unix-timestamp of the last query performed using this request ticket
    async fn last_query_timestamp<'ctx>(&self, _ctx: &Context<'ctx>) -> i64 {
        Utc::now().timestamp()
    }
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn user_request_tickets<'ctx>(&self, _ctx: &Context<'ctx>) -> Result<Vec<RequestTicket>> {
        Ok(vec![])
    }
}
