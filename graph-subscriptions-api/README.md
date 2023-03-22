# Graph Subscriptions API

GraphQL API that exposes queries for user's to see their graph-subscription request tickets and the stats associated with these request tickets.

## Env

This app utilizes the `dotenv` library to handle environment variables. The default env file is `.env` in the `graph-subscriptions-api` directory.

See the [.env.example](./.env.example) file for default values.

## Running

```bash
cargo run

# -> graph-subscriptions-api listening on: http://localhost:4000
# -> graphiql IDE running on: http://localhost:4000/graphql
```
