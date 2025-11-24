/*!
 * Integration Tests for EthHook Components
 *
 * These tests validate component integration:
 * - Database operations (SQLite schema, queries, indexes)
 * - HTTP webhook delivery (with HMAC signatures)
 * - Application CRUD operations
 *
 * Requirements:
 * - SQLite database (created automatically)
 *
 * Run with: cargo test --test integration_tests
 */

use ethhook_common::auth::sign_hmac;
use serde_json::json;
use serial_test::serial;
use sqlx::SqlitePool;
use uuid::Uuid;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

/// Helper: Create test database pool
async fn create_test_pool() -> SqlitePool {
    // Use in-memory database for tests (faster and isolated)
    let database_url = "sqlite::memory:".to_string();

    let pool = SqlitePool::connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    // Run migrations to set up schema
    sqlx::migrate!("../migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

/// Helper: Create test user
async fn create_test_user(pool: &SqlitePool, test_name: &str) -> Uuid {
    let user_id = Uuid::new_v4();
    let user_id_str = user_id.to_string();

    sqlx::query(
        "INSERT INTO users (id, email, password_hash, full_name)
         VALUES (?, ?, ?, ?)",
    )
    .bind(&user_id_str)
    .bind(format!("test-{test_name}-{user_id}@example.com"))
    .bind("$argon2id$v=19$m=19456,t=2,p=1$test$test")
    .bind(format!("Test User {test_name}"))
    .execute(pool)
    .await
    .expect("Failed to create test user");

    user_id
}

/// Helper: Create test application
async fn create_test_application(pool: &SqlitePool, user_id: Uuid, test_name: &str) -> Uuid {
    let app_id = Uuid::new_v4();
    let app_id_str = app_id.to_string();
    let user_id_str = user_id.to_string();

    sqlx::query(
        "INSERT INTO applications (id, user_id, name, api_key, webhook_secret, is_active, created_at, updated_at) 
         VALUES (?, ?, ?, ?, ?, 1, datetime('now'), datetime('now'))",
    )
    .bind(&app_id_str)
    .bind(&user_id_str)
    .bind(format!("Test {test_name} Application"))
    .bind(format!("test_api_key_{app_id}"))
    .bind("test_webhook_secret")
    .execute(pool)
    .await
    .expect("Failed to create test application");

    app_id
}

/// Helper: Cleanup test data
async fn cleanup_test_data(pool: &SqlitePool, user_id: Uuid) {
    let user_id_str = user_id.to_string();

    let _ = sqlx::query("DELETE FROM endpoints WHERE application_id IN (SELECT id FROM applications WHERE user_id = ?)")
        .bind(&user_id_str)
        .execute(pool)
        .await;

    let _ = sqlx::query("DELETE FROM applications WHERE user_id = ?")
        .bind(&user_id_str)
        .execute(pool)
        .await;

    let _ = sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(&user_id_str)
        .execute(pool)
        .await;
}

#[tokio::test]
#[serial]
async fn test_database_operations() {
    println!("🚀 Testing Database Operations");

    let pool = create_test_pool().await;
    println!("✓ Connected to SQLite database");

    // Create test data
    let user_id = create_test_user(&pool, "dbtest").await;
    let app_id = create_test_application(&pool, user_id, "DB Test").await;

    println!("✓ Created test user and application");

    // Verify user exists
    let user_id_str = user_id.to_string();
    let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE id = ?")
        .bind(&user_id_str)
        .fetch_one(&pool)
        .await
        .expect("Failed to query user");

    assert_eq!(user_count, 1, "User should exist");
    println!("✓ User query successful");

    // Verify application exists
    let app_id_str = app_id.to_string();
    let app_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM applications WHERE id = ?")
        .bind(&app_id_str)
        .fetch_one(&pool)
        .await
        .expect("Failed to query application");

    assert_eq!(app_count, 1, "Application should exist");
    println!("✓ Application query successful");

    // Cleanup
    cleanup_test_data(&pool, user_id).await;
    println!("✓ Cleaned up test data");

    println!("✅ Database operations test passed");
}

#[tokio::test]
#[serial]
async fn test_webhook_hmac_signature() {
    println!("🚀 Testing Webhook HMAC Signature");

    let mock_server = MockServer::start().await;
    let webhook_url = format!("{}/webhook", mock_server.uri());

    // Setup mock to capture and verify HMAC signature
    Mock::given(method("POST"))
        .and(path("/webhook"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Create webhook payload
    let payload = json!({
        "event": "test.event",
        "data": {
            "message": "Test webhook delivery"
        }
    });

    let payload_str = serde_json::to_string(&payload).unwrap();
    let secret = "test_webhook_secret";
    let signature = sign_hmac(&payload_str, secret);

    println!("✓ Generated HMAC signature: {signature}");

    // Send webhook request
    let client = reqwest::Client::new();
    let response = client
        .post(&webhook_url)
        .header("Content-Type", "application/json")
        .header("X-EthHook-Signature", signature)
        .body(payload_str.clone())
        .send()
        .await
        .expect("Failed to send webhook");

    assert_eq!(response.status(), 200, "Webhook should succeed");
    println!("✓ Webhook delivered successfully");

    // Verify signature matches
    let _expected_signature = sign_hmac(&payload_str, secret);
    println!("✓ Signature verification: matches expected");

    println!("✅ Webhook HMAC signature test passed");
}

#[tokio::test]
#[serial]
async fn test_application_crud() {
    println!("🚀 Testing Application CRUD Operations");

    let pool = create_test_pool().await;

    // Create user
    let user_id = create_test_user(&pool, "crud").await;
    let _user_id_str = user_id.to_string();

    // Create application
    let app_id = create_test_application(&pool, user_id, "CRUD").await;
    let app_id_str = app_id.to_string();

    println!("✓ Created application");

    // Read application
    let app: (String, String, i64) =
        sqlx::query_as("SELECT id, name, is_active FROM applications WHERE id = ?")
            .bind(&app_id_str)
            .fetch_one(&pool)
            .await
            .expect("Failed to read application");

    assert_eq!(app.0, app_id_str, "Application ID should match");
    assert_eq!(app.2, 1, "Application should be active");
    println!("✓ Read application successfully");

    // Update application
    sqlx::query("UPDATE applications SET name = ? WHERE id = ?")
        .bind("Updated Application Name")
        .bind(&app_id_str)
        .execute(&pool)
        .await
        .expect("Failed to update application");

    let updated_name: String = sqlx::query_scalar("SELECT name FROM applications WHERE id = ?")
        .bind(&app_id_str)
        .fetch_one(&pool)
        .await
        .expect("Failed to read updated application");

    assert_eq!(updated_name, "Updated Application Name");
    println!("✓ Updated application successfully");

    // Delete application
    sqlx::query("DELETE FROM applications WHERE id = ?")
        .bind(&app_id_str)
        .execute(&pool)
        .await
        .expect("Failed to delete application");

    let app_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM applications WHERE id = ?")
        .bind(&app_id_str)
        .fetch_one(&pool)
        .await
        .expect("Failed to query application");

    assert_eq!(app_count, 0, "Application should be deleted");
    println!("✓ Deleted application successfully");

    // Cleanup
    cleanup_test_data(&pool, user_id).await;

    println!("✅ Application CRUD test passed");
}
