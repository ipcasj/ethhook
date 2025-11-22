/**
 * @file main.c
 * @brief Event Ingestor Service - WebSocket listener for Ethereum events
 *
 * Single Translation Unit implementation of the event ingestor.
 * Uses libuv for async I/O, libwebsockets for WebSocket connection,
 * hiredis for Redis publishing, and arena allocation for memory management.
 *
 * Architecture:
 * 1. Connect to Ethereum RPC via WebSocket
 * 2. Subscribe to new blocks (eth_subscribe "newHeads")
 * 3. For each block, fetch logs (eth_getLogs)
 * 4. Deduplicate events using Redis SET
 * 5. Publish to Redis Streams for message-processor
 *
 * Performance target: 10,000+ events/second
 */

#include "ethhook/arena.h"
#include "ethhook/log.h"
#include "ethhook/types.h"

#include <uv.h>
#include <hiredis/hiredis.h>
#include <hiredis/async.h>
#include <hiredis/adapters/libuv.h>
#include <libpq-fe.h>
#include <stdlib.h>
#include <string.h>
#include <signal.h>
#include <unistd.h>

/* ========================================================================
 * Configuration
 * ======================================================================== */

typedef struct {
    const char *eth_ws_url;
    const char *redis_url;
    const char *database_url;
    uint64_t chain_id;
    uint16_t metrics_port;
} config_t;

static config_t load_config(void) {
    config_t cfg = {
        .eth_ws_url = getenv("ETHEREUM_WS_URL"),
        .redis_url = getenv("REDIS_URL"),
        .database_url = getenv("DATABASE_URL"),
        .chain_id = 11155111,  /* Sepolia testnet */
        .metrics_port = 9090,
    };

    if (!cfg.eth_ws_url) {
        fprintf(stderr, "FATAL: ETHEREUM_WS_URL not set\n");
        exit(1);
    }

    if (!cfg.redis_url) {
        cfg.redis_url = "redis://localhost:6379";
    }

    return cfg;
}

/* ========================================================================
 * Global State
 * ======================================================================== */

typedef struct {
    uv_loop_t *loop;
    redisAsyncContext *redis;
    PGconn *db;
    config_t config;
    bool shutdown_requested;
    uint64_t events_ingested;
    uint64_t events_published;
} app_state_t;

static app_state_t g_app;

/* ========================================================================
 * Signal Handling
 * ======================================================================== */

static void signal_handler(uv_signal_t *handle, int signum) {
    LOG_INFO("shutdown_requested", "signal", signum == SIGINT ? "SIGINT" : "SIGTERM");
    g_app.shutdown_requested = true;
    uv_stop(g_app.loop);
}

/* ========================================================================
 * Redis Connection
 * ======================================================================== */

static void redis_connect_cb(const redisAsyncContext *ac, int status) {
    if (status != REDIS_OK) {
        LOG_ERROR("redis_connect_failed", "error", ac->errstr);
        return;
    }
    LOG_INFO("redis_connected");
}

static void redis_disconnect_cb(const redisAsyncContext *ac, int status) {
    if (status != REDIS_OK) {
        LOG_ERROR("redis_disconnected", "error", ac->errstr);
    } else {
        LOG_INFO("redis_disconnected");
    }
}

static int init_redis(app_state_t *app) {
    /* Parse Redis URL (simplified - production would use proper URL parser) */
    const char *host = "127.0.0.1";
    int port = 6379;

    app->redis = redisAsyncConnect(host, port);
    if (app->redis == NULL || app->redis->err) {
        if (app->redis) {
            LOG_ERROR("redis_connect_failed", "error", app->redis->errstr);
            redisAsyncFree(app->redis);
        } else {
            LOG_ERROR("redis_connect_failed", "error", "allocation_failed");
        }
        return -1;
    }

    /* Attach to libuv event loop */
    if (redisLibuvAttach(app->redis, app->loop) != REDIS_OK) {
        LOG_ERROR("redis_libuv_attach_failed");
        redisAsyncFree(app->redis);
        return -1;
    }

    /* Set connection callbacks */
    redisAsyncSetConnectCallback(app->redis, redis_connect_cb);
    redisAsyncSetDisconnectCallback(app->redis, redis_disconnect_cb);

    return 0;
}

/* ========================================================================
 * Event Publishing
 * ======================================================================== */

static void publish_event_cb(redisAsyncContext *ac, void *reply, void *privdata) {
    (void)ac;
    (void)privdata;

    if (reply == NULL) {
        LOG_WARN("redis_publish_failed", "error", "null_reply");
        return;
    }

    g_app.events_published++;

    /* Log every 1000 events */
    if (g_app.events_published % 1000 == 0) {
        LOG_INFO("events_published", "count", "1000");
    }
}

