use anyhow::{Ok, Result};
use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema};

pub type GraphSubscriptionsSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn hello<'a>(&self) -> Result<String> {
        Ok(String::from("world"))
    }
}
