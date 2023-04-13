FROM rust:1.68-bullseye AS build

# used to pull private github packages needed in the build
ARG GH_USER
ARG GH_TOKEN

# app env setup
ARG API_PORT
ARG GRAPHQL_ENDPOINT
ARG LOG_JSON
ARG NETWORK_SUBGRAPH_URL
ARG SUBSCRIPTIONS_CONTRACT_CHAIN_ID
ARG SUBSCRIPTIONS_CONTRACT_ADDRESS
ARG SUBSCRIPTIONS_SUBGRAPH_URL
ARG GRAPH_SUBSCRIPTION_LOGS_KAFKA_BROKER
ARG GRAPH_SUBSCRIPTION_LOGS_KAFKA_GROUP_ID
ARG GRAPH_SUBSCRIPTION_LOGS_KAFKA_TOPIC_ID
ARG GRAPH_SUBSCRIPTION_LOGS_REDIS_URL

ENV API_PORT=${API_PORT}
ENV GRAPHQL_ENDPOINT=${GRAPHQL_ENDPOINT}
ENV LOG_JSON=${LOG_JSON}
ENV NETWORK_SUBGRAPH_URL=${NETWORK_SUBGRAPH_URL}
ENV SUBSCRIPTIONS_CONTRACT_CHAIN_ID=${SUBSCRIPTIONS_CONTRACT_CHAIN_ID}
ENV SUBSCRIPTIONS_CONTRACT_ADDRESS=${SUBSCRIPTIONS_CONTRACT_ADDRESS}
ENV SUBSCRIPTIONS_SUBGRAPH_URL=${SUBSCRIPTIONS_SUBGRAPH_URL}
ENV GRAPH_SUBSCRIPTION_LOGS_KAFKA_BROKER=${GRAPH_SUBSCRIPTION_LOGS_KAFKA_BROKER}
ENV GRAPH_SUBSCRIPTION_LOGS_KAFKA_GROUP_ID=${GRAPH_SUBSCRIPTION_LOGS_KAFKA_GROUP_ID}
ENV GRAPH_SUBSCRIPTION_LOGS_KAFKA_TOPIC_ID=${GRAPH_SUBSCRIPTION_LOGS_KAFKA_TOPIC_ID}
ENV GRAPH_SUBSCRIPTION_LOGS_REDIS_URL=${GRAPH_SUBSCRIPTION_LOGS_REDIS_URL}

RUN apt-get update && apt-get install -y \
  build-essential \
  clang \
  cmake \
  git \
  librdkafka-dev \
  libsasl2-dev \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /opt/graph-subscriptions-api
COPY ./ ./

# Setup GitHub credentials for cargo fetch
RUN git config --global credential.helper store \
  && git config --global --replace-all url.https://github.com/.insteadOf ssh://git@github.com/ \
  && git config --global --add url.https://github.com/.insteadOf git@github.com: \
  && mkdir ~/.cargo && echo "[net]\ngit-fetch-with-cli = true" > ~/.cargo/config.toml \
  && (echo url=https://github.com; echo "username=${GH_USER}"; echo "password=${GH_TOKEN}"; echo ) | git credential approve

ENV CC=clang CXX=clang++
RUN cargo build --release --bin graph-subscriptions-api --color=always

FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y \
  libssl1.1 \
  ca-certificates \
  && rm -rf /var/lib/apt/lists/*

COPY --from=build /opt/graph-subscriptions-api/target/release/graph-subscriptions-api /opt/graph-subscriptions-api/target/release/graph-subscriptions-api

WORKDIR /opt/graph-subscriptions-api

EXPOSE $API_PORT

ENTRYPOINT [ "target/release/graph-subscriptions-api" ]