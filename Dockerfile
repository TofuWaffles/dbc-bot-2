FROM rust:bullseye as builder
WORKDIR /usr/src/dbc-bot
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/dbc-bot /usr/local/bin/dbc-bot
CMD ["dbc-bot"]