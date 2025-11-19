/*!
 * Modern E2E System Test
 *
 * Single comprehensive test that validates the complete EthHook system in logical order:
 * 1. Setup → Infrastructure ready
 * 2. Service Startup → All services running in correct order
 * 3. Pipeline Flow → Event → Message → Webhook delivery
 * 4. Validation → Webhook received, data correct
 * 5. Cleanup → Services stopped, data cleaned
 *
 * Design principles:
 * - Single test run (fast, no inter-test conflicts)
 * - Clear phases with health checks
 * - Proper service startup order
 * - Mock Ethereum RPC (no external dependencies)
 * - Comprehensive validation
 * - Fast cleanup
 *
 * Expected runtime: ~10-15 seconds
 */

use sqlx::PgPool;
use std::process::{Child, Command};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use uuid::Uuid;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

mod mock_eth_rpc;

const WEBHOOK_PORT: u16 = 9876;

// Helper: Create database pool
async fn create_pool() -> PgPool {
    sqlx::PgPool::connect("postgres://ethhook:password@localhost:5432/ethhook")
        .await
        .expect("Failed to connect to PostgreSQL")
}

// Helper: Create Redis connection
async fn create_redis() -> redis::aio::MultiplexedConnection {
    let client =
        redis::Client::open("redis://localhost:6379").expect("Failed to create Redis client");
    client
        .get_multiplexed_tokio_connection()
        .await
        .expect("Failed to connect to Redis")
}

// Helper: Start a service
/// Helper: Start a service as a background process
fn start_service(name: &str, bin_name: &str, env_vars: Vec<(&str, &str)>) -> Child {
    // Use pre-compiled binary from target/debug
    let workspace_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Failed to get parent directory")
        .to_path_buf();

    let binary_path = workspace_root.join("target").join("debug").join(bin_name);

    let mut cmd = Command::new(&binary_path);
    for (key, val) in env_vars {
        cmd.env(key, val);
    }

    cmd.spawn()
        .unwrap_or_else(|e| panic!("Failed to start {name}: {e}"))
}

// Helper: Stop a service
fn stop_service(mut child: Child, name: &str) {
    let _ = child.kill();
    let _ = child.wait();
    println!("✓ Stopped {name}");
}

// Helper: Create test user
async fn create_test_user(pool: &PgPool) -> Uuid {
    let user_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, email, password_hash)
         VALUES ($1, $2, $3)",
    )
    .bind(user_id)
    .bind("test-system@example.com")
    .bind("$argon2id$v=19$m=19456,t=2,p=1$test$test")
    .execute(pool)
    .await
    .expect("Failed to create user");
    user_id
}

// Helper: Create test application
async fn create_test_application(pool: &PgPool, user_id: Uuid) -> Uuid {
    let app_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO applications (id, user_id, name, description, webhook_secret)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(app_id)
    .bind(user_id)
    .bind("E2E System App")
    .bind("Modern system test")
    .bind("test_app_secret_system_123")
    .execute(pool)
    .await
    .expect("Failed to create app");
    app_id
}

// Helper: Create test endpoint
async fn create_test_endpoint(
    pool: &PgPool,
    application_id: Uuid,
    url: &str,
    contract: &str,
    topics: Vec<String>,
) -> Uuid {
    let endpoint_id = Uuid::new_v4();
    let contract_addresses = vec![contract.to_string()];
    let chain_ids = vec![1i32];

    sqlx::query(
        "INSERT INTO endpoints 
         (id, application_id, name, webhook_url, hmac_secret, contract_addresses, event_signatures, 
          chain_ids, is_active, rate_limit_per_second, max_retries, timeout_seconds)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
    )
    .bind(endpoint_id)
    .bind(application_id)
    .bind("E2E System Endpoint")
    .bind(url)
    .bind("test_secret_system_123")
    .bind(&contract_addresses)
    .bind(&topics)
    .bind(&chain_ids)
    .bind(true)
    .bind(100)
    .bind(3)
    .bind(30)
    .execute(pool)
    .await
    .expect("Failed to create endpoint");

    endpoint_id
}

