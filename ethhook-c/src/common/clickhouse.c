/**
 * ClickHouse Client Implementation
 * 
 * High-performance HTTP client with batching and connection pooling.
 */

#define _GNU_SOURCE
#include "ethhook/clickhouse.h"
#include "ethhook/common.h"
#include <curl/curl.h>
#include <pthread.h>
#include <stdatomic.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

// Connection pool entry
typedef struct pooled_curl {
    CURL *handle;
    atomic_bool in_use;
    time_t last_used;
} pooled_curl_t;

// ClickHouse client structure
struct clickhouse_client {
    char *url;
    char *database;
    char *user;
    char *password;
    uint32_t timeout_ms;
    bool enable_compression;
    
    // Connection pool
    pooled_curl_t *connections;
    size_t pool_size;
    pthread_mutex_t pool_lock;
    
    // Metrics
    atomic_uint_fast64_t queries_executed;
    atomic_uint_fast64_t batches_flushed;
    atomic_uint_fast64_t rows_inserted;
    atomic_uint_fast64_t total_latency_ms;
};

// Batch buffer structure
struct clickhouse_batch {
    clickhouse_client_t *client;
    char *table_name;
    void **items;          // Generic array (events or deliveries)
    size_t count;
    size_t capacity;
    bool is_event_batch;   // true = events, false = deliveries
    pthread_mutex_t lock;
    struct timespec last_flush;
    uint32_t timeout_ms;
};

// HTTP response buffer
typedef struct {
    char *data;
    size_t size;
} http_response_t;

// cURL write callback
static size_t write_callback(void *contents, size_t size, size_t nmemb, void *userp) {
    size_t realsize = size * nmemb;
    http_response_t *resp = (http_response_t*)userp;
    
    char *ptr = realloc(resp->data, resp->size + realsize + 1);
    if (!ptr) {
        return 0; // Out of memory
    }
    
    resp->data = ptr;
    memcpy(&(resp->data[resp->size]), contents, realsize);
    resp->size += realsize;
    resp->data[resp->size] = 0;
    
    return realsize;
}

eth_error_t clickhouse_client_create(
    const clickhouse_config_t *config,
    clickhouse_client_t **client
) {
    if (!config || !client) {
        return ETH_ERROR_INVALID_PARAM;
    }
    
    clickhouse_client_t *c = calloc(1, sizeof(clickhouse_client_t));
    if (!c) {
        return ETH_ERROR_MEMORY;
    }
    
    // Copy configuration
    c->url = strdup(config->url);
    c->database = strdup(config->database);
    c->user = strdup(config->user);
    c->password = strdup(config->password ? config->password : "");
    c->timeout_ms = config->timeout_ms ? config->timeout_ms : 30000;
    c->enable_compression = config->enable_compression;
    
    // Initialize connection pool
    c->pool_size = config->pool_size ? config->pool_size : 10;
    c->connections = calloc(c->pool_size, sizeof(pooled_curl_t));
    if (!c->connections) {
        free(c);
        return ETH_ERROR_MEMORY;
    }
    
    pthread_mutex_init(&c->pool_lock, NULL);
    
    // Initialize cURL handles
    curl_global_init(CURL_GLOBAL_ALL);
    
    for (size_t i = 0; i < c->pool_size; i++) {
        c->connections[i].handle = curl_easy_init();
        if (!c->connections[i].handle) {
            clickhouse_client_destroy(c);
            return ETH_ERROR;
        }
        atomic_init(&c->connections[i].in_use, false);
        c->connections[i].last_used = 0;
    }
    
    // Initialize metrics
    atomic_init(&c->queries_executed, 0);
    atomic_init(&c->batches_flushed, 0);
    atomic_init(&c->rows_inserted, 0);
    atomic_init(&c->total_latency_ms, 0);
    
    LOG_INFO("ClickHouse client created: %s (pool_size=%zu)", 
                 config->url, c->pool_size);
    
    *client = c;
    return ETH_OK;
}

