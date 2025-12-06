#define _GNU_SOURCE
#include "ethhook/common.h"
#include <sqlite3.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <libgen.h>
#include <errno.h>
#include <unistd.h>

struct eth_db {
    sqlite3 *handle;
    char *path;
};

// Parse SQLite URL and extract file path
// Handles: sqlite:///path, sqlite://path, /path
static char* parse_sqlite_url(const char *url) {
    if (!url) return NULL;
    
    // Remove sqlite:// or sqlite: prefix
    if (strncmp(url, "sqlite:///", 10) == 0) {
        return strdup(url + 9);  // Keep the leading /
    } else if (strncmp(url, "sqlite://", 9) == 0) {
        return strdup(url + 9);
    } else if (strncmp(url, "sqlite:", 7) == 0) {
        return strdup(url + 7);
    } else {
        // Already a file path
        return strdup(url);
    }
}

// Ensure parent directory exists with proper permissions
static eth_error_t ensure_parent_directory(const char *filepath) {
    char *path_copy = strdup(filepath);
    if (!path_copy) {
        return ETH_ERROR_MEMORY;
    }
    
    char *dir = dirname(path_copy);
    
    // Check if directory exists
    struct stat st;
    if (stat(dir, &st) == 0) {
        // Directory exists, check if writable
        if (access(dir, W_OK) != 0) {
            LOG_ERROR("Directory %s exists but is not writable: %s", dir, strerror(errno));
            free(path_copy);
            return ETH_ERROR_DATABASE;
        }
        free(path_copy);
        return ETH_OK;
    }
    
    // Directory doesn't exist, try to create it
    LOG_INFO("Creating database directory: %s", dir);
    if (mkdir(dir, 0755) != 0) {
        if (errno != EEXIST) {  // Ignore if created by another thread
            LOG_ERROR("Failed to create directory %s: %s", dir, strerror(errno));
            free(path_copy);
            return ETH_ERROR_DATABASE;
        }
    }
    
    // Verify directory is now writable
    if (access(dir, W_OK) != 0) {
        LOG_ERROR("Created directory %s but it's not writable: %s", dir, strerror(errno));
        free(path_copy);
        return ETH_ERROR_DATABASE;
    }
    
    LOG_INFO("Database directory created successfully: %s", dir);
    free(path_copy);
    return ETH_OK;
}

// Initialize database schema
static eth_error_t init_database_schema(sqlite3 *handle) {
    const char *schema = 
        "CREATE TABLE IF NOT EXISTS users ("
        "  id TEXT PRIMARY KEY,"
        "  username TEXT UNIQUE NOT NULL,"
        "  password_hash TEXT NOT NULL,"
        "  is_admin INTEGER DEFAULT 0,"
        "  created_at INTEGER NOT NULL"
        ");"
        "CREATE TABLE IF NOT EXISTS api_keys ("
        "  id TEXT PRIMARY KEY,"
        "  user_id TEXT NOT NULL,"
        "  key_hash TEXT NOT NULL,"
        "  name TEXT,"
        "  created_at INTEGER NOT NULL,"
        "  last_used_at INTEGER,"
        "  FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE"
        ");"
        "CREATE INDEX IF NOT EXISTS idx_api_keys_user_id ON api_keys(user_id);"
        "CREATE INDEX IF NOT EXISTS idx_api_keys_hash ON api_keys(key_hash);";
    
    char *err_msg = NULL;
    int rc = sqlite3_exec(handle, schema, NULL, NULL, &err_msg);
    if (rc != SQLITE_OK) {
        LOG_ERROR("Failed to initialize database schema: %s", err_msg);
        sqlite3_free(err_msg);
        return ETH_ERROR_DATABASE;
    }
    
    LOG_INFO("Database schema initialized successfully");
    return ETH_OK;
}

