# Graph Subscriptions API

GraphQL API that exposes queries for user's to see their graph-subscription request tickets and the stats associated with these request tickets.

## Env

This app utilizes the `dotenv` library to handle environment variables. The default env file is `.env` in the `graph-subscriptions-api` directory.

See the [.env.example](./.env.example) file for default values.

## Running (with Docker/Docker compose)

```bash
# list available jest recipes
just -l

# build the crate
just build # alias ontop of `cargo build`

# build the docker image for the create
just build_docker

# spin up the docker image
## this runs the docker build command as a pre-requisite.
just docker_up

# build the docker image and spin up the image
just docker_build_start

# teardown the docker image
just docker_down

# clean any dangling volumes
just docker_clean
```

### Installing Just

```bash
# on mac os
brew install just
# with cargo directly
cargo install just
```

For other installation options, check out the [repo](https://github.com/casey/just#packages)