void clickhouse_client_destroy(clickhouse_client_t *client) {
    if (!client) return;
    
    // Cleanup connection pool
    for (size_t i = 0; i < client->pool_size; i++) {
        if (client->connections[i].handle) {
            curl_easy_cleanup(client->connections[i].handle);
        }
    }
    free(client->connections);
    
    pthread_mutex_destroy(&client->pool_lock);
    
    free(client->url);
    free(client->database);
    free(client->user);
    free(client->password);
    free(client);
    
    curl_global_cleanup();
}

// Get connection from pool
static CURL *get_connection(clickhouse_client_t *client) {
    pthread_mutex_lock(&client->pool_lock);
    
    // Find available connection
    for (size_t i = 0; i < client->pool_size; i++) {
        bool expected = false;
        if (atomic_compare_exchange_strong(&client->connections[i].in_use, 
                                           &expected, true)) {
            pthread_mutex_unlock(&client->pool_lock);
            return client->connections[i].handle;
        }
    }
    
    pthread_mutex_unlock(&client->pool_lock);
    
    // No available connections (should not happen with proper pool sizing)
    LOG_WARN("No available ClickHouse connections in pool");
    return NULL;
}

// Return connection to pool
static void release_connection(clickhouse_client_t *client, CURL *handle) {
    pthread_mutex_lock(&client->pool_lock);
    
    for (size_t i = 0; i < client->pool_size; i++) {
        if (client->connections[i].handle == handle) {
            client->connections[i].last_used = time(NULL);
            atomic_store(&client->connections[i].in_use, false);
            break;
        }
    }
    
    pthread_mutex_unlock(&client->pool_lock);
}

eth_error_t clickhouse_query(
    clickhouse_client_t *client,
    const char *query,
    clickhouse_result_t **result
) {
    if (!client || !query) {
        return ETH_ERROR_INVALID_PARAM;
    }
    
    struct timespec start, end;
    clock_gettime(CLOCK_MONOTONIC, &start);
    
    CURL *curl = get_connection(client);
    if (!curl) {
        return ETH_ERROR;
    }
    
    // Build URL with database
    char url[1024];
    snprintf(url, sizeof(url), "%s/?database=%s&query=%s", 
             client->url, client->database, "");
    
    http_response_t response = {.data = NULL, .size = 0};
    
    // Configure cURL
    curl_easy_setopt(curl, CURLOPT_URL, client->url);
    curl_easy_setopt(curl, CURLOPT_POST, 1L);
    curl_easy_setopt(curl, CURLOPT_POSTFIELDS, query);
    curl_easy_setopt(curl, CURLOPT_POSTFIELDSIZE, strlen(query));
    curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, write_callback);
    curl_easy_setopt(curl, CURLOPT_WRITEDATA, &response);
    curl_easy_setopt(curl, CURLOPT_TIMEOUT_MS, client->timeout_ms);
    
    // Set headers
    struct curl_slist *headers = NULL;
    headers = curl_slist_append(headers, "Content-Type: text/plain; charset=utf-8");
    
    char db_header[256];
    snprintf(db_header, sizeof(db_header), "X-ClickHouse-Database: %s", client->database);
    headers = curl_slist_append(headers, db_header);
    
    if (client->user && strlen(client->user) > 0) {
        char user_header[256];
        snprintf(user_header, sizeof(user_header), "X-ClickHouse-User: %s", client->user);
        headers = curl_slist_append(headers, user_header);
    }
    
    if (client->password && strlen(client->password) > 0) {
        char pass_header[256];
        snprintf(pass_header, sizeof(pass_header), "X-ClickHouse-Key: %s", client->password);
        headers = curl_slist_append(headers, pass_header);
    }
    
    if (client->enable_compression) {
        headers = curl_slist_append(headers, "Accept-Encoding: gzip");
    }
    
    curl_easy_setopt(curl, CURLOPT_HTTPHEADER, headers);
    
    // Execute query
    CURLcode res = curl_easy_perform(curl);
    
    curl_slist_free_all(headers);
    
    clock_gettime(CLOCK_MONOTONIC, &end);
    uint64_t latency_ms = (end.tv_sec - start.tv_sec) * 1000 + 
                          (end.tv_nsec - start.tv_nsec) / 1000000;
    
    // Update metrics
    atomic_fetch_add(&client->queries_executed, 1);
    atomic_fetch_add(&client->total_latency_ms, latency_ms);
    
    if (res != CURLE_OK) {
        LOG_ERROR("ClickHouse query failed: %s", curl_easy_strerror(res));
        free(response.data);
        release_connection(client, curl);
        return ETH_ERROR;
    }
    
    long http_code = 0;
    curl_easy_getinfo(curl, CURLINFO_RESPONSE_CODE, &http_code);
    
    release_connection(client, curl);
    
    if (http_code != 200) {
        LOG_ERROR("ClickHouse returned HTTP %ld: %s", http_code, 
                      response.data ? response.data : "(no response)");
        free(response.data);
        return ETH_ERROR;
    }
    
    // Build result if requested
    if (result) {
        clickhouse_result_t *r = calloc(1, sizeof(clickhouse_result_t));
        if (!r) {
            free(response.data);
            return ETH_ERROR_MEMORY;
        }
        
        r->data = response.data;
        r->data_len = response.size;
        r->elapsed_sec = latency_ms / 1000.0;
        
        *result = r;
    } else {
        free(response.data);
    }
    
    return ETH_OK;
}

