use anyhow::Result;
use axum::{
    routing::{delete, get, post, put},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::{info, warn};

mod api_key;
mod auth;
mod config;
mod handlers;

use config::Config;

/// Application state shared across handlers
#[derive(Clone)]
struct AppState {
    pool: sqlx::PgPool,
    config: Config,
}

// Implement FromRef to allow extracting individual pieces from AppState
impl axum::extract::FromRef<AppState> for sqlx::PgPool {
    fn from_ref(state: &AppState) -> Self {
        state.pool.clone()
    }
}

impl axum::extract::FromRef<AppState> for Config {
    fn from_ref(state: &AppState) -> Self {
        state.config.clone()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    info!("Starting Admin API server...");

    // Load configuration
    let config = Config::from_env()?;
    info!(
        "Configuration loaded - server will bind to {}:{}",
        config.server_host, config.server_port
    );

    // Create database connection pool
    let pool = PgPoolOptions::new()
        .max_connections(config.database_max_connections)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&config.database_url)
        .await?;

    info!("Database connection pool established");

    // Run migrations
    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await?;

    info!("Database migrations completed");

    // Build application router
    let app = create_router(pool.clone(), config.clone());

    // Create server address
    let addr = format!("{}:{}", config.server_host, config.server_port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!("Admin API server listening on {}", addr);

    // Start server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Admin API server shut down gracefully");

    Ok(())
}

/// Create the application router with all routes and middleware
fn create_router(pool: sqlx::PgPool, config: Config) -> Router {
    let state = AppState { pool, config };
    
    // Public routes (no authentication required)
    let public_routes = Router::new()
        .route("/health", get(health_check))
        .route("/auth/register", post(handlers::users::register))
        .route("/auth/login", post(handlers::users::login));

    // Protected routes (JWT authentication required)
    let protected_routes = Router::new()
        // User routes
        .route("/users/me", get(handlers::users::get_profile))
        .route("/users/me", put(handlers::users::update_profile))
        // Application routes
        .route("/applications", post(handlers::applications::create_application))
        .route("/applications", get(handlers::applications::list_applications))
        .route("/applications/:id", get(handlers::applications::get_application))
        .route("/applications/:id", put(handlers::applications::update_application))
        .route("/applications/:id", delete(handlers::applications::delete_application))
        .route("/applications/:id/regenerate-key", post(handlers::applications::regenerate_api_key))
        // Endpoint routes
        .route("/endpoints", post(handlers::endpoints::create_endpoint))
        .route("/applications/:app_id/endpoints", get(handlers::endpoints::list_endpoints))
        .route("/endpoints/:id", get(handlers::endpoints::get_endpoint))
        .route("/endpoints/:id", put(handlers::endpoints::update_endpoint))
        .route("/endpoints/:id", delete(handlers::endpoints::delete_endpoint))
        .route("/endpoints/:id/regenerate-secret", post(handlers::endpoints::regenerate_hmac_secret))
        .layer(axum::middleware::from_fn(auth::inject_jwt_secret));

    // Combine routes
    let api_routes = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(state);

    // Build application with middleware
    Router::new()
        .nest("/api/v1", api_routes)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(
                    CorsLayer::new()
                        .allow_origin(Any)
                        .allow_methods(Any)
                        .allow_headers(Any),
                ),
        )
}

/// Health check endpoint
async fn health_check() -> &'static str {
    "OK"
}

/// Graceful shutdown signal handler
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            warn!("Received Ctrl+C, shutting down...");
        },
        _ = terminate => {
            warn!("Received terminate signal, shutting down...");
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check() {
        let result = health_check().await;
        assert_eq!(result, "OK");
    }
}
