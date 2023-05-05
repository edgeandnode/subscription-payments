use std::{str::FromStr, time::Duration};

use async_trait::async_trait;
use chrono::Utc;
use futures::TryStreamExt;
use graph_subscriptions::TicketPayload;
use migration::MigratorTrait;
use rdkafka::{
    consumer::{DefaultConsumerContext, StreamConsumer},
    error::KafkaError,
    Message,
};
use sea_orm::{
    prelude::Uuid, ActiveModelTrait, ConnectOptions, Database, DatabaseConnection, FromQueryResult,
    JsonValue, Set, Statement, Value,
};
use serde_json::json;
use toolshed::bytes::{Address, DeploymentId};

use crate::{utils::build_timerange_timestamp, *};

impl FromQueryResult for UniqRequestTicketDeploymentQmHash {
    fn from_query_result(
        res: &sea_orm::QueryResult,
        pre: &str,
    ) -> std::result::Result<Self, migration::DbErr> {
        let deployment_id = res
            .try_get::<String>(pre, "deployment_qm_hash")
            .map(|val| DeploymentId::from_ipfs_hash(&val))?;
        Result::Ok(Self {
            deployment_qm_hash: deployment_id.unwrap_or_default(),
        })
    }
}

impl FromQueryResult for RequestTicket {
    fn from_query_result(res: &sea_orm::QueryResult, pre: &str) -> Result<Self, migration::DbErr> {
        let ticket_user = res
            .try_get::<String>(pre, "ticket_user")
            .map(|val| Address::from_str(&val))?;
        let ticket_user = ticket_user.map_err(|err| migration::DbErr::Custom(err.to_string()))?;
        // ticket_payload comes as a JSON array value.
        // parse the value, grab the first item.
        let ticket_payload_json = res.try_get::<JsonValue>(pre, "ticket_payload")?;
        let ticket_payload = serde_json::from_value::<TicketPayload>(ticket_payload_json)
            .map_err(|err| sea_orm::TryGetError::DbErr(migration::DbErr::Json(err.to_string())))?;

        Result::Ok(Self {
            ticket_name: res.try_get(pre, "ticket_name")?,
            ticket_user,
            total_query_count: res.try_get(pre, "total_query_count")?,
            queried_subgraphs_count: res.try_get(pre, "queried_subgraphs_count")?,
            last_query_timestamp: res.try_get(pre, "last_query_timestamp")?,
            ticket_payload,
        })
    }
}

impl FromQueryResult for UserSubscriptionStat {
    fn from_query_result(
        res: &sea_orm::QueryResult,
        pre: &str,
    ) -> std::result::Result<Self, sea_orm::DbErr> {
        let ticket_user = res
            .try_get::<String>(pre, "ticket_user")
            .map(|val| Address::from_str(&val))?;
        Result::Ok(Self {
            ticket_user: ticket_user.map_err(|err| migration::DbErr::Custom(err.to_string()))?,
            start: res.try_get(pre, "timeframe_start_timestamp")?,
            end: res.try_get(pre, "timeframe_end_timestamp")?,
            query_count: res.try_get(pre, "query_count")?,
            success_rate: res.try_get(pre, "success_rate")?,
            avg_response_time_ms: res.try_get(pre, "avg_response_time_ms")?,
            failed_query_count: res.try_get(pre, "failed_query_count")?,
        })
    }
}

impl FromQueryResult for RequestTicketStat {
    fn from_query_result(
        res: &sea_orm::QueryResult,
        pre: &str,
    ) -> std::result::Result<Self, migration::DbErr> {
        let ticket_user = res
            .try_get::<String>(pre, "ticket_user")
            .map(|val| Address::from_str(&val))?;
        Result::Ok(Self {
            ticket_name: res.try_get(pre, "ticket_name")?,
            ticket_user: ticket_user.map_err(|err| migration::DbErr::Custom(err.to_string()))?,
            start: res.try_get(pre, "timeframe_start_timestamp")?,
            end: res.try_get(pre, "timeframe_end_timestamp")?,
            query_count: res.try_get(pre, "query_count")?,
            success_rate: res.try_get(pre, "success_rate")?,
            avg_response_time_ms: res.try_get(pre, "avg_response_time_ms")?,
            failed_query_count: res.try_get(pre, "failed_query_count")?,
            queried_subgraphs_count: res.try_get(pre, "queried_subgraphs_count")?,
        })
    }
}

