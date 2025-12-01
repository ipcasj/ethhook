#include "ethhook/common.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <jansson.h>

// Simple TOML parser (basic implementation for key=value format)
// In production, use tomlc99 library

eth_error_t eth_config_load(const char *config_file, eth_config_t **config) {
    FILE *fp = fopen(config_file, "r");
    if (!fp) {
        LOG_ERROR("Failed to open config file: %s", config_file);
        return ETH_ERROR_CONFIG;
    }
    
    eth_config_t *cfg = calloc(1, sizeof(eth_config_t));
    if (!cfg) {
        fclose(fp);
        return ETH_ERROR_MEMORY;
    }
    
    // Default values
    cfg->redis_port = 6379;
    
    char line[1024];
    while (fgets(line, sizeof(line), fp)) {
        // Skip comments and empty lines
        if (line[0] == '#' || line[0] == '\n') {
            continue;
        }
        
        // Simple key=value parsing
        char *equals = strchr(line, '=');
        if (!equals) {
            continue;
        }
        
        *equals = '\0';
        char *key = line;
        char *value = equals + 1;
        
        // Trim whitespace
        while (*key == ' ' || *key == '\t') key++;
        while (*value == ' ' || *value == '\t') value++;
        char *end = value + strlen(value) - 1;
        while (end > value && (*end == '\n' || *end == '\r' || *end == ' ')) {
            *end = '\0';
            end--;
        }
        
        // Remove quotes
        if (*value == '"' && *end == '"') {
            value++;
            *end = '\0';
        }
        
        // Parse known keys
        if (strcmp(key, "database_url") == 0) {
            cfg->database_url = strdup(value);
        } else if (strcmp(key, "redis_host") == 0) {
            cfg->redis_host = strdup(value);
        } else if (strcmp(key, "redis_port") == 0) {
            cfg->redis_port = atoi(value);
        } else if (strcmp(key, "redis_password") == 0) {
            cfg->redis_password = strdup(value);
        }
    }
    
    fclose(fp);
    
    // Override with environment variables
    const char *env_db = getenv("DATABASE_URL");
    if (env_db) {
        free(cfg->database_url);
        cfg->database_url = strdup(env_db);
    }
    
    const char *env_redis_host = getenv("REDIS_HOST");
    if (env_redis_host) {
        free(cfg->redis_host);
        cfg->redis_host = strdup(env_redis_host);
    }
    
    const char *env_redis_port = getenv("REDIS_PORT");
    if (env_redis_port) {
        cfg->redis_port = atoi(env_redis_port);
    }
    
    // Validate required fields
    if (!cfg->database_url) {
        LOG_ERROR("Missing required config: database_url");
        eth_config_free(cfg);
        return ETH_ERROR_CONFIG;
    }
    
    if (!cfg->redis_host) {
        cfg->redis_host = strdup("localhost");
    }
    
    *config = cfg;
    return ETH_OK;
}

void eth_config_free(eth_config_t *config) {
    if (!config) {
        return;
    }
    
    free(config->database_url);
    free(config->redis_host);
    free(config->redis_password);
    
    if (config->chains) {
        for (size_t i = 0; i < config->num_chains; i++) {
            free(config->chains[i].name);
            free(config->chains[i].ws_url);
            free(config->chains[i].http_url);
        }
        free(config->chains);
    }
    
    free(config);
}
