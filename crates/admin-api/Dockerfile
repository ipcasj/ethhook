# Build stage
FROM rust:1.85-slim AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files (workspace requires all members)
COPY Cargo.toml Cargo.lock ./
COPY crates/common crates/common
COPY crates/config crates/config
COPY crates/domain crates/domain
COPY crates/admin-api crates/admin-api
COPY crates/pipeline crates/pipeline
COPY migrations migrations

# Build the application (SQLite doesn't need offline mode)
ENV DATABASE_URL=sqlite:config.db
RUN cargo build --release --bin ethhook-admin-api

# Runtime stage
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    sqlite3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy the built binary from the builder stage
COPY --from=builder /app/target/release/ethhook-admin-api /usr/local/bin/admin-api

# Copy migrations
COPY migrations /app/migrations

# Create data directory
RUN mkdir -p /data && chmod 777 /data

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:${PORT:-3000}/health || exit 1

# Run as non-root user
RUN useradd -m -u 1000 ethhook && chown -R ethhook:ethhook /app /data
USER ethhook

# Default environment
ENV RUST_LOG=info \
    DATABASE_URL=sqlite:/data/config.db \
    DATABASE_MAX_CONNECTIONS=5 \
    PORT=3000

# Expose port
EXPOSE 3000

# Run the application
CMD ["/usr/local/bin/admin-api"]