eth_error_t eth_db_open(const char *url, eth_db_t **db) {
    if (!url || !db) {
        LOG_ERROR("Invalid arguments to eth_db_open");
        return ETH_ERROR_INVALID_PARAM;
    }
    
    LOG_INFO("Opening database: %s", url);
    
    // Parse SQLite URL to get actual file path
    char *filepath = parse_sqlite_url(url);
    if (!filepath) {
        LOG_ERROR("Failed to parse database URL: %s", url);
        return ETH_ERROR_CONFIG;
    }
    
    LOG_INFO("Parsed database path: %s", filepath);
    
    // Ensure parent directory exists with proper permissions
    eth_error_t err = ensure_parent_directory(filepath);
    if (err != ETH_OK) {
        LOG_ERROR("Failed to ensure parent directory exists");
        free(filepath);
        return err;
    }
    
    eth_db_t *database = calloc(1, sizeof(eth_db_t));
    if (!database) {
        free(filepath);
        return ETH_ERROR_MEMORY;
    }
    
    database->path = filepath;  // Take ownership
    
    // Check if database file exists (for logging)
    struct stat st;
    bool is_new_db = (stat(filepath, &st) != 0);
    if (is_new_db) {
        LOG_INFO("Database file does not exist, will be created: %s", filepath);
    } else {
        LOG_INFO("Opening existing database: %s (size: %ld bytes)", filepath, st.st_size);
    }
    
    // Open database with error checking
    int rc = sqlite3_open(filepath, &database->handle);
    if (rc != SQLITE_OK) {
        const char *errmsg = database->handle ? sqlite3_errmsg(database->handle) : "unknown error";
        LOG_ERROR("Failed to open database %s: %s (code: %d)", filepath, errmsg, rc);
        
        // Additional diagnostics
        struct stat dir_st;
        char *dir_copy = strdup(filepath);
        char *dir_path = dirname(dir_copy);
        if (stat(dir_path, &dir_st) == 0) {
            LOG_ERROR("Directory %s exists with mode %o, uid=%d, gid=%d", 
                     dir_path, dir_st.st_mode, dir_st.st_uid, dir_st.st_gid);
            LOG_ERROR("Process running as uid=%d, gid=%d", getuid(), getgid());
        } else {
            LOG_ERROR("Directory %s does not exist: %s", dir_path, strerror(errno));
        }
        free(dir_copy);
        
        if (database->handle) {
            sqlite3_close(database->handle);
        }
        free(database->path);
        free(database);
        return ETH_ERROR_DATABASE;
    }
    
    LOG_INFO("Database opened successfully: %s", filepath);
    
    // Initialize schema if this is a new database
    if (is_new_db) {
        LOG_INFO("Initializing database schema for new database");
        err = init_database_schema(database->handle);
        if (err != ETH_OK) {
            LOG_ERROR("Failed to initialize database schema");
            sqlite3_close(database->handle);
            free(database->path);
            free(database);
            return err;
        }
    }
    
    // Enable WAL mode for better concurrency
    char *err_msg = NULL;
    rc = sqlite3_exec(database->handle, "PRAGMA journal_mode=WAL", NULL, NULL, &err_msg);
    if (rc != SQLITE_OK) {
        LOG_WARN("Failed to enable WAL mode: %s", err_msg);
        sqlite3_free(err_msg);
    } else {
        LOG_INFO("WAL mode enabled for database");
    }
    
    // Set busy timeout (5 seconds)
    sqlite3_busy_timeout(database->handle, 5000);
    
    // Additional pragmas for performance and reliability
    sqlite3_exec(database->handle, "PRAGMA synchronous=NORMAL", NULL, NULL, NULL);
    sqlite3_exec(database->handle, "PRAGMA temp_store=MEMORY", NULL, NULL, NULL);
    sqlite3_exec(database->handle, "PRAGMA foreign_keys=ON", NULL, NULL, NULL);
    
    LOG_INFO("Database initialization complete: %s", filepath);
    
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
