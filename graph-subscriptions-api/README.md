# Graph Subscriptions API

GraphQL API that exposes queries for user's to see their graph-subscription request tickets and the stats associated with these request tickets.

## Env

This app utilizes the `dotenv` library to handle environment variables. The default env file is `.env` in the `graph-subscriptions-api` directory.

See the [.env.example](./.env.example) file for default values.

## Running (with Docker/Docker compose)

```bash
# list available jest recipes
just -l

# spin up the necessary resources for the app (kafka, redis, etc)
just docker_up

# run the api in dev mode
just dev

# create the topic on the local kafka broker
## requires running `just docker_up` first
## replace ${TOPIC_ID} with the name of the topic to create
docker exec -it redpanda_1 \
  rpk topic create ${TOPIC_ID} --brokers=localhost:9092

# teardown the docker image
just docker_down

# clean any dangling volumes
just docker_clean
```

Once the api is running, go to http://localhost:4000/graphql

Generating a request ticket `Authorization` header.
The `Authorization` header is required for all queries exposed by the graphql api.
Copy the printed value from the CLI `ticket` command (see below) and paste it as the `Authorization` header `Bearer` token when querying the graphql api. Example:

```json
{
  "Authorization": "Bearer oWZzaWduZXJU85_W5RqtiPb0zmq4gnJ5z_-5ImbA6wIp3scGHIyDvZ3rbuxtrpn3g77L1StaJf1smrTVSkIgBeV0vPEePGiZWSi3AuklQ5kJPvQi80b9TcLoUKvUGw"
}
```

Notes:

- the wallet you use to sign the message for the authorization header must have an active subscription record in the subscriptions contract passed to the `ticket` command

```bash
# nav back into the `cli` directory
cd ../cli

# run the `ticket` command
echo $PRIV_KEY | cargo run -- \
  --chain-id=$CHAIN_ID \
  --subscriptions=$SUBSCRIPTIONS_CONTRACT_ADDRESS \
  --token=$TOKEN_CONTRACT_ADDRESS \
  --provider=$PROVIDER_URL \
  ticket \
  --signer=$SIGNER_WALLET_ADDRESS \
  --name=$TICKET_NAME
```

Where:

- `$PRIV_KEY` = the signing wallet private key
- `$CHAIN_ID` = the chain id the subscriptions/token contract are deployed to.
  - ex: `421613` for arbitrum-goerli
- `$SUBSCRIPTIONS_CONTRACT_ADDRESS` = the deployed subscriptions contract address
  - ex: `0x29f49a438c747e7Dd1bfe7926b03783E47f9447B` for the contract on arbitrum-goerli
- `$TOKEN_CONTRACT_ADDRESS` = the deployed token address plugged into the subscriptions contract
  - ex: `0x8FB1E3fC51F3b789dED7557E680551d93Ea9d892` for the USDC contract on arbitrum-goerli
- `$PROVIDER_URL` = an RPC provider url for the chain id
- `$SIGNER_WALLET_ADDRESS` = the signer wallet address
- `$TICKET_NAME` = a friendly name for the request ticket
  - ex: `req_ticket_1`

### Installing Just

```bash
# on mac os
brew install just
# with cargo directly
cargo install just
```

For other installation options, check out the [repo](https://github.com/casey/just#packages)
