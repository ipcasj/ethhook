#include "ethhook/admin_api.h"
#include "ethhook/clickhouse.h"
#include <sqlite3.h>
#include <jansson.h>
#include <string.h>
#include <stdlib.h>

// Helper to get SQLite handle
extern sqlite3 *eth_db_get_handle(eth_db_t *db);

int handle_login(struct MHD_Connection *connection, request_ctx_t *ctx,
                 const char *method, const char *upload_data, size_t *upload_data_size) {
    (void)ctx;
    
    if (strcmp(method, "POST") != 0) {
        response_t *resp = response_error(MHD_HTTP_METHOD_NOT_ALLOWED, "Method not allowed");
        struct MHD_Response *response = MHD_create_response_from_buffer(
            resp->body_len, resp->body, MHD_RESPMEM_MUST_COPY);
        int ret = MHD_queue_response(connection, resp->status_code, response);
        MHD_destroy_response(response);
        response_free(resp);
        return ret;
    }
    
    // Parse request body
    if (*upload_data_size > 0) {
        // TODO: Parse JSON body with email/password
        *upload_data_size = 0;
        return MHD_YES;
    }
    
    // TODO: Validate credentials and generate JWT token
    json_t *result = json_object();
    json_object_set_new(result, "token", json_string("dummy-jwt-token"));
    
    char *json_str = json_dumps(result, JSON_COMPACT);
    json_decref(result);
    
    struct MHD_Response *response = MHD_create_response_from_buffer(
        strlen(json_str), json_str, MHD_RESPMEM_MUST_FREE);
    MHD_add_response_header(response, "Content-Type", "application/json");
    int ret = MHD_queue_response(connection, MHD_HTTP_OK, response);
    MHD_destroy_response(response);
    
    return ret;
}

int handle_users(struct MHD_Connection *connection, request_ctx_t *ctx,
                 const char *method, const char *upload_data, size_t *upload_data_size) {
    (void)upload_data;
    (void)upload_data_size;
    
    if (!ctx->is_admin) {
        response_t *resp = response_error(MHD_HTTP_FORBIDDEN, "Admin access required");
        struct MHD_Response *response = MHD_create_response_from_buffer(
            resp->body_len, resp->body, MHD_RESPMEM_MUST_COPY);
        int ret = MHD_queue_response(connection, resp->status_code, response);
        MHD_destroy_response(response);
        response_free(resp);
        return ret;
    }
    
    if (strcmp(method, "GET") != 0) {
        response_t *resp = response_error(MHD_HTTP_METHOD_NOT_ALLOWED, "Method not allowed");
        struct MHD_Response *response = MHD_create_response_from_buffer(
            resp->body_len, resp->body, MHD_RESPMEM_MUST_COPY);
        int ret = MHD_queue_response(connection, resp->status_code, response);
        MHD_destroy_response(response);
        response_free(resp);
        return ret;
    }
    
    // Query users from database
    sqlite3 *db = eth_db_get_handle(ctx->db);
    const char *sql = "SELECT id, email, is_admin, created_at FROM users ORDER BY created_at DESC";
    
    sqlite3_stmt *stmt = NULL;
    int rc = sqlite3_prepare_v2(db, sql, -1, &stmt, NULL);
    if (rc != SQLITE_OK) {
        response_t *resp = response_error(MHD_HTTP_INTERNAL_SERVER_ERROR, "Database error");
        struct MHD_Response *response = MHD_create_response_from_buffer(
            resp->body_len, resp->body, MHD_RESPMEM_MUST_COPY);
        int ret = MHD_queue_response(connection, resp->status_code, response);
        MHD_destroy_response(response);
        response_free(resp);
        return ret;
    }
    
    json_t *users_array = json_array();
    
    while ((rc = sqlite3_step(stmt)) == SQLITE_ROW) {
        json_t *user = json_object();
        
        const char *id = (const char *)sqlite3_column_text(stmt, 0);
        const char *email = (const char *)sqlite3_column_text(stmt, 1);
        int is_admin = sqlite3_column_int(stmt, 2);
        const char *created_at = (const char *)sqlite3_column_text(stmt, 3);
        
        if (id) json_object_set_new(user, "id", json_string(id));
        if (email) json_object_set_new(user, "email", json_string(email));
        json_object_set_new(user, "is_admin", json_boolean(is_admin));
        if (created_at) json_object_set_new(user, "created_at", json_string(created_at));
        
        json_array_append_new(users_array, user);
    }
    
    sqlite3_finalize(stmt);
    
    json_t *result = json_object();
    json_object_set_new(result, "users", users_array);
    json_object_set_new(result, "total", json_integer(json_array_size(users_array)));
    
    char *json_str = json_dumps(result, JSON_COMPACT);
    json_decref(result);
    
    struct MHD_Response *response = MHD_create_response_from_buffer(
        strlen(json_str), json_str, MHD_RESPMEM_MUST_FREE);
    MHD_add_response_header(response, "Content-Type", "application/json");
    int ret = MHD_queue_response(connection, MHD_HTTP_OK, response);
    MHD_destroy_response(response);
    
    return ret;
}

