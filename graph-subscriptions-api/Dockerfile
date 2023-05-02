FROM rust:1.68-bullseye AS build

ARG API_PORT=4000
ARG METRICS_PORT=9090

ENV API_PORT=${API_PORT}
ENV METRICS_PORT=${METRICS_PORT}

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
EXPOSE $METRICS_PORT

ENTRYPOINT [ "target/release/graph-subscriptions-api" ]