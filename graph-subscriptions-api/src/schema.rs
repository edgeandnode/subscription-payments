use std::{collections::HashSet, sync::Arc};

use anyhow::{Ok, Result};
use async_graphql::{Context, EmptyMutation, EmptySubscription, Enum, Object, Schema};
use chrono::Utc;
use datasource::{Datasource, DatasourceRedis};
use redis::JsonAsyncCommands as _;
use sha3::{
    digest::{ExtendableOutput, Update, XofReader},
    Shake256,
};
use tokio::sync::Mutex;
use toolshed::bytes::{Address, Bytes32, DeploymentId};

use crate::{
    auth::{AuthError, TicketPayloadWrapper},
    network_subgraph::{GraphAccount, Subgraph, SubgraphDeployments},
};

pub struct GraphSubscriptionsSchemaCtx<'a> {
    pub subgraph_deployments: SubgraphDeployments,
    pub datasource: &'a DatasourceRedis,
}

pub type GraphSubscriptionsSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

#[derive(Enum, Clone, Copy, PartialEq, Eq)]
pub enum OrderDirection {
    Asc,
    Desc,
}
impl Into<datasource::OrderDirection> for OrderDirection {
    fn into(self) -> datasource::OrderDirection {
        match self {
            Self::Asc => datasource::OrderDirection::Asc,
            Self::Desc => datasource::OrderDirection::Desc,
        }
    }
}

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

#[derive(Debug, PartialEq, Eq)]
pub struct RequestTicketDto {
    pub id: Bytes32,
    pub ticket_user: Address,
    pub ticket_signer: Address,
    pub ticket_name: String,
}
/// Rust does not let you define `impl` for structs outside of the package - which we need to do to implement the `async_graphql::Object` trait.
/// Since the `RequestTicket` returned by the `datasource` is external,
/// need to convert from the `datasource::RequestTicket` to `crate::schema::RequestTicket`
impl From<datasource::RequestTicket> for RequestTicketDto {
    fn from(value: datasource::RequestTicket) -> Self {
        let mut hasher = Shake256::default();
        hasher.update(value.ticket_user.0.as_slice());
        hasher.update(value.ticket_name.as_bytes());
        let mut reader = hasher.finalize_xof();
        let mut id_hashed: [u8; 32] = [0; 32];
        reader.read(&mut id_hashed);
        let id = Bytes32::from(id_hashed);
        Self {
            id,
            ticket_user: value.ticket_user,
            ticket_signer: value.ticket_signer,
            ticket_name: value.ticket_name,
        }
    }
}