impl FromQueryResult for RequestTicketSubgraphStat {
    fn from_query_result(
        res: &sea_orm::QueryResult,
        pre: &str,
    ) -> std::result::Result<Self, migration::DbErr> {
        let deployment_id = res
            .try_get::<String>(pre, "deployment_qm_hash")
            .map(|val| DeploymentId::from_ipfs_hash(&val))?;
        let ticket_user = res
            .try_get::<String>(pre, "ticket_user")
            .map(|val| Address::from_str(&val))?;
        Result::Ok(Self {
            subgraph_deployment_qm_hash: deployment_id.unwrap_or_default(),
            ticket_name: res.try_get(pre, "ticket_name")?,
            ticket_user: ticket_user.map_err(|err| migration::DbErr::Custom(err.to_string()))?,
            start: res.try_get(pre, "timeframe_start_timestamp")?,
            end: res.try_get(pre, "timeframe_end_timestamp")?,
            query_count: res.try_get(pre, "query_count")?,
            success_rate: res.try_get(pre, "success_rate")?,
            avg_response_time_ms: res.try_get(pre, "avg_response_time_ms")?,
            failed_query_count: res.try_get(pre, "failed_query_count")?,
        })
    }
}

/// The rdbms (this implementation uses postgres) datasource implements both the `Datasource` and `DatasourceWriter` traits.
/// Allows a user to query and store `GatewaySubscriptionQueryResult` records stored in the postgres database instance.
pub struct DatasourcePostgres {
    pub db_conn: DatabaseConnection,
}
impl DatasourcePostgres {
    /// Create a DatasourcePostgres instance by instantiating a postgres db connection.
    ///
    /// # Arguments
    ///
    /// * `db_url` - the postgres connection url. ex: postgress://username:password@host:port/database
    pub async fn create(db_url: String) -> Result<&'static Self, sea_orm::DbErr> {
        let mut opt = ConnectOptions::new(db_url);
        opt.max_connections(100)
            .min_connections(5)
            .connect_timeout(Duration::from_secs(30))
            .acquire_timeout(Duration::from_secs(10))
            .idle_timeout(Duration::from_secs(10))
            .max_lifetime(Duration::from_secs(10))
            .sqlx_logging(true)
            .sqlx_logging_level(tracing::log::LevelFilter::Info);
        let db_conn = Database::connect(opt).await.map_err(sea_orm::DbErr::from)?;
        // run the migrations on the instance
        migration::Migrator::up(&db_conn, None).await?;

        Result::Ok(Box::leak(Box::new(Self { db_conn })))
    }
    /// Get a list of unique deployment qm hashses from the database for the request ticket
    pub async fn uniq_deployments_for_ticket(
        &self,
        user: Address,
        ticket_name: String,
    ) -> anyhow::Result<Vec<UniqRequestTicketDeploymentQmHash>> {
        UniqRequestTicketDeploymentQmHash::find_by_statement(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"SELECT DISTINCT deployment_qm_hash FROM subscription_query_result WHERE ticket_user = $1 AND ticket_name = $2"#,
            [user.to_string().to_lowercase().into(), ticket_name.into()],
        ))
        .all(&self.db_conn)
        .await
        .map_err(|err| anyhow::Error::from(err))
    }
    /// Check if the user has "access" to the ticket.
    /// Access is determined if a record exists in the logs db matching the `ticket_user` and `ticket_name`
    pub async fn user_has_ticket_access(
        &self,
        user: Address,
        ticket_name: String,
    ) -> anyhow::Result<bool> {
        let result = UserHasTicketResult::find_by_statement(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"SELECT CASE WHEN COUNT(id) >= 1 THEN true ELSE false END AS user_has_ticket FROM subscription_query_result WHERE ticket_user = $1 AND ticket_name = $2"#,
            [user.to_string().to_lowercase().into(), ticket_name.into()],
        )).one(&self.db_conn)
        .await
        .map_err(|err| anyhow::Error::from(err))?
        .unwrap_or_default();

        anyhow::Result::Ok(result.user_has_ticket)
    }
}