int handle_applications(struct MHD_Connection *connection, request_ctx_t *ctx,
                        const char *method, const char *upload_data, size_t *upload_data_size) {
    (void)upload_data;
    (void)upload_data_size;
    
    if (strcmp(method, "GET") != 0) {
        response_t *resp = response_error(MHD_HTTP_METHOD_NOT_ALLOWED, "Method not allowed");
        struct MHD_Response *response = MHD_create_response_from_buffer(
            resp->body_len, resp->body, MHD_RESPMEM_MUST_COPY);
        int ret = MHD_queue_response(connection, resp->status_code, response);
        MHD_destroy_response(response);
        response_free(resp);
        return ret;
    }
    
    // Query applications
    json_t *apps_array = json_array();
    
    json_t *result = json_object();
    json_object_set_new(result, "applications", apps_array);
    
    char *json_str = json_dumps(result, JSON_COMPACT);
    json_decref(result);
    
    struct MHD_Response *response = MHD_create_response_from_buffer(
        strlen(json_str), json_str, MHD_RESPMEM_MUST_FREE);
    MHD_add_response_header(response, "Content-Type", "application/json");
    int ret = MHD_queue_response(connection, MHD_HTTP_OK, response);
    MHD_destroy_response(response);
    
    return ret;
}

int handle_endpoints(struct MHD_Connection *connection, request_ctx_t *ctx,
                     const char *method, const char *upload_data, size_t *upload_data_size) {
    (void)ctx;
    (void)upload_data;
    (void)upload_data_size;
    (void)method;
    
    json_t *result = json_object();
    json_object_set_new(result, "endpoints", json_array());
    
    char *json_str = json_dumps(result, JSON_COMPACT);
    json_decref(result);
    
    struct MHD_Response *response = MHD_create_response_from_buffer(
        strlen(json_str), json_str, MHD_RESPMEM_MUST_FREE);
    MHD_add_response_header(response, "Content-Type", "application/json");
    int ret = MHD_queue_response(connection, MHD_HTTP_OK, response);
    MHD_destroy_response(response);
    
    return ret;
}

int handle_events(struct MHD_Connection *connection, request_ctx_t *ctx,
                  const char *method, const char *upload_data, size_t *upload_data_size) {
    (void)upload_data;
    (void)upload_data_size;
    
    if (strcmp(method, "GET") != 0) {
        response_t *resp = response_error(MHD_HTTP_METHOD_NOT_ALLOWED, "Method not allowed");
        struct MHD_Response *response = MHD_create_response_from_buffer(
            resp->body_len, resp->body, MHD_RESPMEM_MUST_COPY);
        int ret = MHD_queue_response(connection, resp->status_code, response);
        MHD_destroy_response(response);
        response_free(resp);
        return ret;
    }
    
    // Parse query parameters
    const char *limit_str = MHD_lookup_connection_value(connection, MHD_GET_ARGUMENT_KIND, "limit");
    const char *offset_str = MHD_lookup_connection_value(connection, MHD_GET_ARGUMENT_KIND, "offset");
    
    int limit = limit_str ? atoi(limit_str) : 50;
    int offset = offset_str ? atoi(offset_str) : 0;
    
    if (limit <= 0 || limit > 1000) limit = 50;
    if (offset < 0) offset = 0;
    
    // Query ClickHouse for events
    char query[1024];
    snprintf(query, sizeof(query),
             "SELECT toString(id) as id, chain_id, block_number, toString(block_hash) as block_hash, "
             "toString(transaction_hash) as transaction_hash, log_index, contract_address, "
             "topics, data, ingested_at "
             "FROM events "
             "ORDER BY ingested_at DESC "
             "LIMIT %d OFFSET %d "
             "FORMAT JSONEachRow",
             limit, offset);
    
    char *response_body = NULL;
    size_t response_len = 0;
    
    eth_error_t err = clickhouse_query(ctx->ch_client, query, &response_body, &response_len);
    if (err != ETH_OK || !response_body) {
        LOG_ERROR("ClickHouse query failed");
        response_t *resp = response_error(MHD_HTTP_INTERNAL_SERVER_ERROR, "Failed to query events");
        struct MHD_Response *response = MHD_create_response_from_buffer(
            resp->body_len, resp->body, MHD_RESPMEM_MUST_COPY);
        int ret = MHD_queue_response(connection, resp->status_code, response);
        MHD_destroy_response(response);
        response_free(resp);
        return ret;
    }
    
    // Parse JSONEachRow format into array
    json_t *events_array = json_array();
    char *line = response_body;
    char *next_line = NULL;
    
    while (line && *line) {
        next_line = strchr(line, '\n');
        if (next_line) {
            *next_line = '\0';
            next_line++;
        }
        
        if (*line) {
            json_error_t error;
            json_t *event = json_loads(line, 0, &error);
            if (event) {
                json_array_append_new(events_array, event);
            }
        }
        
        line = next_line;
    }
    
    free(response_body);
    
    // Build response
    json_t *result = json_object();
    json_object_set_new(result, "events", events_array);
    json_object_set_new(result, "total", json_integer(json_array_size(events_array)));
    json_object_set_new(result, "limit", json_integer(limit));
    json_object_set_new(result, "offset", json_integer(offset));
    
    char *json_str = json_dumps(result, JSON_COMPACT);
    json_decref(result);
    
    struct MHD_Response *response = MHD_create_response_from_buffer(
        strlen(json_str), json_str, MHD_RESPMEM_MUST_FREE);
    MHD_add_response_header(response, "Content-Type", "application/json");
    int ret = MHD_queue_response(connection, MHD_HTTP_OK, response);
    MHD_destroy_response(response);
    
    return ret;
}

