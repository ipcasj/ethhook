# Build stage
FROM rust:1.85-slim as builder

WORKDIR /app

# Install build dependencies (including g++ for ClickHouse cityhash)
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    g++ \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace manifests
COPY Cargo.toml Cargo.lock ./
COPY .sqlx .sqlx

# Copy all crate source directories (workspace requires all members)
COPY crates/common crates/common
COPY crates/config crates/config
COPY crates/domain crates/domain
COPY crates/validate-env crates/validate-env
COPY crates/admin-api crates/admin-api
COPY crates/pipeline crates/pipeline

# Copy migrations
COPY migrations migrations

# Remove tests from workspace for Docker build (excluded by .dockerignore)
RUN sed -i '/"tests",/d' Cargo.toml

# Build release binary with SQLX offline mode (uses cached query metadata)
ENV SQLX_OFFLINE=true
RUN cargo build --release --bin pipeline

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    sqlite3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/pipeline /usr/local/bin/pipeline

# Copy migrations
COPY migrations /app/migrations

# Create data directory
RUN mkdir -p /data && chmod 777 /data

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:${HEALTH_CHECK_PORT:-8080}/health || exit 1

# Run as non-root user
RUN useradd -m -u 1000 ethhook && chown -R ethhook:ethhook /app /data
USER ethhook

# Default environment
ENV RUST_LOG=info \
    CONFIG_DB_PATH=sqlite:/data/config.db \
    HEALTH_CHECK_PORT=8080 \
    METRICS_PORT=9090

EXPOSE 8080 9090

CMD ["pipeline"]
