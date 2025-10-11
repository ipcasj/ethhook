/*!
 * Comprehensive tests for EndpointMatcher
 *
 * Tests cover:
 * - Endpoint matching logic (contract address, event topics)
 * - NULL handling (match all contracts, match all events)
 * - Active/inactive endpoint filtering
 * - Edge cases and error handling
 */

use ethhook_message_processor::consumer::StreamEvent;
use ethhook_message_processor::matcher::EndpointMatcher;
use sqlx::PgPool;
use uuid::Uuid;

/// Helper: Create test database pool
async fn create_test_pool() -> PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://ethhook:password@localhost:5432/ethhook".to_string());

    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

/// Helper: Create test application
async fn create_test_application(pool: &PgPool, user_id: Uuid) -> Uuid {
    let app_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO applications (id, user_id, name, description)
         VALUES ($1, $2, $3, $4)",
    )
    .bind(app_id)
    .bind(user_id)
    .bind("Test Application")
    .bind("Test Description")
    .execute(pool)
    .await
    .expect("Failed to create test application");

    app_id
}

/// Helper: Create test user
async fn create_test_user(pool: &PgPool) -> Uuid {
    let user_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO users (id, email, password_hash)
         VALUES ($1, $2, $3)",
    )
    .bind(user_id)
    .bind(format!("test-{user_id}@example.com"))
    .bind("$argon2id$v=19$m=19456,t=2,p=1$test$test")
    .execute(pool)
    .await
    .expect("Failed to create test user");

    user_id
}

/// Helper: Create test endpoint
async fn create_test_endpoint(
    pool: &PgPool,
    application_id: Uuid,
    contract: Option<&str>,
    topics: Option<Vec<String>>,
    is_active: bool,
) -> Uuid {
    let endpoint_id = Uuid::new_v4();
    let url = format!("https://webhook.example.com/{endpoint_id}");

    sqlx::query(
        "INSERT INTO endpoints 
         (id, application_id, url, hmac_secret, contract_address, event_topics, 
          is_active, rate_limit_per_second, max_retries, timeout_seconds)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
    )
    .bind(endpoint_id)
    .bind(application_id)
    .bind(&url)
    .bind("test_secret_123")
    .bind(contract)
    .bind(topics)
    .bind(is_active)
    .bind(100)
    .bind(3)
    .bind(30)
    .execute(pool)
    .await
    .expect("Failed to create test endpoint");

    endpoint_id
}

/// Helper: Cleanup test data
async fn cleanup_test_data(pool: &PgPool, user_id: Uuid) {
    // Delete in correct order due to foreign keys
    sqlx::query("DELETE FROM endpoints WHERE application_id IN (SELECT id FROM applications WHERE user_id = $1)")
        .bind(user_id)
        .execute(pool)
        .await
        .ok();

    sqlx::query("DELETE FROM applications WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await
        .ok();

    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(pool)
        .await
        .ok();
}

#[tokio::test]
#[ignore] // Requires database
async fn test_match_specific_contract_and_event() {
    let pool = create_test_pool().await;
    let user_id = create_test_user(&pool).await;
    let app_id = create_test_application(&pool, user_id).await;

    // Create endpoint for USDC Transfer events
    let usdc_address = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
    let transfer_topic = "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";

    let endpoint_id = create_test_endpoint(
        &pool,
        app_id,
        Some(usdc_address),
        Some(vec![transfer_topic.to_string()]),
        true,
    )
    .await;

    let matcher = EndpointMatcher::new(pool.clone());

    // Test event: USDC Transfer
    let event = StreamEvent {
        chain_id: 1,
        block_number: 18000000,
        block_hash: "0xblock123".to_string(),
        transaction_hash: "0xtx456".to_string(),
        log_index: 5,
        contract_address: usdc_address.to_string(),
        topics: vec![
            transfer_topic.to_string(),
            "0x000000000000000000000000alice".to_string(),
            "0x000000000000000000000000bob".to_string(),
        ],
        data: "0x0000000000000000000000000000000000000000000000000000000000000064".to_string(), // 100 tokens
        timestamp: 1696800000,
    };

    let matches = matcher
        .find_matching_endpoints(&event)
        .await
        .expect("Failed to find matches");

    // Should match our endpoint
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].endpoint_id, endpoint_id);
    assert_eq!(matches[0].application_id, app_id);
    assert_eq!(matches[0].hmac_secret, "test_secret_123");

    cleanup_test_data(&pool, user_id).await;
}

