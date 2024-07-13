# Define build arguments
ARG DATABASE_URL="postgres://postgres:password@localhost:5432/example_so_sqlx_not_be_pissed"
ARG BRAWL_STARS_TOKEN="token"
ARG DISCORD_TOKEN="token"

FROM clux/muslrust:stable AS chef
USER root
RUN cargo install cargo-chef
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
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl

# Final stage for the runtime image
FROM alpine:latest AS runtime
ARG DATABASE_URL
ARG BRAWL_STARS_TOKEN
ARG DISCORD_TOKEN
ENV BRAWL_STARS_TOKEN=$BRAWL_STARS_TOKEN
ENV DISCORD_TOKEN=$DISCORD_TOKEN
ENV DATABASE_URL=$DATABASE_URL
RUN apk update && apk add --no-cache \
    ca-certificates \
    && update-ca-certificates
WORKDIR /dbcbot
COPY --from=builder /dbcbot/target/x86_64-unknown-linux-musl/release/dbcbot /usr/local/bin/dbcbot
COPY .sqlx /dbcbot/.sqlx
ENTRYPOINT ["/usr/local/bin/dbcbot"]