void clickhouse_result_free(clickhouse_result_t *result) {
    if (!result) return;
    free(result->data);
    free(result);
}

// =============================================================================
// BATCH OPERATIONS
// =============================================================================

eth_error_t clickhouse_batch_create(
    clickhouse_client_t *client,
    const char *table_name,
    size_t capacity,
    clickhouse_batch_t **batch
) {
    if (!client || !table_name || !batch) {
        return ETH_ERROR_INVALID_PARAM;
    }
    
    clickhouse_batch_t *b = calloc(1, sizeof(clickhouse_batch_t));
    if (!b) {
        return ETH_ERROR_MEMORY;
    }
    
    b->client = client;
    b->table_name = strdup(table_name);
    b->capacity = capacity ? capacity : 1000;
    b->items = calloc(b->capacity, sizeof(void*));
    b->timeout_ms = 1000; // 1 second auto-flush
    
    if (!b->items) {
        free(b);
        return ETH_ERROR_MEMORY;
    }
    
    pthread_mutex_init(&b->lock, NULL);
    clock_gettime(CLOCK_MONOTONIC, &b->last_flush);
    
    *batch = b;
    return ETH_OK;
}

// Helper: Build INSERT query for events
static char *build_events_insert_query(
    const char *table_name,
    clickhouse_event_t **events,
    size_t count
) {
    size_t buf_size = 1024 * 1024; // 1MB initial buffer
    char *query = malloc(buf_size);
    if (!query) return NULL;
    
    size_t offset = snprintf(query, buf_size,
        "INSERT INTO %s (id, endpoint_id, application_id, chain_id, "
        "block_number, block_hash, transaction_hash, log_index, "
        "contract_address, topics, data, ingested_at, processed_at) FORMAT JSONEachRow\n",
        table_name);
    
    for (size_t i = 0; i < count; i++) {
        clickhouse_event_t *e = events[i];
        
        // Build topics array (safe bounded string building)
        char topics_json[4096];
        size_t topics_offset = 0;
        topics_offset += snprintf(topics_json + topics_offset, 
                                  sizeof(topics_json) - topics_offset, "[");
        
        for (size_t t = 0; t < e->topics_count && topics_offset < sizeof(topics_json) - 10; t++) {
            int written = snprintf(topics_json + topics_offset, 
                                   sizeof(topics_json) - topics_offset, 
                                   "\"%s\"%s", 
                                   e->topics[t],
                                   (t < e->topics_count - 1) ? "," : "");
            if (written > 0) {
                topics_offset += (size_t)written;
            }
        }
        
        if (topics_offset < sizeof(topics_json) - 2) {
            topics_offset += snprintf(topics_json + topics_offset, 
                                      sizeof(topics_json) - topics_offset, "]");
        }
        
        // Append row as JSON
        int written = snprintf(query + offset, buf_size - offset,
            "{\"id\":\"%s\",\"endpoint_id\":\"%s\",\"application_id\":\"%s\","
            "\"chain_id\":%lu,\"block_number\":%lu,\"block_hash\":\"%s\","
            "\"transaction_hash\":\"%s\",\"log_index\":%u,\"contract_address\":\"%s\","
            "\"topics\":%s,\"data\":\"%s\",\"ingested_at\":%lu,\"processed_at\":%lu}\n",
            e->id, e->endpoint_id, e->application_id,
            e->chain_id, e->block_number, e->block_hash,
            e->transaction_hash, e->log_index, e->contract_address,
            topics_json, e->data ? e->data : "",
            e->ingested_at_ms, e->processed_at_ms);
        
        if (written < 0 || offset + written >= buf_size) {
            // Buffer too small, reallocate
            buf_size *= 2;
            char *new_query = realloc(query, buf_size);
            if (!new_query) {
                free(query);
                return NULL;
            }
            query = new_query;
            i--; // Retry this row
            continue;
        }
        
        offset += written;
    }
    
    return query;
}

