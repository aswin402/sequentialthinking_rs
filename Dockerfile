FROM rust:1.87-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim AS release
COPY --from=builder /app/target/release/mcp-server-sequential-thinking /usr/local/bin/mcp-server-sequential-thinking
ENTRYPOINT ["mcp-server-sequential-thinking"]
