use async_graphql::{http::GraphiQLSource, EmptyMutation, EmptySubscription, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::Extension,
    response::{self, IntoResponse},
    routing::get,
    Router, Server,
};
use std::net::SocketAddr;
use tracing_subscriber::{self, layer::SubscriberExt as _, util::SubscriberInitExt as _};

mod config;
mod schema;

use crate::config::init_config;
use crate::schema::{GraphSubscriptionsSchema, QueryRoot};

async fn graphql_handler(
    schema: Extension<GraphSubscriptionsSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn graphiql(endpoint: String) -> impl IntoResponse {
    response::Html(GraphiQLSource::build().endpoint(endpoint.as_str()).finish())
}

#[tokio::main]
async fn main() {
    let conf = init_config();
    let graphql_endpoint = conf.graphql_endpoint.clone();

    init_tracing(conf.log_json);

    tracing::info!("Graph Subscriptions API starting...");

    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();

    let app = Router::new()
        .route(
            graphql_endpoint.clone().as_str(),
            get(|| graphiql(graphql_endpoint)).post(graphql_handler),
        )
        .layer(Extension(schema));

    let addr = SocketAddr::from(([0, 0, 0, 0], conf.api_port));
    tracing::info!("Graph Subscriptions API listening on [{}]", addr);
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn init_tracing(json: bool) {
    let filter_layer = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        tracing_subscriber::EnvFilter::try_new("info,graph_subscriptions_api=debug").unwrap()
    });
    let defaults = tracing_subscriber::registry().with(filter_layer);
    let fmt_layer = tracing_subscriber::fmt::layer();
    if json {
        defaults
            .with(fmt_layer.json().with_current_span(false))
            .init();
    } else {
        defaults.with(fmt_layer).init();
    }
}
