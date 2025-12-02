#define _POSIX_C_SOURCE 200809L
#define _GNU_SOURCE
#include "ethhook/delivery.h"
#include "ethhook/clickhouse.h"
#include <hiredis/hiredis.h>
#include <hiredis/adapters/libevent.h>
#include <event2/event.h>
#include <pthread.h>
#include <stdlib.h>
#include <string.h>
#include "yyjson.h"
#include <unistd.h>

struct delivery_ctx {
    eth_config_t *config;
    eth_db_t *db;
    clickhouse_client_t *ch_client;
    clickhouse_batch_t *delivery_batch;
    redisAsyncContext *redis_ctx;
    struct event_base *event_base;
    pthread_t *worker_threads;
    size_t num_workers;
    volatile bool running;
    retry_policy_t retry_policy;
    pthread_mutex_t batch_lock;
};

// Worker thread function
static void *delivery_worker(void *arg) {
    delivery_ctx_t *ctx = (delivery_ctx_t *)arg;
    http_client_t client;
    
    if (http_client_init(&client) != ETH_OK) {
        LOG_ERROR("Failed to initialize HTTP client");
        return NULL;
    }
    
    LOG_INFO("Delivery worker started");
    
    while (ctx->running) {
        // TODO: Consume from Redis delivery queue
        // Format: XREAD BLOCK 1000 STREAMS deliveries:* >
        
        // For now, just sleep
        usleep(100000); // 100ms
    }
    
    http_client_cleanup(&client);
    LOG_INFO("Delivery worker stopped");
    return NULL;
}

// Redis callback for delivery requests
static void on_delivery_request(redisAsyncContext *redis_ctx, void *reply, void *privdata) {
    delivery_ctx_t *ctx = (delivery_ctx_t *)privdata;
    
    if (reply == NULL) {
        return;
    }
    
    redisReply *r = (redisReply *)reply;
    
    // Parse delivery request
    if (r->type == REDIS_REPLY_ARRAY && r->elements > 0) {
        // TODO: Parse and queue delivery requests
    }
    
    // Continue reading
    if (ctx->running) {
        redisAsyncCommand(redis_ctx, on_delivery_request, privdata,
                         "XREAD BLOCK 1000 STREAMS deliveries:* >");
    }
}

