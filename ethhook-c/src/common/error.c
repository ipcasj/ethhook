#include "ethhook/common.h"
#include <string.h>

const char *eth_error_string(eth_error_t code) {
    switch (code) {
        case ETH_OK: return "Success";
        case ETH_ERROR: return "General error";
        case ETH_ERROR_MEMORY: return "Memory allocation failed";
        case ETH_ERROR_CONFIG: return "Configuration error";
        case ETH_ERROR_DATABASE: return "Database error";
        case ETH_ERROR_NETWORK: return "Network error";
        case ETH_ERROR_WEBSOCKET: return "WebSocket error";
        case ETH_ERROR_REDIS: return "Redis error";
        case ETH_ERROR_HTTP: return "HTTP error";
        case ETH_ERROR_JSON: return "JSON parsing error";
        case ETH_ERROR_AUTH: return "Authentication error";
        case ETH_ERROR_INVALID_PARAM: return "Invalid parameter";
        case ETH_ERROR_TIMEOUT: return "Timeout";
        case ETH_ERROR_CIRCUIT_OPEN: return "Circuit breaker open";
        default: return "Unknown error";
    }
}
