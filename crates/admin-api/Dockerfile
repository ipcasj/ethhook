# Build stage
FROM rust:1.83-slim as builder

WORKDIR /app

# Install nightly for edition2024 support
RUN rustup install nightly && rustup default nightly

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
RUN cargo build --release --bin ethhook-admin-api

# Runtime stage
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy the built binary from the builder stage
COPY --from=builder /app/target/release/ethhook-admin-api /usr/local/bin/admin-api

# Expose port
EXPOSE 8080

# Run the application
# Set the entrypoint
CMD ["/usr/local/bin/admin-api"]
