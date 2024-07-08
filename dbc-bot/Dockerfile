FROM rust:1.79.0 as builder
WORKDIR /usr/src/dbc-bot
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
RUN apt-get update && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/myapp /usr/local/bin/dbc-bot
CMD ["dbc-bot"]