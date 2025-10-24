/*!
 * Message Processor Service
 *
 * Reads events from Redis Streams, queries database for matching endpoints,
 * and creates delivery jobs.
 *
 * ## Architecture
 *
 * ```text
 * Redis Streams          Message Processor          PostgreSQL        Redis Queue
 * ─────────────         ──────────────────         ──────────         ───────────
 *                              │
 * events:1 ──────────────────> │
 * events:42161 ──────────────> │
 * events:10 ─────────────────> │
 * events:8453 ───────────────> │
 *                              │
 *                              ├─────── Query endpoints ──────>│
 *                              │          WHERE chain_id = 1   │
 *                              │          AND contract = 0x... │
 *                              │<────── Return endpoints ──────┤
 *                              │
 *                              │                                       |
 *                              ├─── LPUSH delivery_queue ─────────────>│
 *                              │    { endpoint_id, event_data }        │
 * ```
 *
 * ## Why Consumer Groups?
 *
 * Redis Streams support **consumer groups** for horizontal scaling:
 *
 * ```text
 * Stream: events:1
 *    │
 *    ├──> Consumer Group "processors"
 *         ├──> Worker 1 (processes IDs 1-100)
 *         ├──> Worker 2 (processes IDs 101-200)
 *         └──> Worker 3 (processes IDs 201-300)
 * ```
 *
 * Benefits:
 * - **Load Balancing**: Automatic work distribution
 * - **At-Least-Once Delivery**: ACK mechanism
 * - **Fault Tolerance**: Pending entry list (PEL) for crashes
 *
 * ## Performance Targets
 *
 * - **Throughput**: 10,000 events/second
 * - **Latency**: < 100ms from stream to queue
 * - **Database**: Connection pool (20 connections)
 * - **Batch Processing**: Process up to 100 events per XREAD
 */

pub mod config;
pub mod consumer;
pub mod matcher;
pub mod metrics;
pub mod publisher;
