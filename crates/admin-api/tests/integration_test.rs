/*!
 * Admin API Integration Tests
 * 
 * Tests the complete user journey:
 * 1. User registration
 * 2. User login (JWT authentication)
 * 3. Application CRUD operations
 * 4. Endpoint CRUD operations
 * 5. API key validation
 * 
 * Run with: cargo test -p ethhook-admin-api --test integration_test -- --ignored
 * (Requires PostgreSQL and Redis running)
 */

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::util::ServiceExt; // for `oneshot`

// Helper function to create test database pool
async fn create_test_pool() -> PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://ethhook:password@localhost:5432/ethhook".to_string());
    
    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

// Helper function to clean up test data
async fn cleanup_test_data(pool: &PgPool, email: &str) {
    // Use raw query to avoid SQLx offline mode issues
    let _ = sqlx::query("DELETE FROM users WHERE email = $1")
        .bind(email)
        .execute(pool)
        .await;
}

// Helper to extract JSON response body
async fn get_json_response(response: axum::response::Response) -> Value {
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");
    
    serde_json::from_slice(&body).expect("Failed to parse JSON")
}

#[tokio::test]
#[ignore] // Requires PostgreSQL
async fn test_user_registration_success() {
    let pool = create_test_pool().await;
    let test_email = "test_register@example.com";
    
    // Cleanup any existing test data
    cleanup_test_data(&pool, test_email).await;
    
    let app = ethhook_admin_api::create_test_router(pool.clone());
    
    let payload = json!({
        "email": test_email,
        "password": "SecurePassword123!",
        "name": "Test User"
    });
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = get_json_response(response).await;
    
    // Verify response structure
    assert!(body["user"]["id"].is_string());
    assert_eq!(body["user"]["email"], test_email);
    assert_eq!(body["user"]["name"], "Test User");
    assert!(body["token"].is_string());
    
    // Cleanup
    cleanup_test_data(&pool, test_email).await;
}

#[tokio::test]
#[ignore] // Requires PostgreSQL
async fn test_user_registration_duplicate_email() {
    let pool = create_test_pool().await;
    let test_email = "test_duplicate@example.com";
    
    cleanup_test_data(&pool, test_email).await;
    
    let app = ethhook_admin_api::create_test_router(pool.clone());
    
    let payload = json!({
        "email": test_email,
        "password": "SecurePassword123!",
        "name": "Test User"
    });
    
    // First registration - should succeed
    let response1 = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response1.status(), StatusCode::OK);
    
    // Second registration with same email - should fail
    let response2 = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response2.status(), StatusCode::INTERNAL_SERVER_ERROR);
    
    cleanup_test_data(&pool, test_email).await;
}