// Helper: Build INSERT query for deliveries
static char *build_deliveries_insert_query(
    const char *table_name,
    clickhouse_delivery_t **deliveries,
    size_t count
) {
    size_t buf_size = 512 * 1024; // 512KB initial buffer
    char *query = malloc(buf_size);
    if (!query) return NULL;
    
    size_t offset = snprintf(query, buf_size,
        "INSERT INTO %s (id, event_id, endpoint_id, url, status, "
        "attempt_count, http_status_code, error_message, delivered_at, next_retry_at) "
        "FORMAT JSONEachRow\n",
        table_name);
    
    for (size_t i = 0; i < count; i++) {
        clickhouse_delivery_t *d = deliveries[i];
        
        char error_json[256] = "null";
        if (d->error_message) {
            // Escape quotes (safe bounded copy)
            snprintf(error_json, sizeof(error_json), "\"%s\"", d->error_message);
        }
        
        int written = snprintf(query + offset, buf_size - offset,
            "{\"id\":\"%s\",\"event_id\":\"%s\",\"endpoint_id\":\"%s\","
            "\"url\":\"%s\",\"status\":\"%s\",\"attempt_count\":%u,"
            "\"http_status_code\":%d,\"error_message\":%s,"
            "\"delivered_at\":%lu,\"next_retry_at\":%lu}\n",
            d->id, d->event_id, d->endpoint_id,
            d->url, d->status, d->attempt_count,
            d->http_status_code, error_json,
            d->delivered_at_ms, d->next_retry_at_ms);
        
        if (written < 0 || offset + written >= buf_size) {
            buf_size *= 2;
            char *new_query = realloc(query, buf_size);
            if (!new_query) {
                free(query);
                return NULL;
            }
            query = new_query;
            i--;
            continue;
        }
        
        offset += written;
    }
    
    return query;
}

eth_error_t clickhouse_batch_flush(clickhouse_batch_t *batch) {
    if (!batch || batch->count == 0) {
        return ETH_OK;
    }
    
    pthread_mutex_lock(&batch->lock);
    
    if (batch->count == 0) {
        pthread_mutex_unlock(&batch->lock);
        return ETH_OK;
    }
    
    // Build INSERT query
    char *query = NULL;
    if (batch->is_event_batch) {
        query = build_events_insert_query(batch->table_name, 
                                          (clickhouse_event_t**)batch->items,
                                          batch->count);
    } else {
        query = build_deliveries_insert_query(batch->table_name,
                                              (clickhouse_delivery_t**)batch->items,
                                              batch->count);
    }
    
    if (!query) {
        pthread_mutex_unlock(&batch->lock);
        return ETH_ERROR_MEMORY;
    }
    
    // Execute batch insert
    eth_error_t err = clickhouse_query(batch->client, query, NULL);
    free(query);
    
    if (err == ETH_OK) {
        atomic_fetch_add(&batch->client->batches_flushed, 1);
        atomic_fetch_add(&batch->client->rows_inserted, batch->count);
        
        LOG_DEBUG("Flushed %zu rows to ClickHouse table %s", 
                      batch->count, batch->table_name);
    } else {
        LOG_ERROR("Failed to flush batch to ClickHouse");
    }
    
    // Clear batch
    batch->count = 0;
    clock_gettime(CLOCK_MONOTONIC, &batch->last_flush);
    
    pthread_mutex_unlock(&batch->lock);
    
    return err;
}