#[tokio::test]
#[ignore] // Requires database
async fn test_match_all_contracts_wildcard() {
    let pool = create_test_pool().await;
    let user_id = create_test_user(&pool).await;
    let app_id = create_test_application(&pool, user_id).await;

    // Create endpoint with NULL contract (matches ALL contracts)
    let transfer_topic = "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";

    let endpoint_id = create_test_endpoint(
        &pool,
        app_id,
        None, // NULL = match all contracts
        Some(vec![transfer_topic.to_string()]),
        true,
    )
    .await;

    let matcher = EndpointMatcher::new(pool.clone());

    // Test event from ANY contract
    let event = StreamEvent {
        chain_id: 1,
        block_number: 18000000,
        block_hash: "0xblock123".to_string(),
        transaction_hash: "0xtx456".to_string(),
        log_index: 5,
        contract_address: "0x1234567890123456789012345678901234567890".to_string(), // Random contract
        topics: vec![
            transfer_topic.to_string(),
            "0x000000000000000000000000alice".to_string(),
        ],
        data: "0x".to_string(),
        timestamp: 1696800000,
    };

    let matches = matcher
        .find_matching_endpoints(&event)
        .await
        .expect("Failed to find matches");

    // Should match (wildcard contract)
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].endpoint_id, endpoint_id);

    cleanup_test_data(&pool, user_id).await;
}

#[tokio::test]
#[ignore] // Requires database
async fn test_match_all_events_wildcard() {
    let pool = create_test_pool().await;
    let user_id = create_test_user(&pool).await;
    let app_id = create_test_application(&pool, user_id).await;

    // Create endpoint with NULL topics (matches ALL events)
    let usdc_address = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";

    let endpoint_id = create_test_endpoint(
        &pool,
        app_id,
        Some(usdc_address),
        None, // NULL = match all events
        true,
    )
    .await;

    let matcher = EndpointMatcher::new(pool.clone());

    // Test event: ANY event from USDC
    let event = StreamEvent {
        chain_id: 1,
        block_number: 18000000,
        block_hash: "0xblock123".to_string(),
        transaction_hash: "0xtx456".to_string(),
        log_index: 5,
        contract_address: usdc_address.to_string(),
        topics: vec![
            "0x8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925".to_string(), // Approval event
        ],
        data: "0x".to_string(),
        timestamp: 1696800000,
    };

    let matches = matcher
        .find_matching_endpoints(&event)
        .await
        .expect("Failed to find matches");

    // Should match (wildcard topics)
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].endpoint_id, endpoint_id);

    cleanup_test_data(&pool, user_id).await;
}

#[tokio::test]
#[ignore] // Requires database
async fn test_no_match_wrong_contract() {
    let pool = create_test_pool().await;
    let user_id = create_test_user(&pool).await;
    let app_id = create_test_application(&pool, user_id).await;

    // Create endpoint for USDC
    let usdc_address = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
    let transfer_topic = "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";

    create_test_endpoint(
        &pool,
        app_id,
        Some(usdc_address),
        Some(vec![transfer_topic.to_string()]),
        true,
    )
    .await;

    let matcher = EndpointMatcher::new(pool.clone());

    // Test event from DIFFERENT contract (DAI)
    let dai_address = "0x6B175474E89094C44Da98b954EedeAC495271d0F";
    let event = StreamEvent {
        chain_id: 1,
        block_number: 18000000,
        block_hash: "0xblock123".to_string(),
        transaction_hash: "0xtx456".to_string(),
        log_index: 5,
        contract_address: dai_address.to_string(),
        topics: vec![transfer_topic.to_string()],
        data: "0x".to_string(),
        timestamp: 1696800000,
    };

    let matches = matcher
        .find_matching_endpoints(&event)
        .await
        .expect("Failed to find matches");

    // Should NOT match (wrong contract)
    assert_eq!(matches.len(), 0);

    cleanup_test_data(&pool, user_id).await;
}

