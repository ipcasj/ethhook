#include "ethhook/delivery.h"
#include "ethhook/common.h"
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

static volatile bool g_running = true;

static void signal_handler(int signum) {
    (void)signum;
    g_running = false;
}

int main(int argc, char **argv) {
    if (argc < 2) {
        fprintf(stderr, "Usage: %s <config_file>\n", argv[0]);
        return 1;
    }
    
    eth_log_init("ethhook-delivery");
    LOG_INFO("Starting EthHook Webhook Delivery");
    
    // Load configuration
    eth_config_t *config = NULL;
    eth_error_t err = eth_config_load(argv[1], &config);
    if (err != ETH_OK) {
        LOG_ERROR("Failed to load configuration");
        return 1;
    }
    
    // Set defaults
    if (config->delivery.worker_threads == 0) {
        config->delivery.worker_threads = 8;
    }
    if (config->delivery.max_retries == 0) {
        config->delivery.max_retries = 5;
    }
    if (config->delivery.timeout_ms == 0) {
        config->delivery.timeout_ms = 30000; // 30 seconds
    }
    
    LOG_INFO("Configuration loaded: %d worker threads, max %d retries",
             config->delivery.worker_threads, config->delivery.max_retries);
    
    // Setup signal handlers
    signal(SIGINT, signal_handler);
    signal(SIGTERM, signal_handler);
    
    // Create delivery context
    delivery_ctx_t *ctx = NULL;
    err = delivery_ctx_create(config, &ctx);
    if (err != ETH_OK) {
        LOG_ERROR("Failed to create delivery context");
        eth_config_free(config);
        return 1;
    }
    
    // Start delivery (blocking)
    LOG_INFO("Starting delivery workers...");
    err = delivery_run(ctx);
    if (err != ETH_OK) {
        LOG_ERROR("Delivery failed: %d", err);
    }
    
    // Wait for shutdown signal
    while (g_running) {
        sleep(1);
    }
    
    LOG_INFO("Shutting down gracefully...");
    delivery_stop(ctx);
    delivery_ctx_destroy(ctx);
    eth_config_free(config);
    
    LOG_INFO("Shutdown complete");
    return 0;
}
