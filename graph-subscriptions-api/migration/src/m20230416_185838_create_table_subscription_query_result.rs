use sea_orm_migration::{
    prelude::*,
    sea_orm::{ConnectionTrait, EnumIter},
    sea_query::extension::postgres::Type,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        manager
            .create_type(
                Type::create()
                    .as_enum(SubscriptionQueryResultStatus::Table)
                    .values([
                        SubscriptionQueryResultStatus::Success,
                        SubscriptionQueryResultStatus::InternalError,
                        SubscriptionQueryResultStatus::UserError,
                        SubscriptionQueryResultStatus::NotFound,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(SubscriptionQueryResult::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SubscriptionQueryResult::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(SubscriptionQueryResult::QueryId)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SubscriptionQueryResult::TicketUser)
                            .string_len(42)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SubscriptionQueryResult::TicketPayload)
                            .json()
                            .not_null(),
                    )
                    .col(ColumnDef::new(SubscriptionQueryResult::TicketName).text())
                    .col(ColumnDef::new(SubscriptionQueryResult::DeploymentQmHash).string_len(46))
                    .col(
                        ColumnDef::new(SubscriptionQueryResult::QueryCount)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(SubscriptionQueryResult::StatusCode)
                            .enumeration(
                                SubscriptionQueryResultStatus::Table,
                                [
                                    SubscriptionQueryResultStatus::Success,
                                    SubscriptionQueryResultStatus::InternalError,
                                    SubscriptionQueryResultStatus::UserError,
                                    SubscriptionQueryResultStatus::NotFound,
                                ],
                            )
                            .not_null()
                            .default("SUCCESS"),
                    )
                    .col(
                        ColumnDef::new(SubscriptionQueryResult::SubgraphChain)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(SubscriptionQueryResult::ResponseTimeMs)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(SubscriptionQueryResult::QueryBudget)
                            .float()
                            .not_null()
                            .default(0.00),
                    )
                    .col(
                        ColumnDef::new(SubscriptionQueryResult::IndexerFees)
                            .float()
                            .not_null()
                            .default(0.00),
                    )
                    .col(
                        ColumnDef::new(SubscriptionQueryResult::MessageTimestamp)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SubscriptionQueryResult::TimeframeStartTimestamp)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SubscriptionQueryResult::TimeframeEndTimestamp)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SubscriptionQueryResult::MessageOffset)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SubscriptionQueryResult::MessageKey)
                            .text()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // create an index on the: ticket_user, ticket_signer, ticket_name & deployment_qm_hash
        manager
            .create_index(
                Index::create()
                    .name("idx__subscription_query_result__ticket_user")
                    .if_not_exists()
                    .table(SubscriptionQueryResult::Table)
                    .col(SubscriptionQueryResult::TicketUser)
                    .to_owned(),
            )
            .await?;
        // only create an index on the `ticket_name` & `deployment_qm_hash` values where the value is not null
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx__subscription_query_result__ticket_name ON subscription_query_result (ticket_name) WHERE ticket_name IS NOT NULL"
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx__subscription_query_result__deployment_qm_hash ON subscription_query_result (deployment_qm_hash) WHERE deployment_qm_hash IS NOT NULL"
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx__subscription_query_result__ticket_user")
                    .table(SubscriptionQueryResult::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .name("idx__subscription_query_result__ticket_name")
                    .table(SubscriptionQueryResult::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .name("idx__subscription_query_result__deployment_qm_hash")
                    .table(SubscriptionQueryResult::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(SubscriptionQueryResult::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_type(
                Type::drop()
                    .name(SubscriptionQueryResultStatus::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum SubscriptionQueryResult {
    #[iden = "subscription_query_result"]
    Table,
    Id,
    #[iden = "query_id"]
    QueryId,
    #[iden = "ticket_user"]
    TicketUser,
    #[iden = "ticket_payload"]
    TicketPayload,
    #[iden = "ticket_name"]
    TicketName,
    #[iden = "deployment_qm_hash"]
    DeploymentQmHash,
    #[iden = "query_count"]
    QueryCount,
    #[iden = "status_code"]
    StatusCode,
    #[iden = "subgraph_chain"]
    SubgraphChain,
    #[iden = "response_time_ms"]
    ResponseTimeMs,
    #[iden = "query_budget"]
    QueryBudget,
    #[iden = "indexer_fees"]
    IndexerFees,
    #[iden = "message_timestamp"]
    MessageTimestamp,
    #[iden = "timeframe_start_timestamp"]
    TimeframeStartTimestamp,
    #[iden = "timeframe_end_timestamp"]
    TimeframeEndTimestamp,
    #[iden = "message_offset"]
    MessageOffset,
    #[iden = "message_key"]
    MessageKey,
}

#[derive(Iden, EnumIter)]
pub enum SubscriptionQueryResultStatus {
    #[iden = "subscription_query_result_status"]
    Table,
    #[iden = "SUCCESS"]
    Success,
    #[iden = "USER_ERROR"]
    UserError,
    #[iden = "INTERNAL_ERROR"]
    InternalError,
    #[iden = "NOT_FOUND"]
    NotFound,
}
