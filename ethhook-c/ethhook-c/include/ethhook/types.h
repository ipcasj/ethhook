/**
 * @file types.h
 * @brief Common type definitions for ETHhook
 */

#ifndef ETHHOOK_TYPES_H
#define ETHHOOK_TYPES_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/** Maximum Ethereum address length (0x + 40 hex chars + null) */
#define ETH_ADDRESS_LEN 43

/** Maximum Ethereum transaction hash length (0x + 64 hex chars + null) */
#define ETH_HASH_LEN 67

/** Maximum event topic length */
#define ETH_TOPIC_LEN 67

/** UUID string length (36 chars + null) */
#define UUID_STR_LEN 37

/** HMAC secret length (bytes) */
#define HMAC_SECRET_LEN 64

/** Ethereum blockchain event */
typedef struct {
    uint64_t block_number;
    uint64_t chain_id;
    char block_hash[ETH_HASH_LEN];
    char transaction_hash[ETH_HASH_LEN];
    uint32_t log_index;
    char contract_address[ETH_ADDRESS_LEN];
    char **topics;         /**< Array of event topics */
    size_t num_topics;
    char *data;            /**< Hex-encoded event data */
    uint64_t timestamp;    /**< Unix timestamp */
} eth_event_t;

/** Webhook endpoint configuration */
typedef struct {
    char id[UUID_STR_LEN];
    char application_id[UUID_STR_LEN];
    char *name;
    char *url;
    char hmac_secret[HMAC_SECRET_LEN];
    char **contract_addresses;
    size_t num_addresses;
    char **event_signatures;
    size_t num_signatures;
    uint64_t *chain_ids;
    size_t num_chains;
    bool is_active;
    uint32_t rate_limit_per_sec;
    uint32_t max_retries;
    uint32_t timeout_seconds;
} endpoint_t;

/** Result type for error handling */
typedef enum {
    RESULT_OK = 0,
    RESULT_ERROR = -1,
    RESULT_ERROR_NOMEM = -2,
    RESULT_ERROR_INVALID = -3,
    RESULT_ERROR_NOTFOUND = -4,
    RESULT_ERROR_TIMEOUT = -5,
    RESULT_ERROR_NETWORK = -6,
} result_t;

#ifdef __cplusplus
}
#endif

#endif /* ETHHOOK_TYPES_H */