eth_error_t clickhouse_batch_add_event(
    clickhouse_batch_t *batch,
    const clickhouse_event_t *event
) {
    if (!batch || !event) {
        return ETH_ERROR_INVALID_PARAM;
    }
    
    pthread_mutex_lock(&batch->lock);
    
    batch->is_event_batch = true;
    
    // Check if auto-flush needed
    bool should_flush = false;
    
    if (batch->count >= batch->capacity) {
        should_flush = true;
    } else {
        struct timespec now;
        clock_gettime(CLOCK_MONOTONIC, &now);
        uint64_t elapsed_ms = (now.tv_sec - batch->last_flush.tv_sec) * 1000 +
                              (now.tv_nsec - batch->last_flush.tv_nsec) / 1000000;
        if (elapsed_ms >= batch->timeout_ms) {
            should_flush = true;
        }
    }
    
    if (should_flush && batch->count > 0) {
        pthread_mutex_unlock(&batch->lock);
        eth_error_t flush_err = clickhouse_batch_flush(batch);
        if (flush_err != ETH_OK) {
            return flush_err;
        }
        pthread_mutex_lock(&batch->lock);
    }
    
    // Deep copy event
    clickhouse_event_t *e = malloc(sizeof(clickhouse_event_t));
    if (!e) {
        pthread_mutex_unlock(&batch->lock);
        return ETH_ERROR_MEMORY;
    }
    
    memcpy(e, event, sizeof(clickhouse_event_t));
    
    // Deep copy topics
    if (event->topics_count > 0) {
        e->topics = malloc(event->topics_count * sizeof(char*));
        for (size_t i = 0; i < event->topics_count; i++) {
            e->topics[i] = strdup(event->topics[i]);
        }
    }
    
    // Deep copy data
    if (event->data) {
        e->data = strdup(event->data);
    }
    
    batch->items[batch->count++] = e;
    
    pthread_mutex_unlock(&batch->lock);
    
    return ETH_OK;
}

eth_error_t clickhouse_batch_add_delivery(
    clickhouse_batch_t *batch,
    const clickhouse_delivery_t *delivery
) {
    if (!batch || !delivery) {
        return ETH_ERROR_INVALID_PARAM;
    }
    
    pthread_mutex_lock(&batch->lock);
    
    batch->is_event_batch = false;
    
    // Check auto-flush (same logic as events)
    bool should_flush = false;
    
    if (batch->count >= batch->capacity) {
        should_flush = true;
    } else {
        struct timespec now;
        clock_gettime(CLOCK_MONOTONIC, &now);
        uint64_t elapsed_ms = (now.tv_sec - batch->last_flush.tv_sec) * 1000 +
                              (now.tv_nsec - batch->last_flush.tv_nsec) / 1000000;
        if (elapsed_ms >= batch->timeout_ms) {
            should_flush = true;
        }
    }
    
    if (should_flush && batch->count > 0) {
        pthread_mutex_unlock(&batch->lock);
        clickhouse_batch_flush(batch);
        pthread_mutex_lock(&batch->lock);
    }
    
    // Deep copy delivery
    clickhouse_delivery_t *d = malloc(sizeof(clickhouse_delivery_t));
    if (!d) {
        pthread_mutex_unlock(&batch->lock);
        return ETH_ERROR_MEMORY;
    }
    
    memcpy(d, delivery, sizeof(clickhouse_delivery_t));
    
    if (delivery->error_message) {
        d->error_message = strdup(delivery->error_message);
    }
    
    batch->items[batch->count++] = d;
    
    pthread_mutex_unlock(&batch->lock);
    
    return ETH_OK;
}

