/**
 * @file main.c
 * @brief Example webhook receiver with HMAC verification
 *
 * Demonstrates how to receive and verify ETHhook webhooks in C.
 */

#include "ethhook/log.h"
#include "ethhook/arena.h"
#include <uv.h>
#include <openssl/hmac.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define PORT 8000
#define HMAC_SECRET "your-webhook-secret"

int main(int argc, char **argv) {
    (void)argc; (void)argv;

    log_init(LOG_LEVEL_INFO, LOG_FORMAT_TEXT, "webhook-receiver");

    LOG_INFO("webhook_receiver_starting", "port", PORT);

    /* TODO: Implement HTTP server */
    /* 1. Listen on PORT */
    /* 2. Parse HTTP POST requests */
    /* 3. Verify HMAC-SHA256 signature */
    /* 4. Process webhook payload */

    LOG_INFO("webhook_receiver_ready");

    uv_loop_t *loop = uv_default_loop();
    uv_run(loop, UV_RUN_DEFAULT);
    uv_loop_close(loop);

    return 0;
}
