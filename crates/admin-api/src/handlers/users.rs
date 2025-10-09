use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::auth::{AuthUser, generate_token, hash_password, verify_password};
use crate::config::Config;

/// Request to register a new user
#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
    #[validate(length(min = 1, message = "Name cannot be empty"))]
    pub name: String,
}

/// Request to login
#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    pub password: String,
}

/// User response
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Auth response with token
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub user: UserResponse,
    pub token: String,
}

/// Error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// Register a new user
pub async fn register(
    State(pool): State<PgPool>,
    State(config): State<Config>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Validate input
    payload.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Validation error: {}", e),
            }),
        )
    })?;

    // Hash password
    let password_hash = hash_password(&payload.password).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to hash password: {}", e),
            }),
        )
    })?;

    // Insert user
    let user = sqlx::query!(
        r#"
        INSERT INTO users (email, password_hash, name)
        VALUES ($1, $2, $3)
        RETURNING id, email, name, created_at
        "#,
        payload.email,
        password_hash,
        payload.name
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to create user: {}", e),
            }),
        )
    })?;

    // Generate JWT token
    let token = generate_token(
        user.id,
        user.email.clone(),
        &config.jwt_secret,
        config.jwt_expiration_hours,
    )
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to generate token: {}", e),
            }),
        )
    })?;

    Ok(Json(AuthResponse {
        user: UserResponse {
            id: user.id,
            email: user.email,
            name: user.name.unwrap_or_default(),
            created_at: user.created_at.unwrap_or_else(|| chrono::Utc::now()),
        },
        token,
    }))
}

/// Login user
pub async fn login(
    State(pool): State<PgPool>,
    State(config): State<Config>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Validate input
    payload.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Validation error: {}", e),
            }),
        )
    })?;

    // Find user by email
    let user = sqlx::query!(
        r#"
        SELECT id, email, name, password_hash, created_at
        FROM users
        WHERE email = $1
        "#,
        payload.email
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Invalid email or password".to_string(),
            }),
        ),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Database error: {}", e),
            }),
        ),
    })?;

    // Verify password
    let is_valid = verify_password(&payload.password, &user.password_hash).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Password verification failed".to_string(),
            }),
        )
    })?;

    if !is_valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Invalid email or password".to_string(),
            }),
        ));
    }

    // Generate JWT token
    let token = generate_token(
        user.id,
        user.email.clone(),
        &config.jwt_secret,
        config.jwt_expiration_hours,
    )
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to generate token: {}", e),
            }),
        )
    })?;

    Ok(Json(AuthResponse {
        user: UserResponse {
            id: user.id,
            email: user.email,
            name: user.name.unwrap_or_default(),
            created_at: user.created_at.unwrap_or_else(|| chrono::Utc::now()),
        },
        token,
    }))
}

/// Get user profile
pub async fn get_profile(
    State(pool): State<PgPool>,
    auth_user: AuthUser,
) -> Result<Json<UserResponse>, (StatusCode, Json<ErrorResponse>)> {
    let user = sqlx::query!(
        r#"
        SELECT id, email, name, created_at
        FROM users
        WHERE id = $1
        "#,
        auth_user.user_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to fetch user: {}", e),
            }),
        )
    })?;

    Ok(Json(UserResponse {
        id: user.id,
        email: user.email,
        name: user.name.unwrap_or_default(),
        created_at: user.created_at.unwrap_or_else(|| chrono::Utc::now()),
    }))
}

/// Request to update user profile
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateProfileRequest {
    #[validate(length(min = 1, message = "Name cannot be empty"))]
    pub name: String,
}

/// Update user profile
pub async fn update_profile(
    State(pool): State<PgPool>,
    auth_user: AuthUser,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<Json<UserResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Validate input
    payload.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Validation error: {}", e),
            }),
        )
    })?;

    let name = payload.name.trim().to_string();
    if name.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Name cannot be empty".to_string(),
            }),
        ));
    };

    let user = sqlx::query!(
        r#"
        UPDATE users
        SET name = $1, updated_at = NOW()
        WHERE id = $2
        RETURNING id, email, name, created_at
        "#,
        name,
        auth_user.user_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to update user: {}", e),
            }),
        )
    })?;

    Ok(Json(UserResponse {
        id: user.id,
        email: user.email,
        name: user.name.unwrap_or_default(),
        created_at: user.created_at.unwrap_or_else(|| chrono::Utc::now()),
    }))
}
