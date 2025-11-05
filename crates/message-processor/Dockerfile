# Build stage
FROM rust:nightly-slim as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY tests ./tests
COPY .sqlx ./.sqlx
COPY migrations ./migrations

# Set SQLx to offline mode to skip compile-time database checks
ENV SQLX_OFFLINE=true

# Build the application
RUN cargo build --release --bin ethhook-message-processor

# Runtime stage
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy the built binary from the builder stage
COPY --from=builder /app/target/release/ethhook-message-processor /usr/local/bin/message-processor

# Set the entrypoint
CMD ["/usr/local/bin/message-processor"]
