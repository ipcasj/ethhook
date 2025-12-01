#include "ethhook/processor.h"
#include "ethhook/clickhouse.h"
#include <hiredis/hiredis.h>
#include <hiredis/adapters/libevent.h>
#include <event2/event.h>
#include <jansson.h>
#include <pthread.h>
#include <stdlib.h>
#include <string.h>

struct processor_ctx {
    eth_config_t *config;
    eth_db_t *db;
    clickhouse_client_t *ch_client;
    clickhouse_batch_t *event_batch;
    redisAsyncContext *redis_ctx;
    struct event_base *event_base;
    pthread_t *worker_threads;
    size_t num_workers;
    volatile bool running;
    pthread_mutex_t batch_lock;
};

// Process a single event
static void process_event(processor_ctx_t *proc_ctx, json_t *event_obj) {
    // Extract event data
    event_t event = {0};
    
    json_t *id = json_object_get(event_obj, "id");
    json_t *chain_id = json_object_get(event_obj, "chain_id");
    json_t *block_number = json_object_get(event_obj, "block_number");
    json_t *block_hash = json_object_get(event_obj, "block_hash");
    json_t *tx_hash = json_object_get(event_obj, "transaction_hash");
    json_t *log_index = json_object_get(event_obj, "log_index");
    json_t *address = json_object_get(event_obj, "contract_address");
    json_t *topics = json_object_get(event_obj, "topics");
    json_t *data = json_object_get(event_obj, "data");
    
    if (id && json_is_string(id)) {
        strncpy(event.event_id, json_string_value(id), sizeof(event.event_id) - 1);
    }
    if (chain_id && json_is_integer(chain_id)) {
        event.chain_id = json_integer_value(chain_id);
    }
    if (block_number && json_is_integer(block_number)) {
        event.block_number = json_integer_value(block_number);
    }
    if (block_hash && json_is_string(block_hash)) {
        strncpy(event.block_hash, json_string_value(block_hash), sizeof(event.block_hash) - 1);
    }
    if (tx_hash && json_is_string(tx_hash)) {
        strncpy(event.transaction_hash, json_string_value(tx_hash), sizeof(event.transaction_hash) - 1);
    }
    if (log_index && json_is_integer(log_index)) {
        event.log_index = json_integer_value(log_index);
    }
    if (address && json_is_string(address)) {
        strncpy(event.contract_address, json_string_value(address), sizeof(event.contract_address) - 1);
    }
    
    // Parse topics array
    if (topics && json_is_array(topics)) {
        event.num_topics = json_array_size(topics);
        if (event.num_topics > 0) {
            event.topics = calloc(event.num_topics, sizeof(char *));
            for (size_t i = 0; i < event.num_topics; i++) {
                json_t *topic = json_array_get(topics, i);
                if (json_is_string(topic)) {
                    event.topics[i] = strdup(json_string_value(topic));
                }
            }
        }
    }
    
    if (data && json_is_string(data)) {
        event.data = strdup(json_string_value(data));
    }
    
    // Find matching endpoints
    endpoint_t **endpoints = NULL;
    size_t endpoint_count = 0;
    
    eth_error_t err = matcher_find_endpoints(&event, &endpoints, &endpoint_count);
    if (err == ETH_OK && endpoint_count > 0) {
        LOG_INFO("Event %s matched %zu endpoints", event.event_id, endpoint_count);
        
        // Convert to ClickHouse event format
        clickhouse_event_t ch_event = {0};
        strncpy(ch_event.id, event.event_id, sizeof(ch_event.id) - 1);
        ch_event.chain_id = event.chain_id;
        ch_event.block_number = event.block_number;
        strncpy(ch_event.block_hash, event.block_hash, sizeof(ch_event.block_hash) - 1);
        strncpy(ch_event.transaction_hash, event.transaction_hash, sizeof(ch_event.transaction_hash) - 1);
        ch_event.log_index = event.log_index;
        strncpy(ch_event.contract_address, event.contract_address, sizeof(ch_event.contract_address) - 1);
        ch_event.topics = event.topics;
        ch_event.num_topics = event.num_topics;
        ch_event.data = event.data;
        ch_event.ingested_at = time(NULL);
        
        // For each matched endpoint, add to ClickHouse batch
        for (size_t i = 0; i < endpoint_count; i++) {
            strncpy(ch_event.endpoint_id, endpoints[i]->endpoint_id, sizeof(ch_event.endpoint_id) - 1);
            strncpy(ch_event.application_id, endpoints[i]->application_id, sizeof(ch_event.application_id) - 1);
            
            // Add to batch (thread-safe)
            pthread_mutex_lock(&proc_ctx->batch_lock);
            clickhouse_batch_add_event(proc_ctx->event_batch, &ch_event);
            pthread_mutex_unlock(&proc_ctx->batch_lock);
        }
        
        // Publish matched events to delivery queue
        for (size_t i = 0; i < endpoint_count; i++) {
            json_t *delivery = json_object();
            json_object_set_new(delivery, "event_id", json_string(event.event_id));
            json_object_set_new(delivery, "endpoint_id", json_string(endpoints[i]->endpoint_id));
            json_object_set_new(delivery, "webhook_url", json_string(endpoints[i]->webhook_url));
            if (endpoints[i]->webhook_secret) {
                json_object_set_new(delivery, "webhook_secret", json_string(endpoints[i]->webhook_secret));
            }
            
            char *delivery_json = json_dumps(delivery, JSON_COMPACT);
            json_decref(delivery);
            
            if (delivery_json) {
                // Publish to Redis delivery queue
                redisAsyncCommand(proc_ctx->redis_ctx, NULL, NULL,
                                "XADD deliveries:* * delivery %s", delivery_json);
                free(delivery_json);
            }
        }
        
        matcher_free_endpoints(endpoints, endpoint_count);
    }
    
    // Cleanup
    if (event.topics) {
        for (size_t i = 0; i < event.num_topics; i++) {
            free(event.topics[i]);
        }
        free(event.topics);
    }
    free(event.data);
}

