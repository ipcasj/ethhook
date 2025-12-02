#define _GNU_SOURCE
#include "ethhook/admin_api.h"
#include "ethhook/common.h"
#include <signal.h>
#include <stdlib.h>
#include <string.h>
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
    
    eth_log_init("ethhook-admin-api");
    LOG_INFO("Starting EthHook Admin API");
    
    // Load configuration
    eth_config_t *config = NULL;
    eth_error_t err = eth_config_load(argv[1], &config);
    if (err != ETH_OK) {
        LOG_ERROR("Failed to load configuration");
        return 1;
    }
    
    // Set defaults
    if (config->admin_api.port == 0) {
        config->admin_api.port = 3000;
    }
    if (!config->admin_api.jwt_secret) {
        config->admin_api.jwt_secret = strdup("your-secret-key-change-me");
    }
    if (config->admin_api.jwt_expiry_hours == 0) {
        config->admin_api.jwt_expiry_hours = 24;
    }
    
    // Override from environment
    const char *env_port = getenv("PORT");
    if (env_port) {
        config->admin_api.port = atoi(env_port);
    }
    
    const char *env_jwt_secret = getenv("JWT_SECRET");
    if (env_jwt_secret) {
        free(config->admin_api.jwt_secret);
        config->admin_api.jwt_secret = strdup(env_jwt_secret);
    }
    
    LOG_INFO("Configuration loaded: port=%d", config->admin_api.port);
    
    // Setup signal handlers
    signal(SIGINT, signal_handler);
    signal(SIGTERM, signal_handler);
    
    // Create admin API context
    admin_api_ctx_t *ctx = NULL;
    err = admin_api_ctx_create(config, &ctx);
    if (err != ETH_OK) {
        LOG_ERROR("Failed to create admin API context");
        eth_config_free(config);
        return 1;
    }
    
    // Start server (non-blocking in background thread)
    LOG_INFO("Starting admin API server on port %d...", config->admin_api.port);
    err = admin_api_run(ctx);
    if (err != ETH_OK) {
        LOG_ERROR("Failed to start admin API server");
        admin_api_ctx_destroy(ctx);
        eth_config_free(config);
        return 1;
    }
    
    // Wait for shutdown signal
    while (g_running) {
        sleep(1);
    }
    
    LOG_INFO("Shutting down gracefully...");
    admin_api_stop(ctx);
    admin_api_ctx_destroy(ctx);
    eth_config_free(config);
    
    LOG_INFO("Shutdown complete");
    return 0;
}