#[tokio::test]
#[ignore] // Requires database
async fn test_no_match_wrong_event_topic() {
    let pool = create_test_pool().await;
    let user_id = create_test_user(&pool).await;
    let app_id = create_test_application(&pool, user_id).await;

    // Create endpoint for Transfer events
    let usdc_address = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
    let transfer_topic = "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";

    create_test_endpoint(
        &pool,
        app_id,
        Some(usdc_address),
        Some(vec![transfer_topic.to_string()]),
        true,
    )
    .await;

    let matcher = EndpointMatcher::new(pool.clone());

    // Test event: DIFFERENT event (Approval instead of Transfer)
    let approval_topic = "0x8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925";
    let event = StreamEvent {
        chain_id: 1,
        block_number: 18000000,
        block_hash: "0xblock123".to_string(),
        transaction_hash: "0xtx456".to_string(),
        log_index: 5,
        contract_address: usdc_address.to_string(),
        topics: vec![approval_topic.to_string()],
        data: "0x".to_string(),
        timestamp: 1696800000,
    };

    let matches = matcher
        .find_matching_endpoints(&event)
        .await
        .expect("Failed to find matches");

    // Should NOT match (wrong event signature)
    assert_eq!(matches.len(), 0);

    cleanup_test_data(&pool, user_id).await;
}

#[tokio::test]
#[ignore] // Requires database
async fn test_inactive_endpoint_not_matched() {
    let pool = create_test_pool().await;
    let user_id = create_test_user(&pool).await;
    let app_id = create_test_application(&pool, user_id).await;

    // Create INACTIVE endpoint
    let usdc_address = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
    let transfer_topic = "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";

    create_test_endpoint(
        &pool,
        app_id,
        Some(usdc_address),
        Some(vec![transfer_topic.to_string()]),
        false, // INACTIVE
    )
    .await;

    let matcher = EndpointMatcher::new(pool.clone());

    // Test event that would normally match
    let event = StreamEvent {
        chain_id: 1,
        block_number: 18000000,
        block_hash: "0xblock123".to_string(),
        transaction_hash: "0xtx456".to_string(),
        log_index: 5,
        contract_address: usdc_address.to_string(),
        topics: vec![transfer_topic.to_string()],
        data: "0x".to_string(),
        timestamp: 1696800000,
    };

    let matches = matcher
        .find_matching_endpoints(&event)
        .await
        .expect("Failed to find matches");

    // Should NOT match (endpoint is inactive)
    assert_eq!(matches.len(), 0);

    cleanup_test_data(&pool, user_id).await;
}

