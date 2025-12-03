/**
 * ClickHouse Client API
 * 
 * Production-grade ClickHouse HTTP interface with:
 * - Batch insert operations (100x faster than individual inserts)
 * - Connection pooling and reuse
 * - Automatic retry with exponential backoff
 * - Compression support (gzip)
 * - Thread-safe operations
 */

#ifndef ETHHOOK_CLICKHOUSE_H
#define ETHHOOK_CLICKHOUSE_H

#include "ethhook/common.h"
#include <curl/curl.h>
#include <pthread.h>
#include <stdatomic.h>
#include <stdbool.h>
#include <stddef.h>
#include <time.h>

#ifdef __cplusplus
extern "C" {
#endif

// Forward declarations
typedef struct clickhouse_client clickhouse_client_t;
typedef struct clickhouse_result clickhouse_result_t;
typedef struct clickhouse_batch clickhouse_batch_t;

/**
 * ClickHouse client configuration
 */
typedef struct {
    char *url;              // HTTP URL (e.g., "http://localhost:8123")
    char *database;         // Database name
    char *user;             // Username
    char *password;         // Password
    uint32_t pool_size;     // Connection pool size (default: 10)
    uint32_t timeout_ms;    // Query timeout (default: 30000)
    bool enable_compression; // Enable gzip compression (default: true)
    uint32_t batch_size;    // Auto-flush batch size (default: 1000)
    uint32_t batch_timeout_ms; // Auto-flush timeout (default: 1000)
} clickhouse_config_t;

/**
 * ClickHouse query result
 */
struct clickhouse_result {
    char *data;           // Response data (JSON or TSV)
    size_t data_len;      // Data length
    uint64_t rows_read;   // Number of rows read
    uint64_t bytes_read;  // Number of bytes read
    double elapsed_sec;   // Query elapsed time
};

/**
 * Event structure for batch inserts
 */
typedef struct {
    char id[37];              // UUID string
    char endpoint_id[37];     // UUID string
    char application_id[37];  // UUID string
    uint64_t chain_id;
    uint64_t block_number;
    char block_hash[67];      // 0x + 64 hex chars
    char transaction_hash[67];
    uint32_t log_index;
    char contract_address[43]; // 0x + 40 hex chars
    char **topics;            // Array of topic strings
    size_t topics_count;
    char *data;               // Hex string
    uint64_t ingested_at_ms;  // Unix timestamp in milliseconds
    uint64_t processed_at_ms; // Unix timestamp in milliseconds
} clickhouse_event_t;

// Compile-time safety checks
_Static_assert(sizeof(clickhouse_event_t) > 0, "clickhouse_event_t must have non-zero size");
_Static_assert(sizeof(((clickhouse_event_t*)0)->id) == 37, "UUID must be 36 chars + null");
_Static_assert(sizeof(((clickhouse_event_t*)0)->block_hash) == 67, "Block hash must be 66 chars + null");
_Static_assert(sizeof(((clickhouse_event_t*)0)->contract_address) == 43, "Address must be 42 chars + null");

/**
 * Delivery structure for batch inserts
 */
typedef struct {
    char id[37];              // UUID string
    char event_id[37];        // UUID string
    char endpoint_id[37];     // UUID string
    char url[512];            // Webhook URL
    char status[32];          // Status (pending, delivered, failed)
    uint32_t attempt_count;
    int32_t http_status_code;
    char *error_message;      // NULL if no error
    uint64_t delivered_at_ms; // Unix timestamp in milliseconds
    uint64_t next_retry_at_ms; // Unix timestamp in milliseconds
} clickhouse_delivery_t;

// Compile-time safety checks
_Static_assert(sizeof(clickhouse_delivery_t) > 0, "clickhouse_delivery_t must have non-zero size");
_Static_assert(sizeof(((clickhouse_delivery_t*)0)->url) == 512, "URL buffer must be 512 bytes");
_Static_assert(sizeof(((clickhouse_delivery_t*)0)->status) == 32, "Status buffer must be 32 bytes");

/**
 * Initialize ClickHouse client
 * Creates connection pool and initializes libcurl.
 * Thread-safe.
 * 
 * @param config Client configuration
 * @param client Output client pointer
 * @return ETH_OK on success
 */
eth_error_t clickhouse_client_create(
    const clickhouse_config_t *config,
    clickhouse_client_t **client
);

/**
 * Destroy ClickHouse client
 * 
 * Closes all connections and frees resources.
 * Thread-safe.
 * 
 * @param client Client to destroy
 */
void clickhouse_client_destroy(clickhouse_client_t *client);

/**
 * Execute raw SQL query
 * 
 * Executes arbitrary SQL query and returns result.
 * Thread-safe.
 * 
 * @param client ClickHouse client
 * @param query SQL query string
 * @param result Output result pointer (can be NULL for INSERT/CREATE)
 * @return ETH_OK on success
 */
eth_error_t clickhouse_query(
    clickhouse_client_t *client,
    const char *query,
    clickhouse_result_t **result
);

/**
 * Execute query with parameters (SQL injection safe)
 * 
 * Uses parameter substitution to prevent SQL injection.
 * Thread-safe.
 * 
 * @param client ClickHouse client
 * @param query Query template with {param} placeholders
 * @param params Array of parameter values
 * @param param_count Number of parameters
 * @param result Output result pointer
 * @return ETH_OK on success
 */
eth_error_t clickhouse_query_params(
    clickhouse_client_t *client,
    const char *query,
    const char **params,
    size_t param_count,
    clickhouse_result_t **result
);

/**
 * Free query result
 * 
 * @param result Result to free
 */
void clickhouse_result_free(clickhouse_result_t *result);

// =============================================================================
// BATCH OPERATIONS (100x faster than individual inserts)
// =============================================================================

/**
 * Create batch insert buffer
 * 
 * Creates auto-flushing batch buffer for high-throughput inserts.
 * Thread-safe.
 * 
 * @param client ClickHouse client
 * @param table_name Target table name
 * @param capacity Maximum batch size before auto-flush
 * @param batch Output batch pointer
 * @return ETH_OK on success
 */
eth_error_t clickhouse_batch_create(
    clickhouse_client_t *client,
    const char *table_name,
    size_t capacity,
    clickhouse_batch_t **batch
);

/**
 * Add event to batch
 * 
 * Auto-flushes if batch is full or timeout elapsed.
 * Thread-safe.
 * 
 * @param batch Batch buffer
 * @param event Event to add
 * @return ETH_OK on success
 */
eth_error_t clickhouse_batch_add_event(
    clickhouse_batch_t *batch,
    const clickhouse_event_t *event
);

/**
 * Add delivery to batch
 * 
 * Auto-flushes if batch is full or timeout elapsed.
 * Thread-safe.
 * 
 * @param batch Batch buffer
 * @param delivery Delivery to add
 * @return ETH_OK on success
 */
eth_error_t clickhouse_batch_add_delivery(
    clickhouse_batch_t *batch,
    const clickhouse_delivery_t *delivery
);

/**
 * Manually flush batch
 * 
 * Sends all buffered rows to ClickHouse.
 * Thread-safe.
 * 
 * @param batch Batch buffer
 * @return ETH_OK on success
 */
eth_error_t clickhouse_batch_flush(clickhouse_batch_t *batch);

/**
 * Destroy batch buffer
 * 
 * Flushes any pending rows before destroying.
 * Thread-safe.
 * 
 * @param batch Batch to destroy
 */
void clickhouse_batch_destroy(clickhouse_batch_t *batch);

// =============================================================================
// SCHEMA INITIALIZATION
// =============================================================================

/**
 * Initialize ClickHouse schema
 * 
 * Creates tables with optimized settings:
 * - MergeTree engine for fast queries
 * - Monthly partitions by ingested_at
 * - 90-day TTL for automatic cleanup
 * - Optimized ORDER BY for query patterns
 * - LZ4 compression (10x reduction)
 * 
 * @param client ClickHouse client
 * @return ETH_OK on success
 */
eth_error_t clickhouse_init_schema(clickhouse_client_t *client);

/**
 * Get ClickHouse metrics
 * 
 * Returns client performance metrics.
 * 
 * @param client ClickHouse client
 * @param queries_executed Output total queries
 * @param batches_flushed Output total batches
 * @param rows_inserted Output total rows
 * @param total_latency_ms Output cumulative latency
 */
void clickhouse_get_metrics(
    const clickhouse_client_t *client,
    uint64_t *queries_executed,
    uint64_t *batches_flushed,
    uint64_t *rows_inserted,
    uint64_t *total_latency_ms
);

#ifdef __cplusplus
}
#endif

#endif // ETHHOOK_CLICKHOUSE_H
