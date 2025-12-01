#ifndef ETHHOOK_PROCESSOR_H
#define ETHHOOK_PROCESSOR_H

#include "common.h"

// Endpoint matching
typedef struct {
    char endpoint_id[37];
    char application_id[37];
    uint64_t chain_id;
    char *address;
    char **topics; // Array of topic filters
    size_t num_topics;
    char *webhook_url;
    char *webhook_secret;
} endpoint_t;

// Event to process
typedef struct {
    char event_id[37];
    uint64_t chain_id;
    uint64_t block_number;
    char block_hash[67];
    char transaction_hash[67];
    uint32_t log_index;
    char contract_address[43];
    char **topics;
    size_t num_topics;
    char *data;
    uint64_t ingested_at;
} event_t;

// Matcher functions
eth_error_t matcher_init(eth_db_t *db);
void matcher_cleanup(void);

eth_error_t matcher_find_endpoints(event_t *event, endpoint_t ***endpoints, size_t *count);
void matcher_free_endpoints(endpoint_t **endpoints, size_t count);

// Filter functions
bool filter_matches(endpoint_t *endpoint, event_t *event);

// Redis consumer
typedef struct processor_ctx processor_ctx_t;

eth_error_t processor_ctx_create(eth_config_t *config, processor_ctx_t **ctx);
void processor_ctx_destroy(processor_ctx_t *ctx);
eth_error_t processor_run(processor_ctx_t *ctx);
void processor_stop(processor_ctx_t *ctx);

#endif // ETHHOOK_PROCESSOR_H