int handle_deliveries(struct MHD_Connection *connection, request_ctx_t *ctx,
                      const char *method, const char *upload_data, size_t *upload_data_size) {
    (void)upload_data;
    (void)upload_data_size;
    
    if (strcmp(method, "GET") != 0) {
        response_t *resp = response_error(MHD_HTTP_METHOD_NOT_ALLOWED, "Method not allowed");
        struct MHD_Response *response = MHD_create_response_from_buffer(
            resp->body_len, resp->body, MHD_RESPMEM_MUST_COPY);
        int ret = MHD_queue_response(connection, resp->status_code, response);
        MHD_destroy_response(response);
        response_free(resp);
        return ret;
    }
    
    // Parse query parameters
    const char *limit_str = MHD_lookup_connection_value(connection, MHD_GET_ARGUMENT_KIND, "limit");
    const char *offset_str = MHD_lookup_connection_value(connection, MHD_GET_ARGUMENT_KIND, "offset");
    
    int limit = limit_str ? atoi(limit_str) : 50;
    int offset = offset_str ? atoi(offset_str) : 0;
    
    if (limit <= 0 || limit > 1000) limit = 50;
    if (offset < 0) offset = 0;
    
    // Query ClickHouse for deliveries
    char query[1024];
    snprintf(query, sizeof(query),
             "SELECT toString(id) as id, toString(event_id) as event_id, "
             "toString(endpoint_id) as endpoint_id, status_code, "
             "success, error_message, attempt_number, delivered_at "
             "FROM deliveries "
             "ORDER BY delivered_at DESC "
             "LIMIT %d OFFSET %d "
             "FORMAT JSONEachRow",
             limit, offset);
    
    char *response_body = NULL;
    size_t response_len = 0;
    
    eth_error_t err = clickhouse_query(ctx->ch_client, query, &response_body, &response_len);
    if (err != ETH_OK || !response_body) {
        LOG_ERROR("ClickHouse query failed");
        response_t *resp = response_error(MHD_HTTP_INTERNAL_SERVER_ERROR, "Failed to query deliveries");
        struct MHD_Response *response = MHD_create_response_from_buffer(
            resp->body_len, resp->body, MHD_RESPMEM_MUST_COPY);
        int ret = MHD_queue_response(connection, resp->status_code, response);
        MHD_destroy_response(response);
        response_free(resp);
        return ret;
    }
    
    // Parse JSONEachRow format into array
    json_t *deliveries_array = json_array();
    char *line = response_body;
    char *next_line = NULL;
    
    while (line && *line) {
        next_line = strchr(line, '\n');
        if (next_line) {
            *next_line = '\0';
            next_line++;
        }
        
        if (*line) {
            json_error_t error;
            json_t *delivery = json_loads(line, 0, &error);
            if (delivery) {
                json_array_append_new(deliveries_array, delivery);
            }
        }
        
        line = next_line;
    }
    
    free(response_body);
    
    // Build response
    json_t *result = json_object();
    json_object_set_new(result, "deliveries", deliveries_array);
    json_object_set_new(result, "total", json_integer(json_array_size(deliveries_array)));
    json_object_set_new(result, "limit", json_integer(limit));
    json_object_set_new(result, "offset", json_integer(offset));
    
    char *json_str = json_dumps(result, JSON_COMPACT);
    json_decref(result);
    
    struct MHD_Response *response = MHD_create_response_from_buffer(
        strlen(json_str), json_str, MHD_RESPMEM_MUST_FREE);
    MHD_add_response_header(response, "Content-Type", "application/json");
    int ret = MHD_queue_response(connection, MHD_HTTP_OK, response);
    MHD_destroy_response(response);
    
    return ret;
}
