/**
 * @file main.c
 * @brief Webhook Delivery Service - HTTP webhook sender
 *
 * Delivers webhooks with HMAC signatures, retries, and circuit breaking.
 */

#include "ethhook/arena.h"
#include "ethhook/log.h"
#include <uv.h>
#include <curl/curl.h>
#include <stdlib.h>

int main(int argc, char **argv) {
    (void)argc; (void)argv;

    log_init(LOG_LEVEL_INFO, LOG_FORMAT_TEXT, "webhook-delivery");
    LOG_INFO("service_starting", "version", "1.0.0");

    /* Initialize libcurl */
    curl_global_init(CURL_GLOBAL_ALL);

    /* TODO: Implement full webhook delivery */
    /* 1. Read from delivery queue (Redis) */
    /* 2. Sign payload with HMAC-SHA256 */
    /* 3. Send HTTP POST with libcurl multi */
    /* 4. Handle retries with exponential backoff */

    LOG_INFO("service_ready");

    uv_loop_t *loop = uv_default_loop();
    uv_run(loop, UV_RUN_DEFAULT);
    uv_loop_close(loop);

    curl_global_cleanup();

    LOG_INFO("service_stopped");
    return 0;
}
