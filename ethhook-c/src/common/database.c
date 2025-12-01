#include "ethhook/common.h"
#include <sqlite3.h>
#include <stdlib.h>
#include <string.h>

struct eth_db {
    sqlite3 *handle;
    char *path;
};

eth_error_t eth_db_open(const char *path, eth_db_t **db) {
    eth_db_t *database = calloc(1, sizeof(eth_db_t));
    if (!database) {
        return ETH_ERROR_MEMORY;
    }
    
    database->path = strdup(path);
    
    int rc = sqlite3_open(path, &database->handle);
    if (rc != SQLITE_OK) {
        LOG_ERROR("Failed to open database %s: %s", path, sqlite3_errmsg(database->handle));
        free(database->path);
        free(database);
        return ETH_ERROR_DATABASE;
    }
    
    // Enable WAL mode for better concurrency
    char *err_msg = NULL;
    rc = sqlite3_exec(database->handle, "PRAGMA journal_mode=WAL", NULL, NULL, &err_msg);
    if (rc != SQLITE_OK) {
        LOG_WARN("Failed to enable WAL mode: %s", err_msg);
        sqlite3_free(err_msg);
    }
    
    // Set busy timeout
    sqlite3_busy_timeout(database->handle, 5000);
    
    *db = database;
    return ETH_OK;
}

void eth_db_close(eth_db_t *db) {
    if (!db) {
        return;
    }
    
    if (db->handle) {
        sqlite3_close(db->handle);
    }
    
    free(db->path);
    free(db);
}

// Helper function to get SQLite handle (for internal use)
sqlite3 *eth_db_get_handle(eth_db_t *db) {
    return db ? db->handle : NULL;
}
