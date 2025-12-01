#ifndef ETHHOOK_COMMON_H
#define ETHHOOK_COMMON_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <stdatomic.h>

// Error codes
typedef enum {
    ETH_OK = 0,
    ETH_ERROR = -1,
    ETH_ERROR_MEMORY = -2,
    ETH_ERROR_CONFIG = -3,
    ETH_ERROR_DATABASE = -4,
    ETH_ERROR_NETWORK = -5,
    ETH_ERROR_WEBSOCKET = -6,
    ETH_ERROR_REDIS = -7,
    ETH_ERROR_HTTP = -8,
    ETH_ERROR_JSON = -9,
    ETH_ERROR_AUTH = -10,
    ETH_ERROR_INVALID_PARAM = -11,
    ETH_ERROR_TIMEOUT = -12,
    ETH_ERROR_CIRCUIT_OPEN = -13,
} eth_error_t;

// Error context
typedef struct {
    eth_error_t code;
    const char *message;
    const char *file;
    int line;
} eth_error_ctx_t;

#define ETH_ERROR_CTX(code, msg) \
    ((eth_error_ctx_t){.code = (code), .message = (msg), .file = __FILE__, .line = __LINE__})

// Logging levels
typedef enum {
    ETH_LOG_DEBUG = 0,
    ETH_LOG_INFO = 1,
    ETH_LOG_WARN = 2,
    ETH_LOG_ERROR = 3,
} eth_log_level_t;

// Logging functions
void eth_log_init(const char *ident);
void eth_log(eth_log_level_t level, const char *fmt, ...);

#define LOG_DEBUG(...) eth_log(ETH_LOG_DEBUG, __VA_ARGS__)
#define LOG_INFO(...) eth_log(ETH_LOG_INFO, __VA_ARGS__)
#define LOG_WARN(...) eth_log(ETH_LOG_WARN, __VA_ARGS__)
#define LOG_ERROR(...) eth_log(ETH_LOG_ERROR, __VA_ARGS__)

// Configuration
typedef struct {
    // Database
    char *database_url;
    
    // Redis
    char *redis_host;
    int redis_port;
    char *redis_password;
    
    // Chains
    struct chain_config {
        uint64_t chain_id;
        char *name;
        char *ws_url;
        char *http_url;
    } *chains;
    size_t num_chains;
    
    // Service-specific
    union {
        struct {
            int worker_threads;
            int reconnect_delay_ms;
            int max_reconnect_attempts;
        } ingestor;
        
        struct {
            int worker_threads;
            int batch_size;
        } processor;
        
        struct {
            int worker_threads;
            int max_retries;
            int timeout_ms;
        } delivery;
        
        struct {
            int port;
            char *jwt_secret;
            int jwt_expiry_hours;
        } admin_api;
    };
} eth_config_t;

// Config functions
eth_error_t eth_config_load(const char *config_file, eth_config_t **config);
void eth_config_free(eth_config_t *config);

// Arena allocator
typedef struct eth_arena eth_arena_t;

eth_arena_t *eth_arena_create(size_t initial_size);
void *eth_arena_alloc(eth_arena_t *arena, size_t size);
void eth_arena_reset(eth_arena_t *arena);
void eth_arena_destroy(eth_arena_t *arena);

// Circuit breaker
typedef enum {
    CB_STATE_CLOSED = 0,
    CB_STATE_OPEN = 1,
    CB_STATE_HALF_OPEN = 2,
} cb_state_t;

typedef struct {
    atomic_int state;
    atomic_uint_fast64_t failure_count;
    atomic_uint_fast64_t success_count;
    atomic_uint_fast64_t last_failure_time;
    uint32_t failure_threshold;
    uint32_t timeout_ms;
    uint32_t half_open_max_calls;
} circuit_breaker_t;

void circuit_breaker_init(circuit_breaker_t *cb, uint32_t failure_threshold, uint32_t timeout_ms);
bool circuit_breaker_allow(circuit_breaker_t *cb);
void circuit_breaker_success(circuit_breaker_t *cb);
void circuit_breaker_failure(circuit_breaker_t *cb);
cb_state_t circuit_breaker_state(circuit_breaker_t *cb);

// Database functions
typedef struct eth_db eth_db_t;

eth_error_t eth_db_open(const char *path, eth_db_t **db);
void eth_db_close(eth_db_t *db);

// UUID generation
void eth_uuid_generate(char *uuid_str); // 36 chars + null

// Time utilities
uint64_t eth_time_now_ms(void);
uint64_t eth_time_now_us(void);

#endif // ETHHOOK_COMMON_H
