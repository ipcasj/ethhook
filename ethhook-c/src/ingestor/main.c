#include "ethhook/ingestor.h"
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
    
    eth_log_init("ethhook-ingestor");
    LOG_INFO("Starting EthHook Event Ingestor");
    
    // Load configuration
    eth_config_t *config = NULL;
    eth_error_t err = eth_config_load(argv[1], &config);
    if (err != ETH_OK) {
        LOG_ERROR("Failed to load configuration");
        return 1;
    }
    
    // Set defaults for ingestor
    if (config->ingestor.worker_threads == 0) {
        config->ingestor.worker_threads = 4;
    }
    if (config->ingestor.reconnect_delay_ms == 0) {
        config->ingestor.reconnect_delay_ms = 5000;
    }
    if (config->ingestor.max_reconnect_attempts == 0) {
        config->ingestor.max_reconnect_attempts = 10;
    }
    
    // TODO: Load chain configurations from database
    // For now, hardcode Ethereum mainnet
    config->num_chains = 1;
    config->chains = calloc(1, sizeof(struct chain_config));
    config->chains[0].chain_id = 1;
    config->chains[0].name = strdup("ethereum");
    config->chains[0].ws_url = strdup("wss://eth-mainnet.g.alchemy.com/v2/YOUR_KEY");
    
    LOG_INFO("Configuration loaded: %zu chains, %d worker threads", 
             config->num_chains, config->ingestor.worker_threads);
    
    // Setup signal handlers
    signal(SIGINT, signal_handler);
    signal(SIGTERM, signal_handler);
    
    // Create worker threads (one per chain)
    worker_thread_t *workers = calloc(config->num_chains, sizeof(worker_thread_t));
    if (!workers) {
        LOG_ERROR("Failed to allocate worker threads");
        eth_config_free(config);
        return 1;
    }
    
    // Start worker threads
    for (size_t i = 0; i < config->num_chains; i++) {
        workers[i].config = config;
        workers[i].running = true;
        
        // Initialize WebSocket connection
        ws_connection_t *conn = calloc(1, sizeof(ws_connection_t));
        if (!conn) {
            LOG_ERROR("Failed to allocate connection for chain %lu", 
                     config->chains[i].chain_id);
            continue;
        }
        
        err = ws_connection_init(conn, config->chains[i].chain_id,
                                 config->chains[i].ws_url,
                                 config->redis_host, config->redis_port);
        if (err != ETH_OK) {
            LOG_ERROR("Failed to initialize connection for chain %lu", 
                     config->chains[i].chain_id);
            free(conn);
            continue;
        }
        
        workers[i].conn = conn;
        
        if (pthread_create(&workers[i].thread, NULL, ingestor_worker_thread, &workers[i]) != 0) {
            LOG_ERROR("Failed to create worker thread for chain %lu", 
                     config->chains[i].chain_id);
            ws_connection_cleanup(conn);
            free(conn);
            continue;
        }
        
        LOG_INFO("Started worker thread for chain %lu (%s)", 
                 config->chains[i].chain_id, config->chains[i].name);
    }
    
    // Main loop - just wait for signal
    while (g_running) {
        sleep(1);
        
        // TODO: Collect and log metrics
        for (size_t i = 0; i < config->num_chains; i++) {
            if (workers[i].conn) {
                uint64_t received = atomic_load(&workers[i].conn->events_received);
                uint64_t published = atomic_load(&workers[i].conn->events_published);
                uint64_t errors = atomic_load(&workers[i].conn->errors);
                
                if (received > 0 || published > 0 || errors > 0) {
                    LOG_INFO("Chain %lu stats: received=%lu, published=%lu, errors=%lu",
                            config->chains[i].chain_id, received, published, errors);
                }
            }
        }
    }
    
    LOG_INFO("Shutting down gracefully...");
    
    // Stop all workers
    for (size_t i = 0; i < config->num_chains; i++) {
        workers[i].running = false;
        if (workers[i].conn) {
            ws_connection_stop(workers[i].conn);
        }
    }
    
    // Wait for all threads to finish
    for (size_t i = 0; i < config->num_chains; i++) {
        if (workers[i].thread) {
            pthread_join(workers[i].thread, NULL);
        }
        if (workers[i].conn) {
            ws_connection_cleanup(workers[i].conn);
            free(workers[i].conn);
        }
    }
    
    free(workers);
    eth_config_free(config);
    
    LOG_INFO("Shutdown complete");
    return 0;
}