#[tokio::test]
#[ignore] // Requires database
async fn test_multiple_endpoints_match() {
    let pool = create_test_pool().await;
    let user_id = create_test_user(&pool).await;
    let app_id = create_test_application(&pool, user_id).await;

    let usdc_address = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
    let transfer_topic = "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";

    // Create 3 different endpoints that all match
    let endpoint1 = create_test_endpoint(
        &pool,
        app_id,
        Some(usdc_address),
        Some(vec![transfer_topic.to_string()]),
        true,
    )
    .await;

    let endpoint2 = create_test_endpoint(
        &pool,
        app_id,
        None, // Wildcard contract
        Some(vec![transfer_topic.to_string()]),
        true,
    )
    .await;

    let endpoint3 = create_test_endpoint(
        &pool,
        app_id,
        Some(usdc_address),
        None, // Wildcard events
        true,
    )
    .await;

    let matcher = EndpointMatcher::new(pool.clone());

    // Test event
    let event = StreamEvent {
        chain_id: 1,
        block_number: 18000000,
        block_hash: "0xblock123".to_string(),
        transaction_hash: "0xtx456".to_string(),
        log_index: 5,
        contract_address: usdc_address.to_string(),
        topics: vec![transfer_topic.to_string()],
        data: "0x".to_string(),
        timestamp: 1696800000,
    };

    let matches = matcher
        .find_matching_endpoints(&event)
        .await
        .expect("Failed to find matches");

    // Should match all 3 endpoints
    assert_eq!(matches.len(), 3);

    let matched_ids: Vec<Uuid> = matches.iter().map(|m| m.endpoint_id).collect();
    assert!(matched_ids.contains(&endpoint1));
    assert!(matched_ids.contains(&endpoint2));
    assert!(matched_ids.contains(&endpoint3));

    cleanup_test_data(&pool, user_id).await;
}

#[tokio::test]
#[ignore] // Requires database
async fn test_case_insensitive_contract_matching() {
    let pool = create_test_pool().await;
    let user_id = create_test_user(&pool).await;
    let app_id = create_test_application(&pool, user_id).await;

    // Create endpoint with UPPERCASE contract address
    let usdc_address_upper = "0xA0B86991C6218B36C1D19D4A2E9EB0CE3606EB48";
    let transfer_topic = "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";

    let endpoint_id = create_test_endpoint(
        &pool,
        app_id,
        Some(usdc_address_upper),
        Some(vec![transfer_topic.to_string()]),
        true,
    )
    .await;

    let matcher = EndpointMatcher::new(pool.clone());

    // Test event with LOWERCASE contract address
    let usdc_address_lower = "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48";
    let event = StreamEvent {
        chain_id: 1,
        block_number: 18000000,
        block_hash: "0xblock123".to_string(),
        transaction_hash: "0xtx456".to_string(),
        log_index: 5,
        contract_address: usdc_address_lower.to_string(),
        topics: vec![transfer_topic.to_string()],
        data: "0x".to_string(),
        timestamp: 1696800000,
    };

    let matches = matcher
        .find_matching_endpoints(&event)
        .await
        .expect("Failed to find matches");

    // Should match (case-insensitive)
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].endpoint_id, endpoint_id);

    cleanup_test_data(&pool, user_id).await;
}

#[tokio::test]
#[ignore] // Requires database
async fn test_topic_subset_matching() {
    let pool = create_test_pool().await;
    let user_id = create_test_user(&pool).await;
    let app_id = create_test_application(&pool, user_id).await;

    // Create endpoint that listens for Transfer events (just the signature)
    let usdc_address = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
    let transfer_topic = "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";

    let endpoint_id = create_test_endpoint(
        &pool,
        app_id,
        Some(usdc_address),
        Some(vec![transfer_topic.to_string()]), // Just the event signature
        true,
    )
    .await;

    let matcher = EndpointMatcher::new(pool.clone());

    // Test event with MULTIPLE topics (signature + from + to)
    let event = StreamEvent {
        chain_id: 1,
        block_number: 18000000,
        block_hash: "0xblock123".to_string(),
        transaction_hash: "0xtx456".to_string(),
        log_index: 5,
        contract_address: usdc_address.to_string(),
        topics: vec![
            transfer_topic.to_string(),
            "0x000000000000000000000000alice".to_string(), // from
            "0x000000000000000000000000bob".to_string(),   // to
        ],
        data: "0x0000000000000000000000000000000000000000000000000000000000000064".to_string(),
        timestamp: 1696800000,
    };

    let matches = matcher
        .find_matching_endpoints(&event)
        .await
        .expect("Failed to find matches");

    // Should match (endpoint topics are subset of event topics)
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].endpoint_id, endpoint_id);

    cleanup_test_data(&pool, user_id).await;
}

