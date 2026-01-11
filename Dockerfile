# Stage 1: сборка
FROM rust:1.89 AS builder

WORKDIR /usr/src/server

COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo "fn main() {}" > src/main.rs
RUN cargo fetch

COPY . .
RUN cargo build --release

# Stage 2: runtime
FROM debian:bookworm-slim

# копируем бинарь из builder stage (именно "builder")
COPY --from=builder /usr/src/server/target/release/ws_cards_server /usr/local/bin/ws_cards_server

EXPOSE 8080
CMD ["/usr/local/bin/ws_cards_server"]