#[tokio::test]
#[ignore] // Requires PostgreSQL
async fn test_user_login_success() {
    let pool = create_test_pool().await;
    let test_email = "test_login@example.com";
    let test_password = "SecurePassword123!";
    
    cleanup_test_data(&pool, test_email).await;
    
    let app = ethhook_admin_api::create_test_router(pool.clone());
    
    // First, register a user
    let register_payload = json!({
        "email": test_email,
        "password": test_password,
        "name": "Test User"
    });
    
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(register_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    // Now try to login
    let login_payload = json!({
        "email": test_email,
        "password": test_password
    });
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(login_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = get_json_response(response).await;
    
    // Verify login response
    assert!(body["user"]["id"].is_string());
    assert_eq!(body["user"]["email"], test_email);
    assert!(body["token"].is_string());
    
    cleanup_test_data(&pool, test_email).await;
}

#[tokio::test]
#[ignore] // Requires PostgreSQL
async fn test_user_login_invalid_password() {
    let pool = create_test_pool().await;
    let test_email = "test_invalid_pw@example.com";
    
    cleanup_test_data(&pool, test_email).await;
    
    let app = ethhook_admin_api::create_test_router(pool.clone());
    
    // Register user
    let register_payload = json!({
        "email": test_email,
        "password": "CorrectPassword123!",
        "name": "Test User"
    });
    
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(register_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    // Try to login with wrong password
    let login_payload = json!({
        "email": test_email,
        "password": "WrongPassword123!"
    });
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(login_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    
    cleanup_test_data(&pool, test_email).await;
}

#[tokio::test]
#[ignore] // Requires PostgreSQL
async fn test_get_user_profile_authenticated() {
    let pool = create_test_pool().await;
    let test_email = "test_profile@example.com";
    
    cleanup_test_data(&pool, test_email).await;
    
    let app = ethhook_admin_api::create_test_router(pool.clone());
    
    // Register and login to get JWT token
    let register_payload = json!({
        "email": test_email,
        "password": "SecurePassword123!",
        "name": "Test User"
    });
    
    let register_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(register_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    let register_body = get_json_response(register_response).await;
    let token = register_body["token"].as_str().unwrap();
    
    // Get user profile with JWT
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/users/me")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = get_json_response(response).await;
    assert_eq!(body["email"], test_email);
    assert_eq!(body["name"], "Test User");
    
    cleanup_test_data(&pool, test_email).await;
}

#[tokio::test]
#[ignore] // Requires PostgreSQL
async fn test_get_user_profile_unauthenticated() {
    let pool = create_test_pool().await;
    let app = ethhook_admin_api::create_test_router(pool);
    
    // Try to get profile without JWT token
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/users/me")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[ignore] // Requires PostgreSQL
async fn test_complete_application_workflow() {
    let pool = create_test_pool().await;
    let test_email = "test_app_workflow@example.com";
    
    cleanup_test_data(&pool, test_email).await;
    
    let app = ethhook_admin_api::create_test_router(pool.clone());
    
    // 1. Register user
    let register_payload = json!({
        "email": test_email,
        "password": "SecurePassword123!",
        "name": "Test User"
    });
    
    let register_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(register_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    let register_body = get_json_response(register_response).await;
    let token = register_body["token"].as_str().unwrap().to_string();
    
    // 2. Create application
    let create_app_payload = json!({
        "name": "My Test App",
        "description": "Test application for integration testing"
    });
    
    let create_app_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/applications")
                .header("authorization", format!("Bearer {}", token))
                .header("content-type", "application/json")
                .body(Body::from(create_app_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    if create_app_response.status() != StatusCode::CREATED {
        let error_body = get_json_response(create_app_response).await;
        eprintln!("Application creation failed: {:?}", error_body);
        panic!("Expected 201 CREATED, got different status");
    }
    
    let create_app_body = get_json_response(create_app_response).await;
    let app_id = create_app_body["id"].as_str().unwrap();
    let api_key = create_app_body["api_key"].as_str().unwrap();
    
    assert_eq!(create_app_body["name"], "My Test App");
    assert!(!api_key.is_empty());
    
    // 3. List applications
    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/applications")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(list_response.status(), StatusCode::OK);
    
    let list_body = get_json_response(list_response).await;
    assert_eq!(list_body["total"], 1);
    assert_eq!(list_body["applications"].as_array().unwrap().len(), 1);
    
    // 4. Get specific application
    let get_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/v1/applications/{}", app_id))
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(get_response.status(), StatusCode::OK);
    
    // 5. Update application
    let update_payload = json!({
        "name": "Updated App Name"
    });
    
    let update_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(&format!("/api/v1/applications/{}", app_id))
                .header("authorization", format!("Bearer {}", token))
                .header("content-type", "application/json")
                .body(Body::from(update_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(update_response.status(), StatusCode::OK);
    
    let update_body = get_json_response(update_response).await;
    assert_eq!(update_body["name"], "Updated App Name");
    
    // 6. Delete application
    let delete_response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(&format!("/api/v1/applications/{}", app_id))
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);
    
    cleanup_test_data(&pool, test_email).await;
}

#[tokio::test]
#[ignore] // Requires PostgreSQL
async fn test_jwt_validation() {
    let pool = create_test_pool().await;
    let app = ethhook_admin_api::create_test_router(pool);
    
    // Try to create application with invalid token
    let payload = json!({
        "name": "Test App",
        "description": "Should fail"
    });
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/applications")
                .header("authorization", "Bearer invalid.token.here")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[test]
fn test_password_validation() {
    // Test that password hashing works correctly
    let password = "TestPassword123!";
    let hash = ethhook_admin_api::auth::hash_password(password).unwrap();
    
    // Hash should not equal password
    assert_ne!(hash, password);
    
    // Should be able to verify correct password
    assert!(ethhook_admin_api::auth::verify_password(password, &hash).unwrap());
    
    // Should fail to verify wrong password
    assert!(!ethhook_admin_api::auth::verify_password("WrongPassword123!", &hash).unwrap());
}

#[test]
fn test_email_validation() {
    use validator::Validate;
    
    #[derive(validator::Validate)]
    struct TestEmail {
        #[validate(email)]
        email: String,
    }
    
    // Invalid emails
    assert!(TestEmail { email: "invalid".to_string() }.validate().is_err());
    assert!(TestEmail { email: "invalid@".to_string() }.validate().is_err());
    assert!(TestEmail { email: "@example.com".to_string() }.validate().is_err());
    
    // Valid emails
    assert!(TestEmail { email: "test@example.com".to_string() }.validate().is_ok());
    assert!(TestEmail { email: "user+tag@domain.co.uk".to_string() }.validate().is_ok());
}
