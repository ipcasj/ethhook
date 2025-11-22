/**
 * @file config.c
 * @brief Configuration loader from environment variables
 * Single Translation Unit
 */

#include <stdlib.h>
#include <string.h>
#include <stdio.h>

/* Configuration getters with defaults */
const char *config_get_env(const char *key, const char *default_value) {
    const char *value = getenv(key);
    return value ? value : default_value;
}

int config_get_env_int(const char *key, int default_value) {
    const char *value = getenv(key);
    return value ? atoi(value) : default_value;
}

uint64_t config_get_env_uint64(const char *key, uint64_t default_value) {
    const char *value = getenv(key);
    return value ? (uint64_t)strtoull(value, NULL, 10) : default_value;
}

/* Load required configuration or exit */
const char *config_require(const char *key) {
    const char *value = getenv(key);
    if (!value) {
        fprintf(stderr, "FATAL: Required environment variable %s not set\n", key);
        exit(1);
    }
    return value;
}
