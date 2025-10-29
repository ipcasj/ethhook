# ============================================================================
# Leptos Frontend - Production Dockerfile
# ============================================================================
# This Dockerfile builds the Leptos WASM frontend and serves it with Nginx
#
# Build time: ~5-8 minutes
# Image size: ~25MB (compressed)

# ============================================================================
# Stage 1: Build WASM with Rust + Trunk
# ============================================================================
FROM rust:1.83-slim as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install trunk (WASM build tool) and wasm target
RUN cargo install --locked trunk
RUN rustup target add wasm32-unknown-unknown

# Copy workspace files (we need the domain crate dependency)
COPY Cargo.toml Cargo.lock ./
COPY crates/domain ./crates/domain
COPY crates/leptos-portal ./crates/leptos-portal

# Build frontend in release mode
WORKDIR /app/crates/leptos-portal
RUN trunk build --release

# List what was built (for debugging)
RUN ls -lah dist/

# ============================================================================
# Stage 2: Serve with Nginx (tiny runtime image)
# ============================================================================
FROM nginx:alpine

# Copy built static files from builder stage
COPY --from=builder /app/crates/leptos-portal/dist /usr/share/nginx/html

# Copy nginx configuration
COPY crates/leptos-portal/nginx.conf /etc/nginx/nginx.conf

# Add health check file
RUN echo "healthy" > /usr/share/nginx/html/health

# Expose HTTP port
EXPOSE 80

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --quiet --tries=1 --spider http://localhost/health || exit 1

# Start nginx in foreground
CMD ["nginx", "-g", "daemon off;"]