#[Object]
/// The RequestTicketDto is a derived structure that represents fields derived from a user querying a Subgraph on The Graph Network.
/// After a user subscribes to The Graph Subscriptions Contract, they can then sign an EIP-712 domain message and use this to query Subgraphs on The Graph Network.
/// When a gateway receives the query, with this request ticket, it pushes data about the query to logs.
/// This api then queries the data from those logs to build this structure.
impl RequestTicketDto {
    async fn id(&self) -> String {
        self.id.to_string()
    }
    /// The wallet address of the user who owns the request ticket/signed the message
    async fn ticket_user(&self) -> String {
        self.ticket_user.to_string()
    }
    /// The wallet address of the user who signed the request ticket/signed the message
    async fn ticket_signer(&self) -> String {
        self.ticket_signer.to_string()
    }
    /// The user-chosen, friendly, name of the request ticket.
    /// This value is not stored on-chain. It is selected filled out by the user when they sign the EIP-712 message.
    async fn ticket_name(&self) -> String {
        self.ticket_name.to_string()
    }
    /// Count of all of the `Subgraphs` queried by the request ticket.
    async fn queried_subgraphs_count<'ctx>(&self, ctx: &Context<'ctx>) -> Result<u32> {
        let schema_ctx = ctx
            .data_unchecked::<Arc<Mutex<GraphSubscriptionsSchemaCtx>>>()
            .lock()
            .await;
        let mut conn = schema_ctx
            .datasource
            .redis_client
            .get_async_connection()
            .await?;
        let path = format!(
            "'$..{}[?(@.ticket_name==\"{}\")].deployment_qm_hash'",
            self.ticket_user.to_string().to_lowercase(),
            self.ticket_name.to_string()
        );
        let subgraph_deployment_qm_hashes: Vec<String> = conn
            .json_get(
                schema_ctx
                    .datasource
                    .graph_subscriptions_query_result_key
                    .to_string(),
                path,
            )
            .await?;
        if subgraph_deployment_qm_hashes.is_empty() {
            return Ok(0);
        }
        let unq_subgraph_deployment_qm_hashes = subgraph_deployment_qm_hashes
            .iter()
            .cloned()
            .collect::<HashSet<_>>();

        Ok(unq_subgraph_deployment_qm_hashes.len() as u32)
    }
    /// List of `Subgraph` records that this request ticket queried.
    async fn queried_subgraphs<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        _first: Option<i32>,
        _skip: Option<i32>,
    ) -> Result<Option<Vec<Subgraph>>> {
        let schema_ctx = ctx
            .data_unchecked::<Arc<Mutex<GraphSubscriptionsSchemaCtx>>>()
            .lock()
            .await;
        let mut conn = schema_ctx
            .datasource
            .redis_client
            .get_async_connection()
            .await?;
        let path = format!(
            "'$..{}[?(@.ticket_name==\"{}\")].deployment_qm_hash'",
            self.ticket_user.to_string().to_lowercase(),
            self.ticket_name.to_string()
        );
        let subgraph_deployment_qm_hashes: Vec<String> = conn
            .json_get(
                schema_ctx
                    .datasource
                    .graph_subscriptions_query_result_key
                    .to_string(),
                path,
            )
            .await?;
        if subgraph_deployment_qm_hashes.is_empty() {
            return Ok(Some(vec![]));
        }
        // TODO: FIGURE OUT HOW TO MAP THE QM HASHES TO A LIST OF SUBGRAPHS

        Ok(Some(vec![]))
    }
    /// Total count of queries performed, across all `Subgraphs`, using this request ticket
    async fn total_query_count<'ctx>(&self, ctx: &Context<'ctx>) -> Result<u32> {
        let schema_ctx = ctx
            .data_unchecked::<Arc<Mutex<GraphSubscriptionsSchemaCtx>>>()
            .lock()
            .await;
        let mut conn = schema_ctx
            .datasource
            .redis_client
            .get_async_connection()
            .await?;
        let path = format!(
            "'$..{}[?(@.ticket_name==\"{}\")].query_count'",
            self.ticket_user.to_string().to_lowercase(),
            self.ticket_name.to_string()
        );
        let results: Vec<u32> = conn
            .json_get(
                schema_ctx
                    .datasource
                    .graph_subscriptions_query_result_key
                    .to_string(),
                path,
            )
            .await?;
        if results.is_empty() {
            return Ok(0);
        }

        Ok(results.iter().sum())
    }
    /// Percentage of queries used for the user's active subscription.
    /// An active subscription stores the start and end block timestamp as well as a query rate that the user is paying for on-chain (in the Subscriptions contract).
    /// As the user queries `Subgraphs` using their request ticket, they "use up" part of their paid for rate (which is more of a way to rate-limit querying),
    /// in the given time-period.
    /// This value represents the percentage (from 0.00 -> 1.00) of the rate that has been used by the amount of queries made with the request ticket.
    async fn query_rate_used_percentage<'ctx>(&self, _ctx: &Context<'ctx>) -> f32 {
        // TODO: BUILD OUT VolumeEstimator LOGIC FROM gateway TO CALCULATE HOW MANY QUERIES AVAILABLE ON THE SUB
        0.00
    }
    /// Unix-timestamp of the last query performed using this request ticket
    async fn last_query_timestamp<'ctx>(&self, _ctx: &Context<'ctx>) -> i64 {
        Utc::now().timestamp()
    }
}

