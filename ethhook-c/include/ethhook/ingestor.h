#ifndef ETHHOOK_INGESTOR_H
#define ETHHOOK_INGESTOR_H

#include <pthread.h>
#include "common.h"
#include <event2/event.h>

// WebSocket connection state
typedef struct {
    uint64_t chain_id;
    char *ws_url;
    struct lws *wsi;
    struct event_base *event_base;
    circuit_breaker_t circuit_breaker;
    eth_arena_t *arena;
    
    // Redis publisher
    void *redis_ctx;
    
    // Stats
    atomic_uint_fast64_t events_received;
    atomic_uint_fast64_t events_published;
    atomic_uint_fast64_t errors;
} ws_connection_t;

// Worker thread data
typedef struct {
    pthread_t thread;
    ws_connection_t *conn;
    eth_config_t *config;
    volatile bool running;
} worker_thread_t;

// Initialize WebSocket connection
eth_error_t ws_connection_init(ws_connection_t *conn, uint64_t chain_id, 
                                const char *ws_url, const char *redis_host, int redis_port);

// Start WebSocket connection (blocking)
eth_error_t ws_connection_start(ws_connection_t *conn);

// Stop WebSocket connection
void ws_connection_stop(ws_connection_t *conn);

// Cleanup
void ws_connection_cleanup(ws_connection_t *conn);

// Worker thread function
void *ingestor_worker_thread(void *arg);

#endif // ETHHOOK_INGESTOR_H
