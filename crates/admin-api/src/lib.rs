/*!
 * Admin API Service
 *
 * REST API for managing users, applications, and webhook endpoints.
 *
 * ## Architecture
 *
 * ```text
 * Client                Admin API              Database
 * ──────               ──────────              ────────
 *                           │
 * POST /auth/register ─────>│
 *                           ├─────> INSERT user
 *                           │<───── user_id
 *                           │
 * <─── JWT token ───────────┤
 *                           │
 * POST /applications ──────>│
 * Authorization: Bearer JWT │
 *                           ├─────> INSERT application
 *                           │<───── app_id, api_key
 *                           │
 * <─── application ─────────┤
 *                           │
 * POST /endpoints ─────────>│
 * Authorization: Bearer JWT │
 *                           ├─────> INSERT endpoint
 *                           │<───── endpoint_id, hmac_secret
 *                           │
 * <─── endpoint ────────────┤
 * ```
 *
 * ## API Endpoints
 *
 * ### Authentication
 * - `POST /api/v1/auth/register` - Register new user
 * - `POST /api/v1/auth/login` - Login and get JWT token
 *
 * ### Users (Protected)
 * - `GET /api/v1/users/me` - Get current user profile
 * - `PUT /api/v1/users/me` - Update user profile
 *
 * ### Applications (Protected)
 * - `POST /api/v1/applications` - Create application
 * - `GET /api/v1/applications` - List user's applications
 * - `GET /api/v1/applications/:id` - Get application details
 * - `PUT /api/v1/applications/:id` - Update application
 * - `DELETE /api/v1/applications/:id` - Delete application
 * - `POST /api/v1/applications/:id/regenerate-key` - Regenerate API key
 *
 * ### Endpoints (Protected)
 * - `POST /api/v1/endpoints` - Create webhook endpoint
 * - `GET /api/v1/applications/:app_id/endpoints` - List endpoints
 * - `GET /api/v1/endpoints/:id` - Get endpoint details
 * - `PUT /api/v1/endpoints/:id` - Update endpoint
 * - `DELETE /api/v1/endpoints/:id` - Delete endpoint
 * - `POST /api/v1/endpoints/:id/regenerate-secret` - Regenerate HMAC secret
 *
 * ## Authentication
 *
 * Protected endpoints require JWT authentication:
 * ```
 * Authorization: Bearer <jwt_token>
 * ```
 *
 * JWT tokens are issued on registration and login, valid for 24 hours by default.
 *
 * ## Security Features
 *
 * - **Password Hashing**: bcrypt with cost factor 12
 * - **JWT Tokens**: HS256 algorithm with configurable expiration
 * - **API Keys**: Secure random generation with prefix
 * - **HMAC Secrets**: 64-character random strings
 * - **CORS**: Configurable allowed origins
 * - **Rate Limiting**: Per-user request limits (TODO)
 */

pub mod api_key;
pub mod auth;
pub mod config;
pub mod handlers;
pub mod state;

// Re-export for testing
pub use auth::{hash_password, verify_password};
pub use config::Config;
pub use state::AppState;

use axum::Router;
use sqlx::PgPool;

/// Create a test router for integration tests
pub fn create_test_router(pool: PgPool) -> Router {
    use axum::{
        Router,
        routing::{delete, get, post, put},
    };
    use tower::ServiceBuilder;
    use tower_http::{
        cors::{Any, CorsLayer},
        trace::TraceLayer,
    };

    let config = Config {
        database_url: String::new(), // Not used in tests
        database_max_connections: 5,
        redis_url: "redis://localhost:6379".to_string(),
        jwt_secret: std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "test-secret-key-for-testing-only".to_string()),
        jwt_expiration_hours: 24,
        server_host: "127.0.0.1".to_string(),
        server_port: 8080,
        api_key_prefix: "ethh".to_string(),
        rate_limit_per_minute: 60,
        cors_allowed_origins: vec!["*".to_string()],
    };

    let state = AppState { pool, config };

    let public_routes = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/auth/register", post(handlers::users::register))
        .route("/auth/login", post(handlers::users::login));

    let protected_routes = Router::new()
        .route("/users/me", get(handlers::users::get_profile))
        .route("/users/me", put(handlers::users::update_profile))
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
        .route("/endpoints", post(handlers::endpoints::create_endpoint))
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
        );

    Router::new()
        .nest(
            "/api/v1",
            Router::new()
                .merge(public_routes)
                .merge(protected_routes)
                .layer(axum::middleware::from_fn(auth::inject_jwt_secret))
                .with_state(state),
        )
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
