# Graph Subscriptions API | Datasource

This subpackage to the Graph Subscriptions API exposes a generic, extendable, API for consuming, parsing, and querying logs pushed on the subscription query logs `kafka` topic by The Graph Gateway.

## Extendable Traits

- [`Datasource`](./src/datasource.rs): exposes methods for retrieving a list of `RequestTicket`, `RequestTicketStat` and `RequestTicketSubgraphStat` records pulled and parsed from the data storage model determined by the implementer of the `Datasource`.
  - methods:
    - `request_tickets`: retrieves a list of unique `RequestTicket` records
    - `request_ticket_stats`: retrieves a list of `RequestTicketStat` records
    - `request_ticket_subgraph_stats`: retrieves a list of `RequestTicketSubgraphStat` records
  - Some example implementers of `Datasource`:
    - [`DatasourceRedis`](./src/datasource_redis.rs) - which implements the datasource instance, and pulls records from a `redis` database.
- [`DatasourceWriter`](./src/datasource.rs): exposes a `write` method which takes a reference to a `rdkafka::StreamConsumer`, listens on a stream of log messages, and writes them to the storage model defined by the implementer of the trait.
  - Some example implementers of `DatasourceWriter`:
    - [`DatasourceRedis`](./src/datasource_redis.rs) - listens to the log `StreamConsumer` message stream and stores the records in a redis database instance.

## Usage

Instantiate a new [`GraphSubscriptionsDatasource`](./src/lib.rs) using whichever datasource implementer suits your needs for the `graph-subscriptions-api`:

```rust
use datasource:{GraphSubscriptionsDatasource, RequestTicket};
use toolshed::url::Url;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  // instantiate the datasource instance, and begin consuming data from the kafka logs consumer and storing in memory
  let subscriptions_datasource = GraphSubscriptionsDatasource::create_with_datasource_inmemory(
    "localhost:9092".parse::<Url>().unwrap(),
    String::from("graph_subscription_log_group"),
    String::from("gateway_subscription_query_results"),
    Some(2)
  ).await?;
  // get the request tickets
  let user = "0xa476caFd8b08F11179BDDd5145FcF3EF470C7462".parse::<Address>()?;
  let request_tickets: Vec<RequestTicket> = subscriptions_datasource.datasource.request_tickets(user, None, None, None, None)
    .await
    .unwrap_or_default();
}
```