#[tokio::test]
#[ignore] // Requires database
async fn test_multiple_topic_matching() {
    let pool = create_test_pool().await;
    let user_id = create_test_user(&pool).await;
    let app_id = create_test_application(&pool, user_id).await;

    // Create endpoint that listens for Transfer events to a specific address
    let usdc_address = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
    let transfer_topic = "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";
    let bob_topic = "0x000000000000000000000000bob";

    let endpoint_id = create_test_endpoint(
        &pool,
        app_id,
        Some(usdc_address),
        Some(vec![transfer_topic.to_string(), bob_topic.to_string()]), // Transfer to Bob
        true,
    )
    .await;

    let matcher = EndpointMatcher::new(pool.clone());

    // Test event: Transfer to Bob (should match)
    let event = StreamEvent {
        chain_id: 1,
        block_number: 18000000,
        block_hash: "0xblock123".to_string(),
        transaction_hash: "0xtx456".to_string(),
        log_index: 5,
        contract_address: usdc_address.to_string(),
        topics: vec![
            transfer_topic.to_string(),
            "0x000000000000000000000000alice".to_string(), // from
            bob_topic.to_string(),                         // to Bob
        ],
        data: "0x".to_string(),
        timestamp: 1696800000,
    };

    let matches = matcher
        .find_matching_endpoints(&event)
        .await
        .expect("Failed to find matches");

    // Should match (all required topics present)
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].endpoint_id, endpoint_id);

    cleanup_test_data(&pool, user_id).await;
}

#[tokio::test]
#[ignore] // Requires database
async fn test_empty_topics_array() {
    let pool = create_test_pool().await;
    let user_id = create_test_user(&pool).await;
    let app_id = create_test_application(&pool, user_id).await;

    // Create endpoint with empty topics array (not NULL, just empty)
    let usdc_address = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";

    let endpoint_id = create_test_endpoint(
        &pool,
        app_id,
        Some(usdc_address),
        Some(vec![]), // Empty array
        true,
    )
    .await;

    let matcher = EndpointMatcher::new(pool.clone());

    // Test event from the contract
    let event = StreamEvent {
        chain_id: 1,
        block_number: 18000000,
        block_hash: "0xblock123".to_string(),
        transaction_hash: "0xtx456".to_string(),
        log_index: 5,
        contract_address: usdc_address.to_string(),
        topics: vec![
            "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef".to_string(),
        ],
        data: "0x".to_string(),
        timestamp: 1696800000,
    };

    let matches = matcher
        .find_matching_endpoints(&event)
        .await
        .expect("Failed to find matches");

    // Should match (empty array means match all)
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].endpoint_id, endpoint_id);

    cleanup_test_data(&pool, user_id).await;
}

#[tokio::test]
#[ignore] // Requires database
async fn test_match_event_without_topics() {
    let pool = create_test_pool().await;
    let user_id = create_test_user(&pool).await;
    let app_id = create_test_application(&pool, user_id).await;

    // Create wildcard endpoint
    let usdc_address = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";

    let endpoint_id = create_test_endpoint(
        &pool,
        app_id,
        Some(usdc_address),
        None, // Match all events
        true,
    )
    .await;

    let matcher = EndpointMatcher::new(pool.clone());

    // Test event with NO topics (anonymous event or unusual structure)
    let event = StreamEvent {
        chain_id: 1,
        block_number: 18000000,
        block_hash: "0xblock123".to_string(),
        transaction_hash: "0xtx456".to_string(),
        log_index: 5,
        contract_address: usdc_address.to_string(),
        topics: vec![], // No topics
        data: "0x1234".to_string(),
        timestamp: 1696800000,
    };

    let matches = matcher
        .find_matching_endpoints(&event)
        .await
        .expect("Failed to find matches");

    // Should match (wildcard endpoint matches all events)
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].endpoint_id, endpoint_id);

    cleanup_test_data(&pool, user_id).await;
}

