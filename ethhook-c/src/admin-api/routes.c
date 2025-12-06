#include "ethhook/admin_api.h"
#include "ethhook/common.h"
#include "ethhook/clickhouse.h"
#include <stdlib.h>
#include <string.h>

struct admin_api_ctx {
    eth_config_t *config;
    eth_db_t *db;
    clickhouse_client_t *ch_client;
    struct MHD_Daemon *daemon;
};

// Add CORS headers to response
void add_cors_headers(struct MHD_Response *response) {
    MHD_add_response_header(response, "Access-Control-Allow-Origin", "*");
    MHD_add_response_header(response, "Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS");
    MHD_add_response_header(response, "Access-Control-Allow-Headers", "Content-Type, Authorization");
    MHD_add_response_header(response, "Access-Control-Max-Age", "86400");
}

// Request router
static enum MHD_Result route_request(void *cls, struct MHD_Connection *connection,
                        const char *url, const char *method,
                        const char *version, const char *upload_data,
                        size_t *upload_data_size, void **con_cls) {
    (void)version;
    
    admin_api_ctx_t *ctx = (admin_api_ctx_t *)cls;
    request_ctx_t *req_ctx = NULL;
    
    // Initialize request context
    if (*con_cls == NULL) {
        req_ctx = calloc(1, sizeof(request_ctx_t));
        if (!req_ctx) {
            return MHD_NO;
        }
        
        req_ctx->db = ctx->db;
        req_ctx->ch_client = ctx->ch_client;
        req_ctx->jwt_secret = ctx->config->admin_api.jwt_secret;
        req_ctx->user_id = NULL;
        req_ctx->is_admin = false;
        
        // Check for Authorization header
        const char *auth_header = MHD_lookup_connection_value(connection, MHD_HEADER_KIND, "Authorization");
        if (auth_header && strncmp(auth_header, "Bearer ", 7) == 0) {
            const char *token = auth_header + 7;
            
            char *user_id = NULL;
            bool is_admin = false;
            
            if (jwt_verify(token, req_ctx->jwt_secret, &user_id, &is_admin) == ETH_OK) {
                req_ctx->user_id = user_id;
                req_ctx->is_admin = is_admin;
            }
        }
        
        *con_cls = req_ctx;
        return MHD_YES;
    }
    
    req_ctx = (request_ctx_t *)*con_cls;
    
    // Handle OPTIONS preflight requests (CORS)
    if (strcmp(method, "OPTIONS") == 0) {
        struct MHD_Response *response = MHD_create_response_from_buffer(0, NULL, MHD_RESPMEM_PERSISTENT);
        add_cors_headers(response);
        int ret = MHD_queue_response(connection, MHD_HTTP_NO_CONTENT, response);
        MHD_destroy_response(response);
        return ret;
    }
    
    // Route to appropriate handler
    if (strcmp(url, "/health") == 0) {
        // Health check endpoint - returns 200 OK with simple JSON response
        const char *health_response = "{\"status\":\"ok\"}";
        struct MHD_Response *response = MHD_create_response_from_buffer(
            strlen(health_response), (void *)health_response, MHD_RESPMEM_MUST_COPY);
        MHD_add_response_header(response, "Content-Type", "application/json");
        add_cors_headers(response);
        int ret = MHD_queue_response(connection, MHD_HTTP_OK, response);
        MHD_destroy_response(response);
        return ret;
    } else if (strcmp(url, "/api/auth/login") == 0) {
        return handle_login(connection, req_ctx, method, upload_data, upload_data_size);
    } else if (strcmp(url, "/api/users") == 0) {
        return handle_users(connection, req_ctx, method, upload_data, upload_data_size);
    } else if (strncmp(url, "/api/applications", 17) == 0) {
        return handle_applications(connection, req_ctx, method, upload_data, upload_data_size);
    } else if (strncmp(url, "/api/endpoints", 14) == 0) {
        return handle_endpoints(connection, req_ctx, method, upload_data, upload_data_size);
    } else if (strncmp(url, "/api/events", 11) == 0) {
        return handle_events(connection, req_ctx, method, upload_data, upload_data_size);
    } else if (strncmp(url, "/api/deliveries", 15) == 0) {
        return handle_deliveries(connection, req_ctx, method, upload_data, upload_data_size);
    }
    
    // 404 Not Found
    response_t *resp = response_error(MHD_HTTP_NOT_FOUND, "Not found");
    struct MHD_Response *response = MHD_create_response_from_buffer(
        resp->body_len, resp->body, MHD_RESPMEM_MUST_COPY);
    MHD_add_response_header(response, "Content-Type", "application/json");
    add_cors_headers(response);
    int ret = MHD_queue_response(connection, resp->status_code, response);
    MHD_destroy_response(response);
    response_free(resp);
    
    return ret;
}

