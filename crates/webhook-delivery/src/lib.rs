/*!
 * Webhook Delivery Service
 *
 * Consumes delivery jobs from Redis Queue and sends webhooks to customer endpoints.
 *
 * ## Architecture
 *
 * ```text
 * Redis Queue          Webhook Delivery         Customer Endpoint
 * ───────────         ─────────────────         ─────────────────
 *                           │
 * delivery_queue ─────────> │
 *   (BRPOP)                 │
 *                           ├─── POST /webhook ───────────────>│
 *                           │    X-Webhook-Signature: hmac     │
 *                           │    Content-Type: application/json│
 *                           │    {                             │
 *                           │      "chain_id": 1,              │
 *                           │      "block_number": 18000000,   │
 *                           │      "transaction_hash": "0x...",│
 *                           │      "contract": "0x...",        │
 *                           │      "topics": ["0x..."],        │
 *                           │      "data": "0x..."             │
 *                           │    }                             │
 *                           │                                  │
 *                           │<─── 200 OK ──────────────────────┤
 *                           │     or 4xx/5xx error             │
 * ```
 *
 * ## Retry Strategy
 *
 * Exponential backoff with jitter:
 * - Attempt 1: Immediate
 * - Attempt 2: 2 seconds
 * - Attempt 3: 4 seconds
 * - Attempt 4: 8 seconds
 * - Attempt 5: 16 seconds
 *
 * Total max wait: ~30 seconds over 5 attempts
 *
 * ## Circuit Breaker
 *
 * Per-endpoint circuit breaker:
 * - **Closed**: Normal operation
 * - **Open**: 5+ consecutive failures, wait 60s
 * - **Half-Open**: Test with single request
 *
 * ## Performance Targets
 *
 * - **Throughput**: 1,000 webhooks/second
 * - **Latency**: < 500ms per webhook (p95)
 * - **Worker Pool**: 50 concurrent workers
 * - **Timeout**: 30 seconds per request
 */

pub mod circuit_breaker;
pub mod config;
pub mod consumer;
pub mod delivery;
pub mod metrics;
pub mod retry;
