//! `SeaORM` Entity. Generated by sea-orm-codegen 0.11.2

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "subscription_query_result_status"
)]
pub enum SubscriptionQueryResultStatus {
    #[sea_orm(string_value = "INTERNAL_ERROR")]
    InternalError,
    #[sea_orm(string_value = "NOT_FOUND")]
    NotFound,
    #[sea_orm(string_value = "SUCCESS")]
    Success,
    #[sea_orm(string_value = "USER_ERROR")]
    UserError,
}

impl SubscriptionQueryResultStatus {
    pub fn from_i32(val: i32) -> Self {
        match val {
            0 => SubscriptionQueryResultStatus::Success,
            1 => SubscriptionQueryResultStatus::InternalError,
            2 => SubscriptionQueryResultStatus::UserError,
            _ => SubscriptionQueryResultStatus::NotFound,
        }
    }
}
