FROM rust:1.89 AS builder

WORKDIR /usr/src/server

COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo "fn main() {}" > src/main.rs
RUN cargo fetch

COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

COPY --from=builder /usr/src/server/target/release/ws_cards_server /usr/local/bin/ws_cards_server

EXPOSE 8080
CMD ["/usr/local/bin/ws_cards_server"]