// Redis consumer callback
static void on_redis_message(redisAsyncContext *ctx, void *reply, void *privdata) {
    processor_ctx_t *proc_ctx = (processor_ctx_t *)privdata;
    
    if (reply == NULL) {
        return;
    }
    
    redisReply *r = (redisReply *)reply;
    
    // Parse XREAD response
    if (r->type == REDIS_REPLY_ARRAY && r->elements > 0) {
        for (size_t i = 0; i < r->elements; i++) {
            redisReply *stream = r->element[i];
            if (stream->type == REDIS_REPLY_ARRAY && stream->elements == 2) {
                // stream->element[0] is stream name
                // stream->element[1] is array of messages
                
                redisReply *messages = stream->element[1];
                if (messages->type == REDIS_REPLY_ARRAY) {
                    for (size_t j = 0; j < messages->elements; j++) {
                        redisReply *message = messages->element[j];
                        if (message->type == REDIS_REPLY_ARRAY && message->elements == 2) {
                            // message->element[0] is message ID
                            // message->element[1] is field-value array
                            
                            redisReply *fields = message->element[1];
                            if (fields->type == REDIS_REPLY_ARRAY) {
                                // Parse event JSON
                                for (size_t k = 0; k < fields->elements; k += 2) {
                                    if (strcmp(fields->element[k]->str, "event") == 0) {
                                        const char *event_json = fields->element[k + 1]->str;
                                        
                                        // Parse and process event
                                        json_error_t error;
                                        json_t *event_obj = json_loads(event_json, 0, &error);
                                        if (event_obj) {
                                            process_event(proc_ctx, event_obj);
                                            json_decref(event_obj);
                                        } else {
                                            LOG_ERROR("Failed to parse event JSON: %s", error.text);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Continue reading
    if (proc_ctx->running) {
        redisAsyncCommand(ctx, on_redis_message, privdata,
                         "XREAD BLOCK 1000 STREAMS events:* >");
    }
}

static void redis_connect_callback(const redisAsyncContext *ctx, int status) {
    if (status != REDIS_OK) {
        LOG_ERROR("Redis connection error: %s", ctx->errstr);
        return;
    }
    
    LOG_INFO("Connected to Redis");
}

static void redis_disconnect_callback(const redisAsyncContext *ctx, int status) {
    if (status != REDIS_OK) {
        LOG_ERROR("Redis disconnection error: %s", ctx->errstr);
        return;
    }
    
    LOG_INFO("Disconnected from Redis");
}

eth_error_t processor_ctx_create(eth_config_t *config, processor_ctx_t **ctx) {
    processor_ctx_t *proc_ctx = calloc(1, sizeof(processor_ctx_t));
    if (!proc_ctx) {
        return ETH_ERROR_MEMORY;
    }
    
    proc_ctx->config = config;
    proc_ctx->running = true;
    pthread_mutex_init(&proc_ctx->batch_lock, NULL);
    
    // Open database
    eth_error_t err = eth_db_open(config->database_url, &proc_ctx->db);
    if (err != ETH_OK) {
        pthread_mutex_destroy(&proc_ctx->batch_lock);
        free(proc_ctx);
        return err;
    }
    
    // Initialize ClickHouse client
    err = clickhouse_client_create(config, &proc_ctx->ch_client);
    if (err != ETH_OK) {
        LOG_ERROR("Failed to create ClickHouse client");
        eth_db_close(proc_ctx->db);
        pthread_mutex_destroy(&proc_ctx->batch_lock);
        free(proc_ctx);
        return err;
    }
    
    // Create event batch
    uint32_t batch_size = config->clickhouse.batch_size > 0 ? 
                         config->clickhouse.batch_size : 1000;
    uint32_t timeout_ms = config->clickhouse.batch_timeout_ms > 0 ?
                         config->clickhouse.batch_timeout_ms : 1000;
    
    err = clickhouse_batch_create(proc_ctx->ch_client, "events", 
                                  batch_size, timeout_ms, &proc_ctx->event_batch);
    if (err != ETH_OK) {
        LOG_ERROR("Failed to create event batch");
        clickhouse_client_destroy(proc_ctx->ch_client);
        eth_db_close(proc_ctx->db);
        pthread_mutex_destroy(&proc_ctx->batch_lock);
        free(proc_ctx);
        return err;
    }
    
    LOG_INFO("ClickHouse batch initialized: size=%u, timeout=%ums", batch_size, timeout_ms);
    
    // Initialize matcher
    err = matcher_init(proc_ctx->db);
    if (err != ETH_OK) {
        clickhouse_batch_destroy(proc_ctx->event_batch);
        clickhouse_client_destroy(proc_ctx->ch_client);
        eth_db_close(proc_ctx->db);
        pthread_mutex_destroy(&proc_ctx->batch_lock);
        free(proc_ctx);
        return err;
    }
    
    // Create event base
    proc_ctx->event_base = event_base_new();
    if (!proc_ctx->event_base) {
        matcher_cleanup();
        clickhouse_batch_destroy(proc_ctx->event_batch);
        clickhouse_client_destroy(proc_ctx->ch_client);
        eth_db_close(proc_ctx->db);
        pthread_mutex_destroy(&proc_ctx->batch_lock);
        free(proc_ctx);
        return ETH_ERROR;
    }
    
    // Connect to Redis
    proc_ctx->redis_ctx = redisAsyncConnect(config->redis_host, config->redis_port);
    if (proc_ctx->redis_ctx == NULL || proc_ctx->redis_ctx->err) {
        if (proc_ctx->redis_ctx) {
            LOG_ERROR("Redis connection error: %s", proc_ctx->redis_ctx->errstr);
            redisAsyncFree(proc_ctx->redis_ctx);
        } else {
            LOG_ERROR("Failed to allocate Redis context");
        }
        event_base_free(proc_ctx->event_base);
        matcher_cleanup();
        clickhouse_batch_destroy(proc_ctx->event_batch);
        clickhouse_client_destroy(proc_ctx->ch_client);
        eth_db_close(proc_ctx->db);
        pthread_mutex_destroy(&proc_ctx->batch_lock);
        free(proc_ctx);
        return ETH_ERROR_REDIS;
    }
    
    // Attach Redis to event loop
    redisLibeventAttach(proc_ctx->redis_ctx, proc_ctx->event_base);
    redisAsyncSetConnectCallback(proc_ctx->redis_ctx, redis_connect_callback);
    redisAsyncSetDisconnectCallback(proc_ctx->redis_ctx, redis_disconnect_callback);
    
    *ctx = proc_ctx;
    return ETH_OK;
}

void processor_ctx_destroy(processor_ctx_t *ctx) {
    if (!ctx) {
        return;
    }
    
    if (ctx->redis_ctx) {
        redisAsyncDisconnect(ctx->redis_ctx);
    }
    
    if (ctx->event_base) {
        event_base_free(ctx->event_base);
    }
    
    // Flush remaining events
    if (ctx->event_batch) {
        pthread_mutex_lock(&ctx->batch_lock);
        clickhouse_batch_flush(ctx->event_batch);
        clickhouse_batch_destroy(ctx->event_batch);
        pthread_mutex_unlock(&ctx->batch_lock);
    }
    
    if (ctx->ch_client) {
        clickhouse_client_destroy(ctx->ch_client);
    }
    
    pthread_mutex_destroy(&ctx->batch_lock);
    matcher_cleanup();
    eth_db_close(ctx->db);
    free(ctx);
}

eth_error_t processor_run(processor_ctx_t *ctx) {
    if (!ctx) {
        return ETH_ERROR_INVALID_PARAM;
    }
    
    // Start consuming from Redis
    redisAsyncCommand(ctx->redis_ctx, on_redis_message, ctx,
                     "XREAD BLOCK 1000 STREAMS events:* >");
    
    // Run event loop
    event_base_dispatch(ctx->event_base);
    
    return ETH_OK;
}

void processor_stop(processor_ctx_t *ctx) {
    if (ctx) {
        ctx->running = false;
        if (ctx->event_base) {
            event_base_loopbreak(ctx->event_base);
        }
    }
}
