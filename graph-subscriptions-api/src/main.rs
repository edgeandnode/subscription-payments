use std::{io::Write as _, net::SocketAddr, sync::Arc, time::Duration};

use async_graphql::{http::GraphiQLSource, EmptyMutation, EmptySubscription, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use auth::TicketPayloadWrapper;
use axum::{
    extract::Extension,
    http::{header, status::StatusCode, HeaderMap, Method},
    response::{self, IntoResponse},
    routing::get,
    Router, Server,
};
use datasource::{CreateWithDatasourcePgArgs, DatasourcePostgres, GraphSubscriptionsDatasource};
use graph_subscriptions::TicketVerificationDomain;
use prometheus::{self, Encoder as _};
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{self, layer::SubscriberExt as _, util::SubscriberInitExt as _};

mod auth;
mod config;
mod network_subgraph;
mod schema;
mod subgraph_client;
mod subscriptions_subgraph;

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
        req = req.data::<TicketPayloadWrapper>(token);
    }
    schema.execute(req).await.into()
}

async fn graphiql(endpoint: String) -> impl IntoResponse {
    response::Html(GraphiQLSource::build().endpoint(endpoint.as_str()).finish())
}

async fn handle_metrics() -> impl IntoResponse {
    let encoder = prometheus::TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    if let Err(metrics_encode_err) = encoder.encode(&metric_families, &mut buffer) {
        tracing::error!(%metrics_encode_err);
        buffer.clear();
        write!(&mut buffer, "Failed to encode metrics").unwrap();
        return (StatusCode::INTERNAL_SERVER_ERROR, buffer);
    }
    (StatusCode::OK, buffer)
}

#[tokio::main]
async fn main() {
    let conf = init_config();
    let graphql_endpoint = conf.graphql_endpoint;

    init_tracing(conf.log_json);

    tracing::info!("Graph Subscriptions API starting...");

    // Host metrics on a separate server with a port that isn't open to public requests.
    tokio::spawn(async move {
        let router = Router::new().route("/metrics", get(handle_metrics));

        let metrics_addr = SocketAddr::from(([0, 0, 0, 0], conf.metrics_port));
        tracing::info!(
            "Graph Subscriptions API::metrics listening on [{}]",
            metrics_addr
        );
        Server::bind(&metrics_addr)
            .serve(router.into_make_service())
            .await
            .expect("Failed to start metrics server");
    });

    let http_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap();
    let network_subgraph_client =
        subgraph_client::Client::new(http_client.clone(), conf.network_subgraph_url.clone());
    let network_subgraph_data = network_subgraph::Client::create(network_subgraph_client);
    let subscriptions = subscriptions_subgraph::Client::create(subgraph_client::Client::new(
        http_client.clone(),
        conf.subscriptions_subgraph_url.clone(),
    ));

    let subscriptions_datasource =
        GraphSubscriptionsDatasource::<DatasourcePostgres>::create_with_datasource_pg(
            CreateWithDatasourcePgArgs {
                kafka_broker: conf.graph_subscription_logs_kafka_broker,
                kafka_subscription_logs_group_id: conf.graph_subscription_logs_kafka_group_id,
                kafka_subscription_logs_topic_id: conf.graph_subscription_logs_kafka_topic_id,
                kafka_additional_config: conf.graph_subscription_logs_kafka_additional_config,
                postgres_db_url: conf.graph_subscription_logs_db_url,
                num_workers: Some(2),
            },
        )
        .await
        .expect("Failure instantiating the `GraphSubscriptionsDatasource` instance");

    // instantiate a context instance that will be passed as data to the graphql resolver functions in the context instance
    let ctx = Arc::new(Mutex::new(GraphSubscriptionsSchemaCtx {
        subgraph_deployments: network_subgraph_data.subgraph_deployments,
        datasource: subscriptions_datasource.datasource,
        subscription_tiers: conf.subscriptions_tiers,
    }));

    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .data::<Arc<Mutex<GraphSubscriptionsSchemaCtx>>>(ctx)
        .limit_depth(32)
        .finish();

    let subscriptions_domain = TicketVerificationDomain {
        contract: ethers::types::H160(conf.subscriptions_contract_address.0),
        chain_id: conf.subscriptions_chain_id,
    };
    let auth_handler = AuthHandler::create(subscriptions_domain, subscriptions);

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
