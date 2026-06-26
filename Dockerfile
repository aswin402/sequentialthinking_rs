FROM rust:1.80-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim AS release
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
RUN mkdir -p /data

COPY --from=builder /app/target/release/mcp-server-sequential-thinking /usr/local/bin/mcp-server-sequential-thinking

# Expose port for HTTP/SSE transport
EXPOSE 3000

# Environment defaults
ENV TRANSPORT=http
ENV PORT=3000
ENV LOG_FORMAT=pretty
ENV STORAGE=sqlite
ENV DB_PATH=/data/history.db

ENTRYPOINT ["mcp-server-sequential-thinking"]