eth_error_t delivery_ctx_create(eth_config_t *config, delivery_ctx_t **ctx) {
    delivery_ctx_t *del_ctx = calloc(1, sizeof(delivery_ctx_t));
    if (!del_ctx) {
        return ETH_ERROR_MEMORY;
    }
    
    del_ctx->config = config;
    del_ctx->running = true;
    pthread_mutex_init(&del_ctx->batch_lock, NULL);
    
    // Initialize retry policy
    del_ctx->retry_policy.max_retries = config->delivery.max_retries;
    del_ctx->retry_policy.base_delay_ms = 1000;
    del_ctx->retry_policy.max_delay_ms = 60000;
    del_ctx->retry_policy.backoff_multiplier = 2.0;
    
    // Open database
    eth_error_t err = eth_db_open(config->database_url, &del_ctx->db);
    if (err != ETH_OK) {
        pthread_mutex_destroy(&del_ctx->batch_lock);
        free(del_ctx);
        return err;
    }
    
    // Initialize ClickHouse client
    clickhouse_config_t ch_config = {
        .url = config->database_url,
        .database = "ethhook",
        .user = NULL,
        .password = NULL,
        .pool_size = 10,
        .timeout_ms = 30000,
        .enable_compression = true,
        .batch_size = 1000,
        .batch_timeout_ms = 1000
    };
    
    err = clickhouse_client_create(&ch_config, &del_ctx->ch_client);
    if (err != ETH_OK) {
        LOG_ERROR("Failed to create ClickHouse client");
        eth_db_close(del_ctx->db);
        pthread_mutex_destroy(&del_ctx->batch_lock);
        free(del_ctx);
        return err;
    }
    
    // Create delivery batch
    size_t batch_capacity = ch_config.batch_size;
    
    err = clickhouse_batch_create(del_ctx->ch_client, "deliveries", 
                                  batch_capacity, &del_ctx->delivery_batch);
    if (err != ETH_OK) {
        LOG_ERROR("Failed to create delivery batch");
        clickhouse_client_destroy(del_ctx->ch_client);
        eth_db_close(del_ctx->db);
        pthread_mutex_destroy(&del_ctx->batch_lock);
        free(del_ctx);
        return err;
    }
    
    // Create event base
    del_ctx->event_base = event_base_new();
    if (!del_ctx->event_base) {
        clickhouse_batch_destroy(del_ctx->delivery_batch);
        clickhouse_client_destroy(del_ctx->ch_client);
        eth_db_close(del_ctx->db);
        pthread_mutex_destroy(&del_ctx->batch_lock);
        free(del_ctx);
        return ETH_ERROR;
    }
    
    // Connect to Redis
    del_ctx->redis_ctx = redisAsyncConnect(config->redis_host, config->redis_port);
    if (del_ctx->redis_ctx == NULL || del_ctx->redis_ctx->err) {
        if (del_ctx->redis_ctx) {
            LOG_ERROR("Redis connection error: %s", del_ctx->redis_ctx->errstr);
            redisAsyncFree(del_ctx->redis_ctx);
        }
        event_base_free(del_ctx->event_base);
        eth_db_close(del_ctx->db);
        free(del_ctx);
        return ETH_ERROR_REDIS;
    }
    
    redisLibeventAttach(del_ctx->redis_ctx, del_ctx->event_base);
    
    // Create worker threads
    del_ctx->num_workers = config->delivery.worker_threads;
    del_ctx->worker_threads = calloc(del_ctx->num_workers, sizeof(pthread_t));
    if (!del_ctx->worker_threads) {
        redisAsyncDisconnect(del_ctx->redis_ctx);
        event_base_free(del_ctx->event_base);
        eth_db_close(del_ctx->db);
        free(del_ctx);
        return ETH_ERROR_MEMORY;
    }
    
    *ctx = del_ctx;
    return ETH_OK;
}

void delivery_ctx_destroy(delivery_ctx_t *ctx) {
    if (!ctx) {
        return;
    }
    
    if (ctx->worker_threads) {
        free(ctx->worker_threads);
    }
    
    if (ctx->redis_ctx) {
        redisAsyncDisconnect(ctx->redis_ctx);
    }
    
    if (ctx->event_base) {
        event_base_free(ctx->event_base);
    }
    
    // Flush remaining deliveries
    if (ctx->delivery_batch) {
        pthread_mutex_lock(&ctx->batch_lock);
        clickhouse_batch_flush(ctx->delivery_batch);
        clickhouse_batch_destroy(ctx->delivery_batch);
        pthread_mutex_unlock(&ctx->batch_lock);
    }
    
    if (ctx->ch_client) {
        clickhouse_client_destroy(ctx->ch_client);
    }
    
    pthread_mutex_destroy(&ctx->batch_lock);
    eth_db_close(ctx->db);
    free(ctx);
}

eth_error_t delivery_run(delivery_ctx_t *ctx) {
    if (!ctx) {
        return ETH_ERROR_INVALID_PARAM;
    }
    
    // Start worker threads
    for (size_t i = 0; i < ctx->num_workers; i++) {
        if (pthread_create(&ctx->worker_threads[i], NULL, delivery_worker, ctx) != 0) {
            LOG_ERROR("Failed to create worker thread %zu", i);
            return ETH_ERROR;
        }
    }
    
    // Start consuming from Redis
    redisAsyncCommand(ctx->redis_ctx, on_delivery_request, ctx,
                     "XREAD BLOCK 1000 STREAMS deliveries:* >");
    
    // Run event loop
    event_base_dispatch(ctx->event_base);
    
    return ETH_OK;
}

void delivery_stop(delivery_ctx_t *ctx) {
    if (!ctx) {
        return;
    }
    
    ctx->running = false;
    
    if (ctx->event_base) {
        event_base_loopbreak(ctx->event_base);
    }
    
    // Wait for workers
    if (ctx->worker_threads) {
        for (size_t i = 0; i < ctx->num_workers; i++) {
            pthread_join(ctx->worker_threads[i], NULL);
        }
    }
}