// Cleanup request context
static void request_completed(void *cls, struct MHD_Connection *connection,
                              void **con_cls, enum MHD_RequestTerminationCode toe) {
    (void)cls;
    (void)connection;
    (void)toe;
    
    request_ctx_t *req_ctx = (request_ctx_t *)*con_cls;
    if (req_ctx) {
        free(req_ctx->user_id);
        free(req_ctx);
    }
}

eth_error_t admin_api_ctx_create(eth_config_t *config, admin_api_ctx_t **ctx) {
    admin_api_ctx_t *api_ctx = calloc(1, sizeof(admin_api_ctx_t));
    if (!api_ctx) {
        return ETH_ERROR_MEMORY;
    }
    
    api_ctx->config = config;
    
    // Open database
    eth_error_t err = eth_db_open(config->database_url, &api_ctx->db);
    if (err != ETH_OK) {
        free(api_ctx);
        return err;
    }
    
    // Initialize ClickHouse client (optional - only if URL configured)
    if (config->clickhouse_url) {
        LOG_INFO("Initializing ClickHouse client: %s", config->clickhouse_url);
        clickhouse_config_t ch_config = {
            .url = config->clickhouse_url,
            .database = config->clickhouse_database ? config->clickhouse_database : "ethhook",
            .user = config->clickhouse_user,
            .password = config->clickhouse_password,
            .pool_size = 10,
            .timeout_ms = 30000,
            .enable_compression = true,
            .batch_size = 1000,
            .batch_timeout_ms = 1000
        };
        err = clickhouse_client_create(&ch_config, &api_ctx->ch_client);
        if (err != ETH_OK) {
            LOG_WARN("Failed to create ClickHouse client (non-fatal, continuing without analytics)");
            api_ctx->ch_client = NULL;
        } else {
            LOG_INFO("ClickHouse client initialized successfully");
        }
    } else {
        LOG_INFO("ClickHouse URL not configured, skipping analytics initialization");
        api_ctx->ch_client = NULL;
    }
    
    LOG_INFO("ClickHouse client initialized for admin API");
    
    *ctx = api_ctx;
    return ETH_OK;
}

void admin_api_ctx_destroy(admin_api_ctx_t *ctx) {
    if (!ctx) {
        return;
    }
    
    if (ctx->daemon) {
        MHD_stop_daemon(ctx->daemon);
    }
    
    if (ctx->ch_client) {
        clickhouse_client_destroy(ctx->ch_client);
    }
    
    eth_db_close(ctx->db);
    free(ctx);
}

eth_error_t admin_api_run(admin_api_ctx_t *ctx) {
    if (!ctx) {
        return ETH_ERROR_INVALID_PARAM;
    }
    
    ctx->daemon = MHD_start_daemon(
        MHD_USE_THREAD_PER_CONNECTION,
        ctx->config->admin_api.port,
        NULL, NULL,
        &route_request, ctx,
        MHD_OPTION_NOTIFY_COMPLETED, request_completed, NULL,
        MHD_OPTION_END
    );
    
    if (!ctx->daemon) {
        LOG_ERROR("Failed to start HTTP daemon");
        return ETH_ERROR_HTTP;
    }
    
    LOG_INFO("Admin API listening on port %d", ctx->config->admin_api.port);
    return ETH_OK;
}

void admin_api_stop(admin_api_ctx_t *ctx) {
    if (ctx && ctx->daemon) {
        MHD_stop_daemon(ctx->daemon);
        ctx->daemon = NULL;
    }
}