#[async_trait]
impl Datasource for DatasourcePostgres {
    /// Retrieve the user's unique `RequestTicket` records derived from the stored query result records from the postgres database.
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
    ) -> anyhow::Result<Vec<RequestTicket>> {
        let order_by = order_by.unwrap_or(RequestTicketOrderBy::Name);
        let order_direction = order_direction.unwrap_or(OrderDirection::Asc);
        let limit = first.unwrap_or(100);
        let offset = skip.unwrap_or(0);

        // **NOTE**: `COALESCE(JSON_AGG(result.ticket_payload) ->> (JSON_ARRAY_LENGTH(JSON_AGG(result.ticket_payload)) - 1), '{}')::JSON`
        // The `ticket_payload` is a JSON object stored in the DB.
        // This select aggregates all of the `result.ticket_payload` data into a JSON array,
        // because this value could potentially become _very, very_ large,
        // we want to just return the last item from the array.
        // For most use-cases, this is fine as each entry in the array will be the same,
        // and this value is used to "reconstruct" the ticket in a UI to let a user resign the
        // request ticket message with the same domain to get the same value.
        // So using the last item means, if the user updates their security settings,
        // this will return the latest of their updates.
        RequestTicket::find_by_statement(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"
            WITH queried_subgraphs_count (ticket, queried_subgraphs_count) AS (
                SELECT CONCAT(ticket_name, ticket_user), COUNT(DISTINCT deployment_qm_hash)
                FROM subscription_query_result
                GROUP BY CONCAT(ticket_name, ticket_user)
            )
            SELECT
                result.ticket_name,
                result.ticket_user,
                COALESCE(JSON_AGG(result.ticket_payload) ->> (JSON_ARRAY_LENGTH(JSON_AGG(result.ticket_payload)) - 1), '{}')::JSON AS ticket_payload,
                CAST(SUM(result.query_count) AS bigint) as total_query_count,
                MAX(queried_subgraphs_count.queried_subgraphs_count) AS queried_subgraphs_count,
                MAX(message_timestamp) AS last_query_timestamp
            FROM subscription_query_result AS result
            JOIN queried_subgraphs_count
                ON queried_subgraphs_count.ticket = CONCAT(result.ticket_name, result.ticket_user)
            WHERE ticket_user = $1 AND ticket_name IS NOT NULL
            GROUP BY result.ticket_name, result.ticket_user
            ORDER BY $2
            LIMIT $3
            OFFSET $4
            "#,
            [
                user.to_string().into(),
                format!("{} {}", order_by.as_str(), order_direction.as_str()).into(),
                limit.into(),
                offset.into(),
            ],
        ))
        .all(&self.db_conn)
        .await
        .map_err(|err| anyhow::Error::from(err))
    }

    /// Retrieve the user's unique `UserSubscriptionStat` records derived from the stored query result records from the postgres database.
    ///
    /// # Arguments
    ///
    /// * `user` - [REQUIRED] the User address who owns the `UserSubscription` and who has been performing the queries with the genrated request tickets.
    /// * `start` - [OPTIONAL] lower-bound timeframe. if specified, returns Stats with a `start` value >= the given value
    /// * `end` - [OPTIONAL] upper-bound timeframe. if specified, returns Stats with a `end` value <= the given value
    /// * `order_by` - [OPTIONAL:default StatOrderBy::Start] what to order the stats by
    /// * `order_direction` - [OPTIONAL:default OrderDirection::ASC] direction to order the stats by
    async fn user_subscription_stats(
        &self,
        user: Address,
        start: Option<i64>,
        end: Option<i64>,
        order_by: Option<UserSubscriptionStatOrderBy>,
        order_direction: Option<OrderDirection>,
    ) -> Result<Vec<UserSubscriptionStat>> {
        let order_by = order_by.unwrap_or(UserSubscriptionStatOrderBy::Start);
        let order_direction = order_direction.unwrap_or(OrderDirection::Asc);

        // **Note** `timeframe_stats.timeframe_start_timestamp >= COALESCE($2, (SELECT MIN(timeframe_start_timestamp) FROM subscription_query_result WHERE ticket_user = result.ticket_user))`
        // Building dynamic queries with the `from_sql_and_values` is not particularly straight forward as it has to be valid SQL.
        // To handle a potentially `None` `start` and `end` value, the `where` clause uses a COALESCE to say:
        // - use the passed in variable value if one is provided,
        // - otherwise, find all records where the `timeframe_start_timestamp` is >= to the lowest `timeframe_start_timestamp` for the user
        // The same logic goes for the `end` value.
        // It may be more efficient to just pass in `O` for a start if none is provided,
        // and the `i64::MAX` value for the end, if none is provided.
        UserSubscriptionStat::find_by_statement(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"
            WITH timeframe_stats (ticket_user, timeframe_start_timestamp, timeframe_end_timestamp, query_count, success_rate, failed_query_count, avg_response_time_ms) AS (
                SELECT
                    ticket_user,
                    timeframe_start_timestamp,
                    timeframe_end_timestamp,
                    CAST(SUM(query_count) AS bigint) AS query_count,
                    SUM(CASE WHEN status_code = 'SUCCESS' THEN query_count ELSE 0 END)::float4 / SUM(query_count)::float4 AS success_rate,
                    SUM(CASE WHEN status_code != 'SUCCESS' THEN query_count ELSE 0 END)::bigint AS failed_query_count,
                    AVG(response_time_ms)::INT4 AS avg_response_time_ms
                FROM subscription_query_result
                GROUP BY ticket_user, timeframe_start_timestamp, timeframe_end_timestamp
            )
            SELECT
                result.ticket_user,
                timeframe_stats.timeframe_start_timestamp AS timeframe_start_timestamp,
                timeframe_stats.timeframe_end_timestamp AS timeframe_end_timestamp,
                CAST(MAX(timeframe_stats.query_count) AS bigint) AS query_count,
                MAX(timeframe_stats.success_rate)::float4 AS success_rate,
                AVG(timeframe_stats.avg_response_time_ms)::INT4 AS avg_response_time_ms,
                MAX(timeframe_stats.failed_query_count) AS failed_query_count
            FROM subscription_query_result AS result
            JOIN timeframe_stats
            ON timeframe_stats.ticket_user = result.ticket_user
                AND timeframe_stats.timeframe_start_timestamp = result.timeframe_start_timestamp
                AND timeframe_stats.timeframe_end_timestamp = result.timeframe_end_timestamp
            WHERE
                result.ticket_user = $1
                AND timeframe_stats.timeframe_start_timestamp >= COALESCE($2, (SELECT MIN(timeframe_start_timestamp) FROM subscription_query_result WHERE ticket_user = result.ticket_user))
                AND timeframe_stats.timeframe_end_timestamp <= COALESCE($3, (SELECT MAX(timeframe_end_timestamp) FROM subscription_query_result WHERE ticket_user = result.ticket_user))
            GROUP BY
                result.ticket_user,
                timeframe_stats.timeframe_start_timestamp,
                timeframe_stats.timeframe_end_timestamp
            ORDER BY $4;
            "#,
            [
                user.to_string().into(),
                Value::BigInt(start),
                Value::BigInt(end),
                format!("timeframe_stats.{} {}", order_by.as_str(), order_direction.as_str()).into(),
            ],
        ))
        .all(&self.db_conn)
        .await
        .map_err(|err| anyhow::Error::from(err))
    }

    /// Retrieve the user's `RequestTicketStat` records, aggregated over the given timeframe, derived from the stored query result records from the postgres database.
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
        order_by: Option<StatOrderBy>,
        order_direction: Option<OrderDirection>,
    ) -> anyhow::Result<Vec<RequestTicketStat>> {
        let order_by = order_by.unwrap_or(StatOrderBy::Start);
        let order_direction = order_direction.unwrap_or(OrderDirection::Asc);
        let limit = first.unwrap_or(100);
        let offset = skip.unwrap_or(0);

        RequestTicketStat::find_by_statement(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"
            WITH timeframe_stats (ticket, timeframe_start_timestamp, timeframe_end_timestamp, query_count, success_rate, failed_query_count, avg_response_time_ms) AS (
                SELECT
                    CONCAT(ticket_name, ticket_user) AS ticket,
                    timeframe_start_timestamp,
                    timeframe_end_timestamp,
                    CAST(SUM(query_count) AS bigint) AS query_count,
                    SUM(CASE WHEN status_code = 'SUCCESS' THEN query_count ELSE 0 END)::float4 / SUM(query_count)::float4 AS success_rate,
                    SUM(CASE WHEN status_code != 'SUCCESS' THEN query_count ELSE 0 END)::bigint AS failed_query_count,
                    AVG(response_time_ms)::INT4 AS avg_response_time_ms
                FROM subscription_query_result
                GROUP BY CONCAT(ticket_name, ticket_user), timeframe_start_timestamp, timeframe_end_timestamp
            ), queried_subgraphs_count (ticket, queried_subgraphs_count) AS (
                SELECT CONCAT(ticket_name, ticket_user), COUNT(DISTINCT deployment_qm_hash)
                FROM subscription_query_result
                GROUP BY CONCAT(ticket_name, ticket_user)
            )
            SELECT
                ticket_name,
                ticket_user,
                timeframe_stats.timeframe_start_timestamp AS timeframe_start_timestamp,
                timeframe_stats.timeframe_end_timestamp AS timeframe_end_timestamp,
                CAST(MAX(timeframe_stats.query_count) AS bigint) AS query_count,
                MAX(timeframe_stats.success_rate)::float4 AS success_rate,
                AVG(timeframe_stats.avg_response_time_ms)::INT4 AS avg_response_time_ms,
                MAX(timeframe_stats.failed_query_count) AS failed_query_count,
                MAX(queried_subgraphs_count.queried_subgraphs_count) AS queried_subgraphs_count
            FROM subscription_query_result AS result
            JOIN timeframe_stats
                ON timeframe_stats.ticket = CONCAT(result.ticket_name, result.ticket_user)
                    AND timeframe_stats.timeframe_start_timestamp = result.timeframe_start_timestamp
                    AND timeframe_stats.timeframe_end_timestamp = result.timeframe_end_timestamp
            JOIN queried_subgraphs_count
                ON queried_subgraphs_count.ticket = CONCAT(result.ticket_name, result.ticket_user)
            WHERE ticket_user = $1 AND ticket_name = $2
            GROUP BY
                ticket_name,
                ticket_user,
                timeframe_stats.timeframe_start_timestamp,
                timeframe_stats.timeframe_end_timestamp
            ORDER BY $3
            LIMIT $4
            OFFSET $5;
            "#,
            [
                user.to_string().to_lowercase().into(),
                ticket_name.into(),
                format!("{} {}", order_by.as_str(), order_direction.as_str()).into(),
                limit.into(),
                offset.into()
            ],
        ))
        .all(&self.db_conn)
        .await
        .map_err(|err| anyhow::Error::from(err))
    }

    /// Retrieve the user's `RequestTicketSubgraphStat` records, aggregated over the given timeframe, for a specific subgraph deployment Qm hash, derived from the stored query result records from the postgres database.
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
        order_by: Option<StatOrderBy>,
        order_direction: Option<OrderDirection>,
    ) -> anyhow::Result<Vec<RequestTicketSubgraphStat>> {
        let order_by = order_by.unwrap_or(StatOrderBy::Start);
        let order_direction = order_direction.unwrap_or(OrderDirection::Asc);
        let limit = first.unwrap_or(100);
        let offset = skip.unwrap_or(0);

        RequestTicketSubgraphStat::find_by_statement(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"
            WITH timeframe_stats (ticket, timeframe_start_timestamp, timeframe_end_timestamp, query_count, success_rate, failed_query_count, avg_response_time_ms) AS (
                SELECT
                    CONCAT(ticket_name, ticket_user, deployment_qm_hash) AS ticket,
                    timeframe_start_timestamp,
                    timeframe_end_timestamp,
                    CAST(SUM(query_count) AS bigint) AS query_count,
                    SUM(CASE WHEN status_code = 'SUCCESS' THEN query_count ELSE 0 END)::float4 / SUM(query_count)::float4 AS success_rate,
                    SUM(CASE WHEN status_code != 'SUCCESS' THEN query_count ELSE 0 END)::bigint AS failed_query_count,
                    AVG(response_time_ms)::INT4 AS avg_response_time_ms
                FROM subscription_query_result
                GROUP BY CONCAT(ticket_name, ticket_user, deployment_qm_hash), timeframe_start_timestamp, timeframe_end_timestamp
            )
            SELECT
                ticket_name,
                ticket_user,
                deployment_qm_hash,
                timeframe_stats.timeframe_start_timestamp AS timeframe_start_timestamp,
                timeframe_stats.timeframe_end_timestamp AS timeframe_end_timestamp,
                CAST(MAX(timeframe_stats.query_count) AS bigint) AS query_count,
                MAX(timeframe_stats.success_rate)::float4 AS success_rate,
                AVG(timeframe_stats.avg_response_time_ms)::INT4 AS avg_response_time_ms,
                MAX(timeframe_stats.failed_query_count) AS failed_query_count
            FROM subscription_query_result AS result
            JOIN timeframe_stats
                ON timeframe_stats.ticket = CONCAT(result.ticket_name, result.ticket_user, result.deployment_qm_hash)
                    AND timeframe_stats.timeframe_start_timestamp = result.timeframe_start_timestamp
                    AND timeframe_stats.timeframe_end_timestamp = result.timeframe_end_timestamp
            WHERE
                ticket_user = $1
                AND ticket_name = $2
                AND deployment_qm_hash = $3
            GROUP BY
                ticket_name,
                ticket_user,
                deployment_qm_hash,
                timeframe_stats.timeframe_start_timestamp,
                timeframe_stats.timeframe_end_timestamp
            ORDER BY $4
            LIMIT $5
            OFFSET $6;
            "#,
            [
                user.to_string().to_lowercase().into(),
                ticket_name.into(),
                subgraph_deployment_qm_hash.ipfs_hash().into(),
                format!("{} {}", order_by.as_str(), order_direction.as_str()).into(),
                limit.into(),
                offset.into()
            ],
        ))
        .all(&self.db_conn)
        .await
        .map_err(|err| anyhow::Error::from(err))
    }
}

