/**
 * @file main.c
 * @brief Admin API Service - REST API for managing webhooks
 *
 * Provides REST endpoints for CRUD operations on applications and endpoints.
 */

#include "ethhook/arena.h"
#include "ethhook/log.h"
#include <uv.h>
#include <libpq-fe.h>
#include <stdlib.h>

int main(int argc, char **argv) {
    (void)argc; (void)argv;

    log_init(LOG_LEVEL_INFO, LOG_FORMAT_TEXT, "admin-api");
    LOG_INFO("service_starting", "version", "1.0.0");

    /* TODO: Implement full admin API */
    /* 1. Start HTTP server (libevent or uv_tcp) */
    /* 2. Handle REST endpoints */
    /* 3. JWT authentication */
    /* 4. PostgreSQL CRUD operations */

    LOG_INFO("service_ready", "port", "8080");

    uv_loop_t *loop = uv_default_loop();
    uv_run(loop, UV_RUN_DEFAULT);
    uv_loop_close(loop);

    LOG_INFO("service_stopped");
    return 0;
}