#[tokio::test]
#[ignore] // Requires database
async fn test_mixed_case_checksum_addresses() {
    let pool = create_test_pool().await;
    let user_id = create_test_user(&pool).await;
    let app_id = create_test_application(&pool, user_id).await;

    // Create endpoint with EIP-55 checksummed address
    let usdc_checksum = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"; // Mixed case
    let transfer_topic = "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";

    let endpoint_id = create_test_endpoint(
        &pool,
        app_id,
        Some(usdc_checksum),
        Some(vec![transfer_topic.to_string()]),
        true,
    )
    .await;

    let matcher = EndpointMatcher::new(pool.clone());

    // Test event with different case
    let usdc_lower = "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48";
    let event = StreamEvent {
        chain_id: 1,
        block_number: 18000000,
        block_hash: "0xblock123".to_string(),
        transaction_hash: "0xtx456".to_string(),
        log_index: 5,
        contract_address: usdc_lower.to_string(),
        topics: vec![transfer_topic.to_string()],
        data: "0x".to_string(),
        timestamp: 1696800000,
    };

    let matches = matcher
        .find_matching_endpoints(&event)
        .await
        .expect("Failed to find matches");

    // Should match (case-insensitive matching)
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].endpoint_id, endpoint_id);

    cleanup_test_data(&pool, user_id).await;
}

#[tokio::test]
#[ignore] // Requires database
async fn test_different_applications_isolated() {
    let pool = create_test_pool().await;
    let user1_id = create_test_user(&pool).await;
    let user2_id = create_test_user(&pool).await;
    let app1_id = create_test_application(&pool, user1_id).await;
    let app2_id = create_test_application(&pool, user2_id).await;

    // Both apps have endpoints for the same contract/event
    let usdc_address = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
    let transfer_topic = "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";

    let endpoint1 = create_test_endpoint(
        &pool,
        app1_id,
        Some(usdc_address),
        Some(vec![transfer_topic.to_string()]),
        true,
    )
    .await;

    let endpoint2 = create_test_endpoint(
        &pool,
        app2_id,
        Some(usdc_address),
        Some(vec![transfer_topic.to_string()]),
        true,
    )
    .await;

    let matcher = EndpointMatcher::new(pool.clone());

    // Test event
    let event = StreamEvent {
        chain_id: 1,
        block_number: 18000000,
        block_hash: "0xblock123".to_string(),
        transaction_hash: "0xtx456".to_string(),
        log_index: 5,
        contract_address: usdc_address.to_string(),
        topics: vec![transfer_topic.to_string()],
        data: "0x".to_string(),
        timestamp: 1696800000,
    };

    let matches = matcher
        .find_matching_endpoints(&event)
        .await
        .expect("Failed to find matches");

    // Should match BOTH endpoints (different applications can have same rules)
    assert_eq!(matches.len(), 2);

    let matched_ids: Vec<Uuid> = matches.iter().map(|m| m.endpoint_id).collect();
    assert!(matched_ids.contains(&endpoint1));
    assert!(matched_ids.contains(&endpoint2));

    // Verify they belong to different applications
    let app_ids: Vec<Uuid> = matches.iter().map(|m| m.application_id).collect();
    assert!(app_ids.contains(&app1_id));
    assert!(app_ids.contains(&app2_id));

    cleanup_test_data(&pool, user1_id).await;
    cleanup_test_data(&pool, user2_id).await;
}