static void publish_to_redis(const char *stream_key, const char *event_json) {
    if (!g_app.redis) {
        return;
    }

    /* Use XADD command to add to Redis Stream
     * XADD events:eth * data <json> */
    redisAsyncCommand(g_app.redis, publish_event_cb, NULL,
                     "XADD %s * data %s", stream_key, event_json);
}

/* ========================================================================
 * WebSocket Handler (Placeholder - full implementation would use libwebsockets)
 * ======================================================================== */

static void simulate_event_ingestion(uv_timer_t *handle) {
    (void)handle;

    /* This is a simulation - real implementation would:
     * 1. Connect to WebSocket
     * 2. Subscribe to newHeads
     * 3. Fetch logs for each block
     * 4. Parse events
     * 5. Deduplicate
     * 6. Publish to Redis
     */

    /* Simulate receiving an event */
    arena_t *arena = arena_create(4096);
    if (!arena) {
        return;
    }

    /* Create mock event JSON */
    const char *event_json = "{\"block_number\":12345,\"transaction_hash\":\"0xabc123\"}";

    /* Publish to Redis Stream */
    publish_to_redis("events:eth", event_json);

    g_app.events_ingested++;

    arena_destroy(arena);

    /* Log every 100 events */
    if (g_app.events_ingested % 100 == 0) {
        LOG_DEBUG("events_ingested", "count", "100");
    }
}

/* ========================================================================
 * Health Check Server
 * ======================================================================== */

static void health_check_handler(uv_timer_t *handle) {
    (void)handle;

    /* Simple HTTP server would go here */
    /* For now, just log metrics */
    if (g_app.events_ingested % 5000 == 0 && g_app.events_ingested > 0) {
        LOG_INFO("metrics",
                "events_ingested", (long long)g_app.events_ingested,
                "events_published", (long long)g_app.events_published);
    }
}

/* ========================================================================
 * Main Application
 * ======================================================================== */

int main(int argc, char **argv) {
    (void)argc;
    (void)argv;

    /* Initialize logging */
    const char *log_level_str = getenv("LOG_LEVEL");
    log_level_t log_level = log_level_from_string(log_level_str);

    const char *log_format_str = getenv("LOG_FORMAT");
    log_format_t log_format = (log_format_str && strcmp(log_format_str, "json") == 0)
                              ? LOG_FORMAT_JSON : LOG_FORMAT_TEXT;

    log_init(log_level, log_format, "event-ingestor");

    LOG_INFO("service_starting", "version", "1.0.0");

    /* Load configuration */
    g_app.config = load_config();
    g_app.shutdown_requested = false;
    g_app.events_ingested = 0;
    g_app.events_published = 0;

    LOG_INFO("config_loaded",
            "eth_ws_url", g_app.config.eth_ws_url,
            "chain_id", (long long)g_app.config.chain_id);

    /* Initialize libuv event loop */
    g_app.loop = uv_default_loop();

    /* Initialize Redis */
    if (init_redis(&g_app) != 0) {
        LOG_ERROR("redis_init_failed");
        return 1;
    }

    /* Set up signal handlers for graceful shutdown */
    uv_signal_t sigint_handle, sigterm_handle;
    uv_signal_init(g_app.loop, &sigint_handle);
    uv_signal_start(&sigint_handle, signal_handler, SIGINT);

    uv_signal_init(g_app.loop, &sigterm_handle);
    uv_signal_start(&sigterm_handle, signal_handler, SIGTERM);

    /* Start event ingestion timer (simulation) */
    uv_timer_t ingestion_timer;
    uv_timer_init(g_app.loop, &ingestion_timer);
    uv_timer_start(&ingestion_timer, simulate_event_ingestion, 0, 10);  /* Every 10ms */

    /* Start health check timer */
    uv_timer_t health_timer;
    uv_timer_init(g_app.loop, &health_timer);
    uv_timer_start(&health_timer, health_check_handler, 1000, 5000);  /* Every 5s */

    LOG_INFO("service_ready",
            "metrics_port", (int)g_app.config.metrics_port);

    /* Run event loop */
    int ret = uv_run(g_app.loop, UV_RUN_DEFAULT);

    /* Cleanup */
    LOG_INFO("service_stopping");

    uv_timer_stop(&ingestion_timer);
    uv_timer_stop(&health_timer);
    uv_signal_stop(&sigint_handle);
    uv_signal_stop(&sigterm_handle);

    if (g_app.redis) {
        redisAsyncFree(g_app.redis);
    }

    uv_loop_close(g_app.loop);

    LOG_INFO("service_stopped",
            "total_events_ingested", (long long)g_app.events_ingested,
            "total_events_published", (long long)g_app.events_published);

    return ret;
}
