# Build stage
FROM rust:1.83-slim as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace manifests
COPY Cargo.toml Cargo.lock ./

# Copy all crate source directories
COPY crates/common crates/common
COPY crates/config crates/config
COPY crates/domain crates/domain
COPY crates/pipeline crates/pipeline

# Copy migrations
COPY migrations migrations

# Build release binary
ENV DATABASE_URL=sqlite:config.db
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
