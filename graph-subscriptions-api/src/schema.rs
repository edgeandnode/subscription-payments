use anyhow::{Ok, Result};
use async_graphql::{Context, EmptyMutation, EmptySubscription, Object, Schema};
use graph_subscriptions::TicketPayload;

use crate::auth::AuthError;

pub type GraphSubscriptionsSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn hello<'a>(&self, ctx: &Context<'a>) -> Result<String> {
        let token = ctx.data_opt::<TicketPayload>();
        if token.is_none() {
            return Err(AuthError::Unauthorized.into());
        }
        Ok(String::from("world"))
    }
}
