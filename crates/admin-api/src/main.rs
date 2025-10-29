use anyhow::Result;
use axum::{
    Router,
    routing::{delete, get, post, put},
};
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, warn};

mod api_key;
mod auth;
mod config;
mod handlers;
mod metrics;
mod metrics_middleware;

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

    // Run migrations (ignore if already applied)
    match sqlx::migrate!("../../migrations").run(&pool).await {
        Ok(_) => info!("Database migrations completed"),
        Err(sqlx::migrate::MigrateError::VersionMissing(_)) => {
            info!("Database migrations already applied, skipping");
        }
        Err(e) => {
            // Only fail on actual errors, not "already exists" errors
            if !e.to_string().contains("already exists") {
                return Err(e.into());
            }
            info!("Database migrations already applied, skipping");
        }
    }

    // Build application router
    let app = create_router(pool.clone(), config.clone());

    // Create metrics router (separate server on port 9090)
    let metrics_app = Router::new().route("/metrics", get(metrics_handler));

    // Create server addresses
    let addr = format!("{}:{}", config.server_host, config.server_port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    let metrics_addr = format!("{}:9090", config.server_host);
    let metrics_listener = tokio::net::TcpListener::bind(&metrics_addr).await?;

    info!("Admin API server listening on {}", addr);
    info!("Metrics server listening on {}", metrics_addr);

    // Start both servers concurrently with graceful shutdown
    tokio::select! {
        res = axum::serve(listener, app).with_graceful_shutdown(shutdown_signal()) => {
            res?;
        }
        res = axum::serve(metrics_listener, metrics_app).with_graceful_shutdown(shutdown_signal()) => {
            res?;
        }
    }

    info!("Admin API server shut down gracefully");

    Ok(())
}

/// Create the application router with all routes and middleware
fn create_router(pool: sqlx::PgPool, config: Config) -> Router {
    // Build CORS layer based on configuration
    let cors = if config.cors_allowed_origins.contains(&"*".to_string()) {
        // Allow all origins in development
        CorsLayer::permissive()
    } else {
        // Use specific origins in production
        let origins: Vec<_> = config
            .cors_allowed_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();
        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods([
                axum::http::Method::GET,
                axum::http::Method::POST,
                axum::http::Method::PUT,
                axum::http::Method::DELETE,
                axum::http::Method::OPTIONS,
            ])
            .allow_headers([
                axum::http::header::AUTHORIZATION,
                axum::http::header::CONTENT_TYPE,
            ])
            .allow_credentials(true)
    };

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
        .route(
            "/applications",
            post(handlers::applications::create_application),
        )
        .route(
            "/applications",
            get(handlers::applications::list_applications),
        )
        .route(
            "/applications/{id}",
            get(handlers::applications::get_application),
        )
        .route(
            "/applications/{id}",
            put(handlers::applications::update_application),
        )
        .route(
            "/applications/{id}",
            delete(handlers::applications::delete_application),
        )
        .route(
            "/applications/{id}/regenerate-key",
            post(handlers::applications::regenerate_api_key),
        )
        // Endpoint routes
        .route(
            "/endpoints",
            post(handlers::endpoints::create_endpoint)
                .get(handlers::endpoints::list_all_user_endpoints),
        )
        .route(
            "/applications/{app_id}/endpoints",
            get(handlers::endpoints::list_endpoints),
        )
        .route("/endpoints/{id}", get(handlers::endpoints::get_endpoint))
        .route("/endpoints/{id}", put(handlers::endpoints::update_endpoint))
        .route(
            "/endpoints/{id}",
            delete(handlers::endpoints::delete_endpoint),
        )
        .route(
            "/endpoints/{id}/regenerate-secret",
            post(handlers::endpoints::regenerate_hmac_secret),
        )
        // Events routes
        .route("/events", get(handlers::events::list_events))
        .route("/events/{id}", get(handlers::events::get_event))
        .route(
            "/delivery-attempts",
            get(handlers::events::list_delivery_attempts),
        )
        // Statistics routes
        .route(
            "/statistics/dashboard",
            get(handlers::statistics::get_dashboard_statistics),
        )
        .layer(axum::middleware::from_fn(auth::inject_jwt_secret));

    // Combine routes
    let api_routes = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(state);

    // Build application with middleware
    Router::new().nest("/api/v1", api_routes).layer(
        ServiceBuilder::new()
            .layer(axum::middleware::from_fn(metrics_middleware::track_metrics))
            .layer(TraceLayer::new_for_http())
            .layer(cors),
    )
}

/// Health check endpoint
async fn health_check() -> &'static str {
    "OK"
}

/// Metrics endpoint handler
async fn metrics_handler() -> Result<String, (axum::http::StatusCode, String)> {
    metrics::render_metrics()
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
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
