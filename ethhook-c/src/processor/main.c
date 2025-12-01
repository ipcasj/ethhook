#include "ethhook/processor.h"
#include "ethhook/common.h"
#include <signal.h>
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
    
    eth_log_init("ethhook-processor");
    LOG_INFO("Starting EthHook Message Processor");
    
    // Load configuration
    eth_config_t *config = NULL;
    eth_error_t err = eth_config_load(argv[1], &config);
    if (err != ETH_OK) {
        LOG_ERROR("Failed to load configuration");
        return 1;
    }
    
    // Set defaults
    if (config->processor.worker_threads == 0) {
        config->processor.worker_threads = 4;
    }
    if (config->processor.batch_size == 0) {
        config->processor.batch_size = 100;
    }
    
    LOG_INFO("Configuration loaded: %d worker threads, batch size %d",
             config->processor.worker_threads, config->processor.batch_size);
    
    // Setup signal handlers
    signal(SIGINT, signal_handler);
    signal(SIGTERM, signal_handler);
    
    // Create processor context
    processor_ctx_t *ctx = NULL;
    err = processor_ctx_create(config, &ctx);
    if (err != ETH_OK) {
        LOG_ERROR("Failed to create processor context");
        eth_config_free(config);
        return 1;
    }
    
    // Start processor (blocking)
    LOG_INFO("Starting processor...");
    err = processor_run(ctx);
    if (err != ETH_OK) {
        LOG_ERROR("Processor failed: %d", err);
    }
    
    // Wait for shutdown signal
    while (g_running) {
        sleep(1);
    }
    
    LOG_INFO("Shutting down gracefully...");
    processor_stop(ctx);
    processor_ctx_destroy(ctx);
    eth_config_free(config);
    
    LOG_INFO("Shutdown complete");
    return 0;
}