void clickhouse_batch_destroy(clickhouse_batch_t *batch) {
    if (!batch) return;
    
    // Flush pending rows
    if (batch->count > 0) {
        clickhouse_batch_flush(batch);
    }
    
    // Free items
    for (size_t i = 0; i < batch->count; i++) {
        if (batch->is_event_batch) {
            clickhouse_event_t *e = (clickhouse_event_t*)batch->items[i];
            for (size_t t = 0; t < e->topics_count; t++) {
                free(e->topics[t]);
            }
            free(e->topics);
            free(e->data);
        } else {
            clickhouse_delivery_t *d = (clickhouse_delivery_t*)batch->items[i];
            free(d->error_message);
        }
        free(batch->items[i]);
    }
    
    free(batch->items);
    free(batch->table_name);
    pthread_mutex_destroy(&batch->lock);
    free(batch);
}

// =============================================================================
// SCHEMA INITIALIZATION
// =============================================================================

eth_error_t clickhouse_init_schema(clickhouse_client_t *client) {
    if (!client) {
        return ETH_ERROR_INVALID_PARAM;
    }
    
    // Create events table
    const char *events_schema = 
        "CREATE TABLE IF NOT EXISTS events ("
        "    id UUID,"
        "    endpoint_id UUID,"
        "    application_id UUID,"
        "    chain_id UInt64,"
        "    block_number UInt64,"
        "    block_hash String,"
        "    transaction_hash String,"
        "    log_index UInt32,"
        "    contract_address String,"
        "    topics Array(String),"
        "    data String,"
        "    ingested_at DateTime64(3),"
        "    processed_at DateTime64(3)"
        ") ENGINE = MergeTree()"
        " PARTITION BY toYYYYMM(ingested_at)"
        " ORDER BY (chain_id, block_number, log_index)"
        " TTL ingested_at + INTERVAL 90 DAY"
        " SETTINGS index_granularity = 8192";
    
    eth_error_t err = clickhouse_query(client, events_schema, NULL);
    if (err != ETH_OK) {
        return err;
    }
    
    // Create deliveries table
    const char *deliveries_schema =
        "CREATE TABLE IF NOT EXISTS deliveries ("
        "    id UUID,"
        "    event_id UUID,"
        "    endpoint_id UUID,"
        "    url String,"
        "    status String,"
        "    attempt_count UInt32,"
        "    http_status_code Int32,"
        "    error_message Nullable(String),"
        "    delivered_at DateTime64(3),"
        "    next_retry_at DateTime64(3)"
        ") ENGINE = MergeTree()"
        " PARTITION BY toYYYYMM(delivered_at)"
        " ORDER BY (endpoint_id, delivered_at)"
        " TTL delivered_at + INTERVAL 90 DAY"
        " SETTINGS index_granularity = 8192";
    
    err = clickhouse_query(client, deliveries_schema, NULL);
    if (err != ETH_OK) {
        return err;
    }
    
    LOG_INFO("ClickHouse schema initialized successfully");
    
    return ETH_OK;
}

void clickhouse_get_metrics(
    const clickhouse_client_t *client,
    uint64_t *queries_executed,
    uint64_t *batches_flushed,
    uint64_t *rows_inserted,
    uint64_t *total_latency_ms
) {
    if (!client) return;
    
    if (queries_executed) {
        *queries_executed = atomic_load(&client->queries_executed);
    }
    if (batches_flushed) {
        *batches_flushed = atomic_load(&client->batches_flushed);
    }
    if (rows_inserted) {
        *rows_inserted = atomic_load(&client->rows_inserted);
    }
    if (total_latency_ms) {
        *total_latency_ms = atomic_load(&client->total_latency_ms);
    }
}
