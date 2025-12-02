#define _POSIX_C_SOURCE 200112L
#include "ethhook/ingestor.h"
#include <unistd.h>

void *ingestor_worker_thread(void *arg) {
    worker_thread_t *worker = (worker_thread_t *)arg;
    
    LOG_INFO("Worker thread started for chain %lu", worker->conn->chain_id);
    
    while (worker->running) {
        eth_error_t err = ws_connection_start(worker->conn);
        
        if (err != ETH_OK) {
            LOG_ERROR("WebSocket connection failed for chain %lu: %d",
                    worker->conn->chain_id, err);
            
            // Exponential backoff
            int delay = worker->config->ingestor.reconnect_delay_ms;
            LOG_INFO("Reconnecting in %d ms...", delay);
            usleep(delay * 1000);
            
            // Increase delay for next attempt
            if (delay < 60000) { // Max 60 seconds
                delay *= 2;
            }
        }
    }
    
    LOG_INFO("Worker thread stopped for chain %lu", worker->conn->chain_id);
    return NULL;
}
