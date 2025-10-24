# ============================================================================
# Leptos Frontend - Production Dockerfile for Railway
# ============================================================================
# This Dockerfile builds the Leptos WASM frontend and serves it with Nginx
#
# Build time: ~5-8 minutes
# Image size: ~25MB (compressed)
#
# Railway will automatically:
# - Build this image
# - Deploy it
# - Provide HTTPS
# - Give you a public URL

# ============================================================================
# Stage 1: Build WASM with Rust + Trunk
# ============================================================================
FROM rust:1.83-slim as builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install trunk (WASM build tool) and wasm target
RUN cargo install --locked trunk
RUN rustup target add wasm32-unknown-unknown

# Copy ONLY frontend files (not entire workspace)
COPY Cargo.toml ./Cargo.toml
COPY index.html ./index.html
COPY style.css ./style.css
COPY src/ ./src/

# Build frontend in release mode
# This compiles Rust → WASM and bundles with HTML/CSS
RUN trunk build --release

# List what was built (for debugging)
RUN ls -lah dist/

# ============================================================================
# Stage 2: Serve with Nginx (tiny runtime image)
# ============================================================================
FROM nginx:alpine

# Copy built static files from builder stage
COPY --from=builder /build/dist /usr/share/nginx/html

# Copy nginx configuration
COPY nginx.conf /etc/nginx/nginx.conf

# Add health check file
RUN echo "healthy" > /usr/share/nginx/html/health

# Expose HTTP port (Railway will handle HTTPS)
EXPOSE 80

# Health check for Railway
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --quiet --tries=1 --spider http://localhost/health || exit 1

# Start nginx in foreground
CMD ["nginx", "-g", "daemon off;"]

# ============================================================================
# Build instructions:
#
# Local build (for testing):
#   cd crates/leptos-portal
#   docker build -t ethhook-frontend .
#   docker run -p 8080:80 ethhook-frontend
#   open http://localhost:8080
#
# Railway will build automatically when you:
#   - Set Dockerfile path: crates/leptos-portal/Dockerfile
#   - Push to GitHub
# ============================================================================
