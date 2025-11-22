/**
 * @file main.c
 * @brief Message Processor Service - Event filtering and routing
 *
 * Consumes events from Redis Streams, matches against endpoint filters,
 * and queues for delivery.
 */

#include "ethhook/arena.h"
#include "ethhook/log.h"
#include <uv.h>
#include <stdlib.h>

int main(int argc, char **argv) {
    (void)argc; (void)argv;

    log_init(LOG_LEVEL_INFO, LOG_FORMAT_TEXT, "message-processor");
    LOG_INFO("service_starting", "version", "1.0.0");

    /* TODO: Implement full message processor */
    /* 1. Connect to Redis Streams */
    /* 2. Load endpoint filters from PostgreSQL */
    /* 3. Match events against filters */
    /* 4. Queue for webhook delivery */

    LOG_INFO("service_ready");

    /* Event loop would run here */
    uv_loop_t *loop = uv_default_loop();
    uv_run(loop, UV_RUN_DEFAULT);
    uv_loop_close(loop);

    LOG_INFO("service_stopped");
    return 0;
}
