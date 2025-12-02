#define _GNU_SOURCE
#include "ethhook/processor.h"
#include <sqlite3.h>
#include <stdlib.h>
#include <string.h>

static eth_db_t *g_db = NULL;

// Helper to get SQLite handle
extern sqlite3 *eth_db_get_handle(eth_db_t *db);

eth_error_t matcher_init(eth_db_t *db) {
    g_db = db;
    return ETH_OK;
}

void matcher_cleanup(void) {
    g_db = NULL;
}

eth_error_t matcher_find_endpoints(event_t *event, endpoint_t ***endpoints, size_t *count) {
    if (!g_db || !event || !endpoints || !count) {
        return ETH_ERROR_INVALID_PARAM;
    }
    
    *endpoints = NULL;
    *count = 0;
    
    sqlite3 *db = eth_db_get_handle(g_db);
    if (!db) {
        return ETH_ERROR_DATABASE;
    }
    
    // Query for matching endpoints
    const char *sql = 
        "SELECT e.id, e.application_id, e.chain_id, e.address, e.topics, "
        "       a.webhook_url, a.webhook_secret "
        "FROM endpoints e "
        "JOIN applications a ON e.application_id = a.id "
        "WHERE e.chain_id = ? AND e.enabled = 1";
    
    sqlite3_stmt *stmt = NULL;
    int rc = sqlite3_prepare_v2(db, sql, -1, &stmt, NULL);
    if (rc != SQLITE_OK) {
        LOG_ERROR("Failed to prepare statement: %s", sqlite3_errmsg(db));
        return ETH_ERROR_DATABASE;
    }
    
    sqlite3_bind_int64(stmt, 1, event->chain_id);
    
    // Temporary array to collect matching endpoints
    endpoint_t **results = NULL;
    size_t capacity = 0;
    size_t num_results = 0;
    
    while ((rc = sqlite3_step(stmt)) == SQLITE_ROW) {
        // Parse endpoint
        endpoint_t *ep = calloc(1, sizeof(endpoint_t));
        if (!ep) {
            continue;
        }
        
        // Copy ID
        const char *id = (const char *)sqlite3_column_text(stmt, 0);
        if (id) {
            strncpy(ep->endpoint_id, id, sizeof(ep->endpoint_id) - 1);
        }
        
        // Copy application ID
        const char *app_id = (const char *)sqlite3_column_text(stmt, 1);
        if (app_id) {
            strncpy(ep->application_id, app_id, sizeof(ep->application_id) - 1);
        }
        
        ep->chain_id = sqlite3_column_int64(stmt, 2);
        
        // Copy address
        const char *address = (const char *)sqlite3_column_text(stmt, 3);
        if (address) {
            ep->address = strdup(address);
        }
        
        // Parse topics JSON array
        const char *topics_json = (const char *)sqlite3_column_text(stmt, 4);
        if (topics_json) {
            // TODO: Parse JSON array of topics
            // For now, simple implementation
            ep->topics = NULL;
            ep->num_topics = 0;
        }
        
        // Copy webhook URL
        const char *webhook_url = (const char *)sqlite3_column_text(stmt, 5);
        if (webhook_url) {
            ep->webhook_url = strdup(webhook_url);
        }
        
        // Copy webhook secret
        const char *webhook_secret = (const char *)sqlite3_column_text(stmt, 6);
        if (webhook_secret) {
            ep->webhook_secret = strdup(webhook_secret);
        }
        
        // Check if this endpoint matches the event
        if (filter_matches(ep, event)) {
            // Add to results
            if (num_results >= capacity) {
                capacity = capacity == 0 ? 8 : capacity * 2;
                endpoint_t **new_results = realloc(results, capacity * sizeof(endpoint_t *));
                if (!new_results) {
                    free(ep->address);
                    free(ep->webhook_url);
                    free(ep->webhook_secret);
                    free(ep);
                    break;
                }
                results = new_results;
            }
            results[num_results++] = ep;
        } else {
            // Free endpoint
            free(ep->address);
            free(ep->webhook_url);
            free(ep->webhook_secret);
            free(ep);
        }
    }
    
    sqlite3_finalize(stmt);
    
    if (rc != SQLITE_DONE && rc != SQLITE_ROW) {
        LOG_ERROR("Query execution failed: %s", sqlite3_errmsg(db));
        matcher_free_endpoints(results, num_results);
        return ETH_ERROR_DATABASE;
    }
    
    *endpoints = results;
    *count = num_results;
    
    return ETH_OK;
}

void matcher_free_endpoints(endpoint_t **endpoints, size_t count) {
    if (!endpoints) {
        return;
    }
    
    for (size_t i = 0; i < count; i++) {
        if (endpoints[i]) {
            free(endpoints[i]->address);
            free(endpoints[i]->webhook_url);
            free(endpoints[i]->webhook_secret);
            
            if (endpoints[i]->topics) {
                for (size_t j = 0; j < endpoints[i]->num_topics; j++) {
                    free(endpoints[i]->topics[j]);
                }
                free(endpoints[i]->topics);
            }
            
            free(endpoints[i]);
        }
    }
    
    free(endpoints);
}
