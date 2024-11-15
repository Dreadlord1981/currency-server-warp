# state 1 planner

FROM rust:latest as planner
WORKDIR /usr/apps/currency
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# state 2 cacher

FROM rust:latest as cacher
WORKDIR /usr/apps/currency
RUN cargo install cargo-chef
COPY --from=planner /usr/apps/currency/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# state 3 build

FROM rust:latest as builder
WORKDIR /usr/apps/currency
COPY . .

COPY --from=cacher /usr/apps/currency/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo

RUN cargo build --release
FROM debian:buster-slim
RUN apt-get update & apt-get install -y extra-runtime-dependencies & rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/apps/currency/target/release/currency /usr/local/bin/currency
CMD ["currency"]