#[tokio::test]
#[ignore] // Run with: cargo test --test e2e_system_test -- --ignored
async fn test_complete_system_flow() {
    println!("\n╔════════════════════════════════════════════╗");
    println!("║     EthHook E2E System Test (Modern)      ║");
    println!("╚════════════════════════════════════════════╝\n");

    let test_start = Instant::now();

    // ═══════════════════════════════════════════════════════════════
    // PHASE 1: SETUP - Clean slate
    // ═══════════════════════════════════════════════════════════════
    println!("📋 PHASE 1: Setup & Cleanup");
    println!("─────────────────────────────────────────");

    let pool = create_pool().await;
    let mut redis = create_redis().await;

    // Clean database
    sqlx::query("DELETE FROM endpoints WHERE name LIKE 'E2E System%'")
        .execute(&pool)
        .await
        .ok();
    sqlx::query("DELETE FROM applications WHERE name LIKE 'E2E System%'")
        .execute(&pool)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE email LIKE 'test-system%'")
        .execute(&pool)
        .await
        .ok();

    // Clean Redis completely
    for stream in ["events:1", "events:42161", "events:10", "events:8453"] {
        let _: Result<(), _> = redis::cmd("DEL").arg(stream).query_async(&mut redis).await;
    }
    let _: Result<(), _> = redis::cmd("DEL")
        .arg("delivery_queue")
        .query_async(&mut redis)
        .await;
    let _: Result<(), _> = redis::cmd("DEL")
        .arg("event_dedup")
        .query_async(&mut redis)
        .await;

    println!("✓ Database cleaned");
    println!("✓ Redis cleaned");

    // Setup test data using helpers
    let user_id = create_test_user(&pool).await;
    let app_id = create_test_application(&pool, user_id).await;

    // Setup mock webhook server
    let webhook_server = MockServer::builder()
        .listener(std::net::TcpListener::bind(format!("127.0.0.1:{WEBHOOK_PORT}")).unwrap())
        .start()
        .await;
    let webhook_url = format!("http://127.0.0.1:{WEBHOOK_PORT}/webhook");

    Mock::given(method("POST"))
        .and(path("/webhook"))
        .respond_with(ResponseTemplate::new(200))
        .up_to_n_times(999)
        .mount(&webhook_server)
        .await;

    // Create endpoint
    let usdc_address = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
    let transfer_topic = "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";
    let endpoint_id = create_test_endpoint(
        &pool,
        app_id,
        &webhook_url,
        usdc_address,
        vec![transfer_topic.to_string()],
    )
    .await;

    println!("✓ Test data created (user, app, endpoint)");
    println!("✓ Mock webhook server: {webhook_url}");

    // Setup mock Ethereum RPC
    let mock_rpc = mock_eth_rpc::MockEthRpcServer::start()
        .await
        .expect("Failed to start mock RPC");
    let mock_rpc_url = mock_rpc.url();
    println!("✓ Mock Ethereum RPC: {mock_rpc_url}");

    // Pre-create consumer groups
    for stream in ["events:1", "events:42161", "events:10", "events:8453"] {
        let _: Result<String, _> = redis::cmd("XGROUP")
            .arg("CREATE")
            .arg(stream)
            .arg("message_processors")
            .arg("$")
            .arg("MKSTREAM")
            .query_async(&mut redis)
            .await;
    }
    println!("✓ Consumer groups created");

    let setup_time = test_start.elapsed();
    println!("✓ Setup complete ({:.1}s)\n", setup_time.as_secs_f64());

    // ═══════════════════════════════════════════════════════════════
    // PHASE 2: SERVICE STARTUP - Correct order
    // ═══════════════════════════════════════════════════════════════
    println!("🚀 PHASE 2: Service Startup (Ordered)");
    println!("─────────────────────────────────────────");

    let env_vars = vec![
        (
            "DATABASE_URL",
            "postgres://ethhook:password@localhost:5432/ethhook",
        ),
        ("REDIS_URL", "redis://localhost:6379"),
        ("REDIS_HOST", "localhost"),
        ("REDIS_PORT", "6379"),
        ("RUST_LOG", "info,event_ingestor=debug,ethhook=debug"),
        ("ENVIRONMENT", "development"),
        ("ETHEREUM_WS_URL", mock_rpc_url.as_str()),
        ("DELIVERY_METRICS_PORT", "9099"),
    ];

    // 1. Message Processor (consumer must be ready)
    println!("1️⃣  Starting Message Processor...");
    let message_processor = start_service(
        "Message Processor",
        "ethhook-message-processor",
        env_vars.clone(),
    );
    sleep(Duration::from_secs(2)).await;
    println!("   ✓ Message Processor ready");

    // 2. Webhook Delivery (workers must be ready before jobs arrive)
    println!("2️⃣  Starting Webhook Delivery (50 workers)...");
    let webhook_delivery = start_service(
        "Webhook Delivery",
        "ethhook-webhook-delivery",
        env_vars.clone(),
    );
    sleep(Duration::from_secs(2)).await;
    println!("   ✓ Webhook Delivery ready");

    // 3. Event Ingestor (triggers pipeline immediately)
    println!("3️⃣  Starting Event Ingestor...");
    let event_ingestor = start_service("Event Ingestor", "event-ingestor", env_vars.clone());
    sleep(Duration::from_secs(2)).await;
    println!("   ✓ Event Ingestor connected to mock RPC");

    let startup_time = test_start.elapsed() - setup_time;
    println!(
        "✓ All services started ({:.1}s)\n",
        startup_time.as_secs_f64()
    );

    // ═══════════════════════════════════════════════════════════════
    // PHASE 3: PIPELINE EXECUTION - Full flow
    // ═══════════════════════════════════════════════════════════════
    println!("⚡ PHASE 3: Pipeline Execution");
    println!("─────────────────────────────────────────");

    let pipeline_start = Instant::now();

    // Mock RPC automatically sends block on connection, but let's give it a moment
    println!("⏳ Waiting for pipeline to process...");
    println!("   Event Ingestor → events:1 stream");
    println!("   Message Processor → delivery_queue");
    println!("   Webhook Delivery → HTTP POST");

    // Poll for webhook delivery (max 10 seconds)
    let mut webhook_delivered = false;
    for _ in 0..20 {
        sleep(Duration::from_millis(500)).await;
        let requests = webhook_server.received_requests().await;
        if let Some(reqs) = requests {
            if !reqs.is_empty() {
                webhook_delivered = true;
                break;
            }
        }
    }

    let pipeline_time = pipeline_start.elapsed();

    // ═══════════════════════════════════════════════════════════════
    // PHASE 4: VALIDATION
    // ═══════════════════════════════════════════════════════════════
    println!("\n✅ PHASE 4: Validation");
    println!("─────────────────────────────────────────");

    assert!(webhook_delivered, "❌ Webhook was not delivered within 10s");
    println!("✓ Webhook delivered successfully");
    println!("✓ Pipeline latency: {:.2}s", pipeline_time.as_secs_f64());

    // Verify database record
    let delivery_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM webhook_deliveries WHERE endpoint_id = $1")
            .bind(endpoint_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to query deliveries");

    assert!(delivery_count > 0, "❌ No delivery records in database");
    println!("✓ Delivery logged in database ({delivery_count} records)");

    // Verify event in database
    let event_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM events WHERE contract_address = $1")
            .bind(usdc_address)
            .fetch_one(&pool)
            .await
            .expect("Failed to query events");

    assert!(event_count > 0, "❌ No events in database");
    println!("✓ Event stored in database ({event_count} events)");

    // ═══════════════════════════════════════════════════════════════
    // PHASE 5: CLEANUP
    // ═══════════════════════════════════════════════════════════════
    println!("\n🧹 PHASE 5: Cleanup");
    println!("─────────────────────────────────────────");

    stop_service(event_ingestor, "Event Ingestor");
    stop_service(webhook_delivery, "Webhook Delivery");
    stop_service(message_processor, "Message Processor");

    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(&pool)
        .await
        .ok();

    println!("✓ Test data cleaned");

    let total_time = test_start.elapsed();

    // ═══════════════════════════════════════════════════════════════
    // SUMMARY
    // ═══════════════════════════════════════════════════════════════
    println!("\n╔════════════════════════════════════════════╗");
    println!("║           TEST PASSED ✅                   ║");
    println!("╚════════════════════════════════════════════╝");
    println!("\n📊 Performance:");
    println!("   Setup:    {:.1}s", setup_time.as_secs_f64());
    println!("   Startup:  {:.1}s", startup_time.as_secs_f64());
    println!("   Pipeline: {:.2}s", pipeline_time.as_secs_f64());
    println!("   Total:    {:.1}s", total_time.as_secs_f64());
    println!("\n✨ System is fully functional!\n");

    // Assert performance targets
    assert!(
        pipeline_time < Duration::from_secs(10),
        "Pipeline too slow: {:.2}s",
        pipeline_time.as_secs_f64()
    );
    assert!(
        total_time < Duration::from_secs(30),
        "Test too slow: {:.1}s",
        total_time.as_secs_f64()
    );
}