#[tokio::test]
#[ignore] // Requires database
async fn test_wildcard_contract_and_events() {
    let pool = create_test_pool().await;
    let user_id = create_test_user(&pool).await;
    let app_id = create_test_application(&pool, user_id).await;

    // Create endpoint with BOTH wildcards (matches everything)
    let endpoint_id = create_test_endpoint(
        &pool, app_id, None, // NULL contract
        None, // NULL topics
        true,
    )
    .await;

    let matcher = EndpointMatcher::new(pool.clone());

    // Test ANY event from ANY contract
    let event = StreamEvent {
        chain_id: 1,
        block_number: 18000000,
        block_hash: "0xblock123".to_string(),
        transaction_hash: "0xtx456".to_string(),
        log_index: 5,
        contract_address: "0x1234567890123456789012345678901234567890".to_string(),
        topics: vec![
            "0xabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcd".to_string(),
        ],
        data: "0x".to_string(),
        timestamp: 1696800000,
    };

    let matches = matcher
        .find_matching_endpoints(&event)
        .await
        .expect("Failed to find matches");

    // Should match (double wildcard)
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].endpoint_id, endpoint_id);

    cleanup_test_data(&pool, user_id).await;
}

#[tokio::test]
#[ignore] // Requires database
async fn test_matched_endpoint_fields() {
    let pool = create_test_pool().await;
    let user_id = create_test_user(&pool).await;
    let app_id = create_test_application(&pool, user_id).await;

    let usdc_address = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
    let transfer_topic = "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";

    let endpoint_id = create_test_endpoint(
        &pool,
        app_id,
        Some(usdc_address),
        Some(vec![transfer_topic.to_string()]),
        true,
    )
    .await;

    let matcher = EndpointMatcher::new(pool.clone());

    let event = StreamEvent {
        chain_id: 1,
        block_number: 18000000,
        block_hash: "0xblock123".to_string(),
        transaction_hash: "0xtx456".to_string(),
        log_index: 5,
        contract_address: usdc_address.to_string(),
        topics: vec![transfer_topic.to_string()],
        data: "0x".to_string(),
        timestamp: 1696800000,
    };

    let matches = matcher
        .find_matching_endpoints(&event)
        .await
        .expect("Failed to find matches");

    assert_eq!(matches.len(), 1);
    let matched = &matches[0];

    // Verify all fields are populated correctly
    assert_eq!(matched.endpoint_id, endpoint_id);
    assert_eq!(matched.application_id, app_id);
    assert!(matched.url.contains(&endpoint_id.to_string()));
    assert_eq!(matched.hmac_secret, "test_secret_123");
    assert_eq!(matched.rate_limit_per_second, 100);
    assert_eq!(matched.max_retries, 3);
    assert_eq!(matched.timeout_seconds, 30);

    cleanup_test_data(&pool, user_id).await;
}

#[tokio::test]
async fn test_matcher_creation() {
    // Simple unit test that doesn't require database
    let database_url = "postgresql://test:test@localhost:5432/test";

    // This should not panic
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(std::time::Duration::from_secs(1))
        .connect_lazy(database_url)
        .expect("Failed to create pool");

    let _matcher = EndpointMatcher::new(pool);
    // If we get here, matcher was created successfully
}

#[tokio::test]
#[ignore] // Requires database
async fn test_no_matches_empty_result() {
    let pool = create_test_pool().await;
    let user_id = create_test_user(&pool).await;
    let app_id = create_test_application(&pool, user_id).await;

    // Create endpoint for a DIFFERENT contract
    let dai_address = "0x6B175474E89094C44Da98b954EedeAC495271d0F";
    create_test_endpoint(&pool, app_id, Some(dai_address), None, true).await;

    let matcher = EndpointMatcher::new(pool.clone());

    // Test event from USDC (no match expected)
    let usdc_address = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
    let event = StreamEvent {
        chain_id: 1,
        block_number: 18000000,
        block_hash: "0xblock123".to_string(),
        transaction_hash: "0xtx456".to_string(),
        log_index: 5,
        contract_address: usdc_address.to_string(),
        topics: vec![
            "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef".to_string(),
        ],
        data: "0x".to_string(),
        timestamp: 1696800000,
    };

    let matches = matcher
        .find_matching_endpoints(&event)
        .await
        .expect("Failed to find matches");

    // Should return empty vec (not error)
    assert_eq!(matches.len(), 0);
    assert!(matches.is_empty());

    cleanup_test_data(&pool, user_id).await;
}