#[async_trait]
impl DatasourceWriter for DatasourcePostgres {
    async fn write(&self, consumer: &StreamConsumer<DefaultConsumerContext>) {
        let stream_processor = consumer.stream().try_for_each(move |borrowed_msg| {
            async move {
                let msg = borrowed_msg.detach();
                // convert the `OwnedMessage`
                let query_result_msg = match GatewaySubscriptionQueryResult::from_slice(
                    msg.payload().unwrap_or_default(),
                ) {
                    Err(err) => {
                        tracing::warn!("DatasourcePostgres.store_subscription_query_result_record()::cannot deserialize message. skipping offset: [{}]. {}", msg.offset(), err);
                        return Result::<(), KafkaError>::Ok(());
                    }
                    std::result::Result::Ok(payload) => payload,
                };

                // build a `GraphSubscriptionQueryResultRecord`
                let timestamp = msg
                    .timestamp()
                    .to_millis()
                    .map(|ms| ms / 1000)
                    .unwrap_or(Utc::now().timestamp());
                let offset = msg.offset();
                let key = String::from_utf8_lossy(msg.key().unwrap_or_default()).to_string();
                let (start, end) = build_timerange_timestamp(timestamp);

                let ticket_payload = serde_json::from_str::<TicketPayload>(&query_result_msg.ticket_payload).map_err(|_| KafkaError::MessageConsumption(rdkafka::types::RDKafkaErrorCode::BadMessage))?;
                let ticket_name = ticket_payload.name;

                let result_record = entity::subscription_query_result::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    query_id: Set(query_result_msg.query_id),
                    ticket_user: Set(query_result_msg.ticket_user.to_lowercase()),
                    ticket_payload: Set(json!(query_result_msg.ticket_payload)),
                    ticket_name: Set(ticket_name),
                    deployment_qm_hash: Set(query_result_msg.deployment),
                    query_count: Set(query_result_msg.query_count.unwrap_or(0) as i64),
                    status_code: Set(entity::sea_orm_active_enums::SubscriptionQueryResultStatus::from_i32(query_result_msg.status_code)),
                    subgraph_chain: Set(query_result_msg.subgraph_chain),
                    response_time_ms: Set(query_result_msg.response_time_ms.try_into().unwrap_or(0)),
                    query_budget: Set(query_result_msg.query_budget.unwrap_or(0.00)),
                    indexer_fees: Set(query_result_msg.indexer_fees.unwrap_or(0.00)),
                    message_timestamp: Set(timestamp),
                    timeframe_start_timestamp: Set(start),
                    timeframe_end_timestamp: Set(end),
                    message_offset: Set(offset),
                    message_key: Set(key)
                };

                match result_record.insert(&self.db_conn).await {
                    Result::Ok(_) => tracing::info!("successfully stored message [{}] in db...", offset),
                    Err(err) => {
                        tracing::error!("failure storing message in db [{:#?}]. skipping...", err);
                        return Result::<(), KafkaError>::Ok(());
                    }
                }

                Result::<(), KafkaError>::Ok(())
            }
        });

        tracing::info!(
            "DatasourcePostgres.write()::initializing message stream consumer processing..."
        );
        stream_processor
            .await
            .expect("DatasourcePostgres.write()::failure processing the stream messages");
        tracing::info!("DatasourcePostgres.write()::message stream consumer terminated");
    }
}
