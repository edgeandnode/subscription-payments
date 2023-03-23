use std::{net::SocketAddr, time::Duration};

use async_graphql::{http::GraphiQLSource, EmptyMutation, EmptySubscription, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::Extension,
    http::{header, HeaderMap, Method},
    response::{self, IntoResponse},
    routing::get,
    Router, Server,
};
use graph_subscriptions::{eip712, TicketPayload};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{self, layer::SubscriberExt as _, util::SubscriberInitExt as _};

mod auth;
mod config;
mod network_subgraph;
mod schema;
mod subgraph_client;

use crate::auth::AuthHandler;
use crate::config::init_config;
use crate::schema::{GraphSubscriptionsSchema, GraphSubscriptionsSchemaCtx, QueryRoot};

async fn graphql_handler(
    schema: Extension<GraphSubscriptionsSchema>,
    headers: HeaderMap,
    req: GraphQLRequest,
    auth_handler: &AuthHandler,
) -> GraphQLResponse {
    let mut req = req.into_inner();
    if let Ok(token) = auth_handler.parse_auth_header(&headers) {
        req = req.data::<TicketPayload>(token);
    }
    schema.execute(req).await.into()
}

async fn graphiql(endpoint: String) -> impl IntoResponse {
    response::Html(GraphiQLSource::build().endpoint(endpoint.as_str()).finish())
}

#[tokio::main]
async fn main() {
    let conf = init_config();
    let graphql_endpoint = conf.graphql_endpoint;

    init_tracing(conf.log_json);

    tracing::info!("Graph Subscriptions API starting...");

    let http_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap();
    let network_subgraph_client =
        subgraph_client::Client::new(http_client.clone(), conf.network_subgraph_url.clone());
    let network_subgraph_data = network_subgraph::Client::create(network_subgraph_client);

    // instantiate a context instance that will be passed as data to the graphql resolver functions in the context instance
    let ctx = GraphSubscriptionsSchemaCtx {
        subgraph_deployments: network_subgraph_data.subgraph_deployments,
    };

    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .data::<GraphSubscriptionsSchemaCtx>(ctx)
        .limit_depth(32)
        .finish();

    let subscriptions_domain_separator =
        eip712::DomainSeparator::new(&TicketPayload::eip712_domain(
            conf.subscriptions_chain_id,
            conf.subscriptions_contract_address.0.into(),
        ));
    let auth_handler = AuthHandler::create(subscriptions_domain_separator);

    let cors = CorsLayer::new()
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_origin(Any);
    let app = Router::new()
        .route(
            graphql_endpoint.clone().as_str(),
            get(|| graphiql(graphql_endpoint))
                .post(|(schema, headers, req)| graphql_handler(schema, headers, req, auth_handler)),
        )
        .layer(Extension(schema))
        .layer(cors);

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
