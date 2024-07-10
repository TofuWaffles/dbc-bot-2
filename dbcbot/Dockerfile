# Define build arguments
ARG DATABASE_URL="postgres://postgres:password@localhost:5432/example_so_sqlx_not_be_pissed"
ARG BRAWL_STARS_TOKEN="token"
ARG DISCORD_TOKEN="token"

FROM rust:bullseye AS chef
RUN cargo install cargo-chef
RUN cargo install sqlx-cli --no-default-features --features rustls,postgres

# Set the working directory
WORKDIR /dbcbot
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage for building the project
FROM chef AS builder
ARG DATABASE_URL
ENV DATABASE_URL=$DATABASE_URL
ENV SQLX_OFFLINE=true

COPY --from=planner /dbcbot/recipe.json recipe.json
RUN cargo chef cook --recipe-path recipe.json
COPY . .
RUN cargo install sqlx-cli --no-default-features --features rustls,postgres
RUN cargo build --release

# Final stage for the runtime image
FROM debian:bullseye

ARG DATABASE_URL
ARG BRAWL_STARS_TOKEN
ARG DISCORD_TOKEN

ENV BRAWL_STARS_TOKEN=$BRAWL_STARS_TOKEN
ENV DISCORD_TOKEN=$DISCORD_TOKEN
ENV DATABASE_URL=$DATABASE_URL

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && update-ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /dbcbot/target/release/dbcbot /usr/local/bin/dbcbot
COPY .sqlx /dbcbot/.sqlx

CMD ["dbcbot"]