#[derive(Enum, Clone, Copy, PartialEq, Eq)]
pub enum RequestTicketOrderBy {
    Owner,
    Signer,
    Name,
}
impl Into<datasource::RequestTicketOrderBy> for RequestTicketOrderBy {
    fn into(self) -> datasource::RequestTicketOrderBy {
        match self {
            Self::Name => datasource::RequestTicketOrderBy::Name,
            Self::Signer => datasource::RequestTicketOrderBy::Signer,
            Self::Owner => datasource::RequestTicketOrderBy::Owner,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct RequestTicketStatDto {
    pub id: Bytes32,
    pub ticket_user: Address,
    pub ticket_signer: Address,
    pub ticket_name: String,
    pub start: i64,
    pub end: i64,
    pub total_query_count: u32,
    pub success_rate: f32,
    pub avg_response_time_ms: u32,
    pub failed_query_count: u32,
}

#[Object]
impl RequestTicketStatDto {
    async fn id(&self) -> String {
        self.id.to_string()
    }
    /// The Request Ticket Owner
    async fn ticket_user(&self) -> String {
        self.ticket_user.to_string()
    }
    /// The Request Ticket Signer
    async fn ticket_signer(&self) -> String {
        self.ticket_signer.to_string()
    }
    /// The start unix-timestamp date range of aggregated stats
    async fn start(&self) -> i64 {
        self.start
    }
    /// The end unix-timestamp date range of the aggregated stats
    async fn end(&self) -> i64 {
        self.end
    }
    /// The total count of queries received in the given date range using the RequestTicket
    async fn total_query_count(&self) -> u32 {
        self.total_query_count
    }
    /// Success rate, from 0.0 -> 1.0, of the number of queries that were returned to the caller successfully
    async fn success_rate(&self) -> f32 {
        self.success_rate
    }
    /// The average time, in ms, it took to return the query from the indexer to the caller
    async fn avg_response_time_ms(&self) -> u32 {
        self.avg_response_time_ms
    }
    /// A count of queries that did not return successfully to the caller.
    /// Whether because the query submitted by the user was invalid, there was an indexer error, or there was an internal error processing the query.
    async fn failed_query_count(&self) -> u32 {
        self.failed_query_count
    }
    /// List of `Subgraphs` queried by the request ticket
    async fn subgraphs<'ctx>(&self, _ctx: &Context<'ctx>) -> Option<Vec<Subgraph>> {
        None
    }
}
/// Convert the [`datasource::RequestTicketStat`] instance to a [`crate::schema::RequestTicketStatDto`] instance
impl From<datasource::RequestTicketStat> for RequestTicketStatDto {
    fn from(value: datasource::RequestTicketStat) -> Self {
        let mut hasher = Shake256::default();
        hasher.update(value.ticket_user.0.as_slice());
        hasher.update(value.ticket_name.as_bytes());
        hasher.update(&value.start.to_le_bytes());
        hasher.update(&value.end.to_le_bytes());
        let mut reader = hasher.finalize_xof();
        let mut id_hashed: [u8; 32] = [0; 32];
        reader.read(&mut id_hashed);
        let id = Bytes32::from(id_hashed);
        Self {
            id,
            ticket_user: value.ticket_user,
            ticket_signer: value.ticket_signer,
            ticket_name: value.ticket_name,
            start: value.start,
            end: value.end,
            total_query_count: value.query_count,
            success_rate: value.success_rate,
            avg_response_time_ms: value.avg_response_time_ms,
            failed_query_count: value.failed_query_count,
        }
    }
}

#[derive(Enum, Clone, Copy, PartialEq, Eq)]
pub enum RequestTicketStatOrderBy {
    Start,
    End,
    TotalQueryCount,
}
impl Into<datasource::RequestTicketStatOrderBy> for RequestTicketStatOrderBy {
    fn into(self) -> datasource::RequestTicketStatOrderBy {
        match self {
            Self::End => datasource::RequestTicketStatOrderBy::End,
            Self::Start => datasource::RequestTicketStatOrderBy::Start,
            Self::TotalQueryCount => datasource::RequestTicketStatOrderBy::TotalQueryCount,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct RequestTicketSubgraphStatDto {
    pub id: Bytes32,
    pub ticket_user: Address,
    pub ticket_signer: Address,
    pub ticket_name: String,
    pub start: i64,
    pub end: i64,
    pub total_query_count: u32,
    pub success_rate: f32,
    pub avg_response_time_ms: u32,
    pub failed_query_count: u32,
    pub subgraph_deployment_qm_hash: DeploymentId,
}
/// Convert the [`datasource::RequestTicketSubgraphStat`] instance to a [`crate::schema::RequestTicketSubgraphStatDto`] instance
impl From<datasource::RequestTicketSubgraphStat> for RequestTicketSubgraphStatDto {
    fn from(value: datasource::RequestTicketSubgraphStat) -> Self {
        let mut hasher = Shake256::default();
        hasher.update(value.ticket_user.0.as_slice());
        hasher.update(value.ticket_name.as_bytes());
        hasher.update(value.subgraph_deployment_qm_hash.0.as_slice());
        hasher.update(&value.start.to_le_bytes());
        hasher.update(&value.end.to_le_bytes());
        let mut reader = hasher.finalize_xof();
        let mut id_hashed: [u8; 32] = [0; 32];
        reader.read(&mut id_hashed);
        let id = Bytes32::from(id_hashed);
        Self {
            id,
            ticket_user: value.ticket_user,
            ticket_signer: value.ticket_signer,
            ticket_name: value.ticket_name,
            start: value.start,
            end: value.end,
            total_query_count: value.query_count,
            success_rate: value.success_rate,
            avg_response_time_ms: value.avg_response_time_ms,
            failed_query_count: value.failed_query_count,
            subgraph_deployment_qm_hash: value.subgraph_deployment_qm_hash,
        }
    }
}

#[Object]
impl RequestTicketSubgraphStatDto {
    async fn id(&self) -> String {
        self.id.to_string()
    }
    /// The Request Ticket Owner
    async fn ticket_user(&self) -> String {
        self.ticket_user.to_string()
    }
    /// The Request Ticket Signer
    async fn ticket_signer(&self) -> String {
        self.ticket_signer.to_string()
    }
    /// The start unix-timestamp date range of aggregated stats
    async fn start(&self) -> i64 {
        self.start
    }
    /// The end unix-timestamp date range of the aggregated stats
    async fn end(&self) -> i64 {
        self.end
    }
    /// The total count of queries received in the given date range using the RequestTicket
    async fn total_query_count(&self) -> u32 {
        self.total_query_count
    }
    /// Success rate, from 0.0 -> 1.0, of the number of queries that were returned to the caller successfully
    async fn success_rate(&self) -> f32 {
        self.success_rate
    }
    /// The average time, in ms, it took to return the query from the indexer to the caller
    async fn avg_response_time_ms(&self) -> u32 {
        self.avg_response_time_ms
    }
    /// A count of queries that did not return successfully to the caller.
    /// Whether because the query submitted by the user was invalid, there was an indexer error, or there was an internal error processing the query.
    async fn failed_query_count(&self) -> u32 {
        self.failed_query_count
    }
    /// The Subgraph Deployment Qm hash the user queried
    async fn subgraph_deployment_qm_hash(&self) -> String {
        self.subgraph_deployment_qm_hash.to_string()
    }
    /// List of `Subgraphs` associated to the `SubgraphDeployment` that the user queried
    async fn subgraphs<'ctx>(&self, ctx: &Context<'ctx>) -> Option<Vec<Subgraph>> {
        if self.subgraph_deployment_qm_hash.is_empty() {
            return None;
        }
        let schema_ctx = ctx.data_unchecked::<GraphSubscriptionsSchemaCtx>();
        schema_ctx
            .subgraph_deployments
            .deployment_subgraphs(&self.subgraph_deployment_qm_hash)
            .await
    }
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// A list of Request Tickets for the authenticated user, found by their wallet address (parsed from the Authorization header).
    async fn user_request_tickets<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        first: Option<i32>,
        skip: Option<i32>,
        order_by: Option<RequestTicketOrderBy>,
        order_direction: Option<OrderDirection>,
    ) -> Result<Vec<RequestTicketDto>> {
        let ticket_payload_wrapper = ctx.data_opt::<TicketPayloadWrapper>();
        if ticket_payload_wrapper.is_none() {
            return Err(AuthError::Unauthorized.into());
        }
        let ticket_payload = ticket_payload_wrapper.unwrap();
        let payload = &ticket_payload.ticket_payload;
        let user = Address(payload.user.unwrap_or(payload.signer).0);

        let schema_ctx = ctx
            .data_unchecked::<Arc<Mutex<GraphSubscriptionsSchemaCtx>>>()
            .lock()
            .await;

        let order_by: Option<datasource::RequestTicketOrderBy> = match order_by {
            None => None,
            Some(by) => Some(by.into()),
        };
        let order_direction: Option<datasource::OrderDirection> = match order_direction {
            None => None,
            Some(direction) => Some(direction.into()),
        };

        match schema_ctx
            .datasource
            .request_tickets(user, first, skip, order_by, order_direction)
            .await
        {
            Err(err) => Err(err),
            Result::Ok(tickets) => Ok(tickets.into_iter().map(RequestTicketDto::from).collect()),
        }
    }
    /// A list of aggregated query stats, by timerange, for the request ticket parsed from the Authorization header.
    async fn request_ticket_stats<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        first: Option<i32>,
        skip: Option<i32>,
        order_by: Option<RequestTicketStatOrderBy>,
        order_direction: Option<OrderDirection>,
    ) -> Result<Vec<RequestTicketStatDto>> {
        let ticket_payload_wrapper = ctx.data_opt::<TicketPayloadWrapper>();
        if ticket_payload_wrapper.is_none() {
            return Err(AuthError::Unauthorized.into());
        }
        let ticket_payload = ticket_payload_wrapper.unwrap();
        let payload = &ticket_payload.ticket_payload;
        let user = Address(payload.user.unwrap_or(payload.signer).0);
        let ticket_name = match &payload.name {
            None => return Err(anyhow::Error::msg("ticket_name is required")),
            Some(name) => name,
        };

        let schema_ctx = ctx
            .data_unchecked::<Arc<Mutex<GraphSubscriptionsSchemaCtx>>>()
            .lock()
            .await;

        let order_by: Option<datasource::RequestTicketStatOrderBy> = match order_by {
            None => None,
            Some(by) => Some(by.into()),
        };
        let order_direction: Option<datasource::OrderDirection> = match order_direction {
            None => None,
            Some(direction) => Some(direction.into()),
        };

        match schema_ctx
            .datasource
            .request_ticket_stats(
                user,
                ticket_name.to_string(),
                first,
                skip,
                order_by,
                order_direction,
            )
            .await
        {
            Err(err) => Err(err),
            Result::Ok(tickets) => Ok(tickets
                .into_iter()
                .map(RequestTicketStatDto::from)
                .collect()),
        }
    }
    /// A list of aggregated query stats, by timerange, for a specific subgraph deployment, for the request ticket parsed from the Authorization header.
    async fn request_ticket_subgraph_stats<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        subgraph_deployment_qm_hash: String,
        first: Option<i32>,
        skip: Option<i32>,
        order_by: Option<RequestTicketStatOrderBy>,
        order_direction: Option<OrderDirection>,
    ) -> Result<Vec<RequestTicketSubgraphStatDto>> {
        let ticket_payload_wrapper = ctx.data_opt::<TicketPayloadWrapper>();
        if ticket_payload_wrapper.is_none() {
            return Err(AuthError::Unauthorized.into());
        }
        let ticket_payload = ticket_payload_wrapper.unwrap();
        let payload = &ticket_payload.ticket_payload;
        let user = Address(payload.user.unwrap_or(payload.signer).0);
        let ticket_name = match &payload.name {
            None => return Err(anyhow::Error::msg("ticket_name is required")),
            Some(name) => name,
        };
        let subgraph_deployment_id =
            match DeploymentId::from_ipfs_hash(&subgraph_deployment_qm_hash) {
                None => return Err(anyhow::Error::msg(
                    "the `subgraph_deployment_qm_hash` is not a valid subgraph deployment Qm hash",
                )),
                Some(hash) => hash,
            };

        let schema_ctx = ctx
            .data_unchecked::<Arc<Mutex<GraphSubscriptionsSchemaCtx>>>()
            .lock()
            .await;

        let order_by: Option<datasource::RequestTicketStatOrderBy> = match order_by {
            None => None,
            Some(by) => Some(by.into()),
        };
        let order_direction: Option<datasource::OrderDirection> = match order_direction {
            None => None,
            Some(direction) => Some(direction.into()),
        };

        match schema_ctx
            .datasource
            .request_ticket_subgraph_stats(
                user,
                ticket_name.to_string(),
                subgraph_deployment_id,
                first,
                skip,
                order_by,
                order_direction,
            )
            .await
        {
            Err(err) => Err(err),
            Result::Ok(tickets) => Ok(tickets
                .into_iter()
                .map(RequestTicketSubgraphStatDto::from)
                .collect()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use sha3::{
        digest::{ExtendableOutput, Update, XofReader},
        Shake256,
    };

    use super::*;

    #[test]
    fn should_convert_datasource_request_ticket_to_schema_type() {
        let given = datasource::RequestTicket {
            ticket_name: String::from("test_req_ticket__1"),
            ticket_user: Address::from_str("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462").unwrap(),
            ticket_signer: Address::from_str("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462").unwrap(),
        };

        let mut hasher = Shake256::default();
        hasher.update(given.ticket_user.0.as_slice());
        hasher.update(given.ticket_name.as_bytes());
        let mut reader = hasher.finalize_xof();
        let mut id_hashed: [u8; 32] = [0; 32];
        reader.read(&mut id_hashed);
        let expected_id = Bytes32::from(id_hashed);
        let expected = RequestTicketDto {
            id: expected_id,
            ticket_user: given.ticket_user,
            ticket_signer: given.ticket_signer,
            ticket_name: given.ticket_name.to_string(),
        };

        let actual = RequestTicketDto::from(given);

        assert_eq!(actual, expected);
    }

    #[test]
    fn should_convert_datasource_request_ticket_stat_to_schema_type() {
        let given = datasource::RequestTicketStat {
            ticket_name: String::from("test_req_ticket__1"),
            ticket_user: Address::from_str("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462").unwrap(),
            ticket_signer: Address::from_str("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462").unwrap(),
            start: 1679791065,
            end: 1679791066,
            query_count: 2,
            avg_response_time_ms: (300 + 400) / 2 as u32,
            success_rate: 1.0,
            failed_query_count: 0,
        };
        let mut hasher = Shake256::default();
        hasher.update(given.ticket_user.0.as_slice());
        hasher.update(given.ticket_name.as_bytes());
        hasher.update(&given.start.to_le_bytes());
        hasher.update(&given.end.to_le_bytes());
        let mut reader = hasher.finalize_xof();
        let mut id_hashed: [u8; 32] = [0; 32];
        reader.read(&mut id_hashed);
        let expected_id = Bytes32::from(id_hashed);
        let expected = RequestTicketStatDto {
            id: expected_id,
            ticket_name: given.ticket_name.to_string(),
            ticket_user: given.ticket_user,
            ticket_signer: given.ticket_signer,
            start: given.start,
            end: given.end,
            total_query_count: given.query_count,
            success_rate: given.success_rate,
            avg_response_time_ms: given.avg_response_time_ms,
            failed_query_count: given.failed_query_count,
        };

        let actual = RequestTicketStatDto::from(given);

        assert_eq!(actual, expected);
    }

    #[test]
    fn should_convert_datasource_request_ticket_subgraph_stat_to_schema_type() {
        let given = datasource::RequestTicketSubgraphStat {
            ticket_name: String::from("test_req_ticket__1"),
            ticket_user: Address::from_str("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462").unwrap(),
            ticket_signer: Address::from_str("0xa476caFd8b08F11179BDDd5145FcF3EF470C7462").unwrap(),
            subgraph_deployment_qm_hash: DeploymentId::from_str(
                "Qmadj8x9km1YEyKmRnJ6EkC2zpJZFCfTyTZpuqC3j6e1QH",
            )
            .unwrap(),
            start: 1679791065,
            end: 1679791066,
            query_count: 2,
            avg_response_time_ms: (300 + 400) / 2 as u32,
            success_rate: 1.0,
            failed_query_count: 0,
        };
        let mut hasher = Shake256::default();
        hasher.update(given.ticket_user.0.as_slice());
        hasher.update(given.ticket_name.as_bytes());
        hasher.update(given.subgraph_deployment_qm_hash.0.as_slice());
        hasher.update(&given.start.to_le_bytes());
        hasher.update(&given.end.to_le_bytes());
        let mut reader = hasher.finalize_xof();
        let mut id_hashed: [u8; 32] = [0; 32];
        reader.read(&mut id_hashed);
        let expected_id = Bytes32::from(id_hashed);
        let expected = RequestTicketSubgraphStatDto {
            id: expected_id,
            ticket_name: given.ticket_name.to_string(),
            ticket_user: given.ticket_user,
            ticket_signer: given.ticket_signer,
            subgraph_deployment_qm_hash: given.subgraph_deployment_qm_hash,
            start: given.start,
            end: given.end,
            total_query_count: given.query_count,
            success_rate: given.success_rate,
            avg_response_time_ms: given.avg_response_time_ms,
            failed_query_count: given.failed_query_count,
        };

        let actual = RequestTicketSubgraphStatDto::from(given);

        assert_eq!(actual, expected);
    }
}
