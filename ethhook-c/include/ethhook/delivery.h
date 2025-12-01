#ifndef ETHHOOK_DELIVERY_H
#define ETHHOOK_DELIVERY_H

#include "common.h"
#include "processor.h"

// Delivery request
typedef struct {
    char delivery_id[37];
    char event_id[37];
    endpoint_t endpoint;
    event_t event;
    uint32_t attempt;
    uint64_t scheduled_at;
} delivery_request_t;

// HTTP client
typedef struct {
    void *curl_handle;
    circuit_breaker_t *circuit_breaker;
    char *user_agent;
} http_client_t;

eth_error_t http_client_init(http_client_t *client);
void http_client_cleanup(http_client_t *client);

eth_error_t http_client_post(http_client_t *client, const char *url,
                              const char *payload, size_t payload_len,
                              const char *signature, int timeout_ms,
                              int *http_status);

// Retry logic
typedef struct {
    uint32_t max_retries;
    uint32_t base_delay_ms;
    uint32_t max_delay_ms;
    double backoff_multiplier;
} retry_policy_t;

uint64_t retry_calculate_delay(retry_policy_t *policy, uint32_t attempt);

// Delivery worker
typedef struct delivery_ctx delivery_ctx_t;

eth_error_t delivery_ctx_create(eth_config_t *config, delivery_ctx_t **ctx);
void delivery_ctx_destroy(delivery_ctx_t *ctx);
eth_error_t delivery_run(delivery_ctx_t *ctx);
void delivery_stop(delivery_ctx_t *ctx);

#endif // ETHHOOK_DELIVERY_H
