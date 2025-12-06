#include "ethhook/admin_api.h"
#include "ethhook/clickhouse.h"
#include <sqlite3.h>
#include "yyjson.h"
#include <string.h>
#include <stdlib.h>

// Helper to get SQLite handle
extern sqlite3 *eth_db_get_handle(eth_db_t *db);

int handle_login(struct MHD_Connection *connection, request_ctx_t *ctx,
                 const char *method, const char *upload_data, size_t *upload_data_size) {
    if (strcmp(method, "POST") != 0) {
        response_t *resp = response_error(MHD_HTTP_METHOD_NOT_ALLOWED, "Method not allowed");
        struct MHD_Response *response = MHD_create_response_from_buffer(
            resp->body_len, resp->body, MHD_RESPMEM_MUST_COPY);
        MHD_add_response_header(response, "Content-Type", "application/json");
        add_cors_headers(response);
        int ret = MHD_queue_response(connection, resp->status_code, response);
        MHD_destroy_response(response);
        response_free(resp);
        return ret;
    }
    
    // Accumulate POST data
    if (*upload_data_size > 0) {
        size_t new_size = ctx->post_data_size + *upload_data_size;
        char *new_data = realloc(ctx->post_data, new_size + 1);
        if (!new_data) {
            return MHD_NO;
        }
        memcpy(new_data + ctx->post_data_size, upload_data, *upload_data_size);
        new_data[new_size] = '\0';
        ctx->post_data = new_data;
        ctx->post_data_size = new_size;
        *upload_data_size = 0;
        return MHD_YES;
    }
    
    // Process accumulated data
    if (!ctx->post_data) {
        response_t *resp = response_error(MHD_HTTP_BAD_REQUEST, "No request body");
        struct MHD_Response *response = MHD_create_response_from_buffer(
            resp->body_len, resp->body, MHD_RESPMEM_MUST_COPY);
        MHD_add_response_header(response, "Content-Type", "application/json");
        add_cors_headers(response);
        int ret = MHD_queue_response(connection, resp->status_code, response);
        MHD_destroy_response(response);
        response_free(resp);
        return ret;
    }
    
    // Parse JSON body
    yyjson_doc *req_doc = yyjson_read(ctx->post_data, ctx->post_data_size, 0);
    if (!req_doc) {
        response_t *resp = response_error(MHD_HTTP_BAD_REQUEST, "Invalid JSON");
        struct MHD_Response *response = MHD_create_response_from_buffer(
            resp->body_len, resp->body, MHD_RESPMEM_MUST_COPY);
        MHD_add_response_header(response, "Content-Type", "application/json");
        add_cors_headers(response);
        int ret = MHD_queue_response(connection, resp->status_code, response);
        MHD_destroy_response(response);
        response_free(resp);
        return ret;
    }
    
    yyjson_val *root = yyjson_doc_get_root(req_doc);
    const char *email = yyjson_get_str(yyjson_obj_get(root, "email"));
    const char *password = yyjson_get_str(yyjson_obj_get(root, "password"));
    
    if (!email || !password) {
        yyjson_doc_free(req_doc);
        response_t *resp = response_error(MHD_HTTP_BAD_REQUEST, "Missing email or password");
        struct MHD_Response *response = MHD_create_response_from_buffer(
            resp->body_len, resp->body, MHD_RESPMEM_MUST_COPY);
        MHD_add_response_header(response, "Content-Type", "application/json");
        add_cors_headers(response);
        int ret = MHD_queue_response(connection, resp->status_code, response);
        MHD_destroy_response(response);
        response_free(resp);
        return ret;
    }
    
    // Query database for user by email only
    sqlite3 *db_handle = eth_db_get_handle(ctx->db);
    sqlite3_stmt *stmt = NULL;
    const char *query = "SELECT id, password_hash, is_admin FROM users WHERE username = ?";
    
    if (sqlite3_prepare_v2(db_handle, query, -1, &stmt, NULL) != SQLITE_OK) {
        yyjson_doc_free(req_doc);
        response_t *resp = response_error(MHD_HTTP_INTERNAL_SERVER_ERROR, "Database error");
        struct MHD_Response *response = MHD_create_response_from_buffer(
            resp->body_len, resp->body, MHD_RESPMEM_MUST_COPY);
        MHD_add_response_header(response, "Content-Type", "application/json");
        add_cors_headers(response);
        int ret = MHD_queue_response(connection, resp->status_code, response);
        MHD_destroy_response(response);
        response_free(resp);
        return ret;
    }
    
    sqlite3_bind_text(stmt, 1, email, -1, SQLITE_STATIC);
    
    int step_result = sqlite3_step(stmt);
    if (step_result != SQLITE_ROW) {
        sqlite3_finalize(stmt);
        yyjson_doc_free(req_doc);
        response_t *resp = response_error(MHD_HTTP_UNAUTHORIZED, "Invalid credentials");
        struct MHD_Response *response = MHD_create_response_from_buffer(
            resp->body_len, resp->body, MHD_RESPMEM_MUST_COPY);
        MHD_add_response_header(response, "Content-Type", "application/json");
        add_cors_headers(response);
        int ret = MHD_queue_response(connection, resp->status_code, response);
        MHD_destroy_response(response);
        response_free(resp);
        return ret;
    }
    
    const char *user_id = (const char *)sqlite3_column_text(stmt, 0);
    const char *password_hash = (const char *)sqlite3_column_text(stmt, 1);
    int is_admin = sqlite3_column_int(stmt, 2);
    
    // Copy user_id before finalizing statement
    char *user_id_copy = strdup(user_id);
    
    // Verify password
    bool password_valid = bcrypt_verify(password, password_hash);
    
    sqlite3_finalize(stmt);
    yyjson_doc_free(req_doc);
    
    if (!password_valid) {
        free(user_id_copy);
        response_t *resp = response_error(MHD_HTTP_UNAUTHORIZED, "Invalid credentials");
        struct MHD_Response *response = MHD_create_response_from_buffer(
            resp->body_len, resp->body, MHD_RESPMEM_MUST_COPY);
        MHD_add_response_header(response, "Content-Type", "application/json");
        add_cors_headers(response);
        int ret = MHD_queue_response(connection, resp->status_code, response);
        MHD_destroy_response(response);
        response_free(resp);
        return ret;
    }
    
    // Generate JWT token
    char *token = jwt_create(user_id_copy, is_admin != 0, ctx->jwt_secret, 24);
    free(user_id_copy);
    
    if (!token) {
        response_t *resp = response_error(MHD_HTTP_INTERNAL_SERVER_ERROR, "Failed to generate token");
        struct MHD_Response *response = MHD_create_response_from_buffer(
            resp->body_len, resp->body, MHD_RESPMEM_MUST_COPY);
        MHD_add_response_header(response, "Content-Type", "application/json");
        add_cors_headers(response);
        int ret = MHD_queue_response(connection, resp->status_code, response);
        MHD_destroy_response(response);
        response_free(resp);
        return ret;
    }
    
    // Build response with token
    yyjson_mut_doc *resp_doc = yyjson_mut_doc_new(NULL);
    yyjson_mut_val *result = yyjson_mut_obj(resp_doc);
    yyjson_mut_doc_set_root(resp_doc, result);
    yyjson_mut_obj_add_str(resp_doc, result, "token", token);
    
    free(token);
    
    size_t json_len;
    char *json_str = yyjson_mut_write(resp_doc, 0, &json_len);
    yyjson_mut_doc_free(resp_doc);
    
    struct MHD_Response *response = MHD_create_response_from_buffer(
        json_len, json_str, MHD_RESPMEM_MUST_FREE);
    MHD_add_response_header(response, "Content-Type", "application/json");
    add_cors_headers(response);
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
        MHD_add_response_header(response, "Content-Type", "application/json");
        add_cors_headers(response);
        int ret = MHD_queue_response(connection, resp->status_code, response);
        MHD_destroy_response(response);
        response_free(resp);
        return ret;
    }
    
    if (strcmp(method, "GET") != 0) {
        response_t *resp = response_error(MHD_HTTP_METHOD_NOT_ALLOWED, "Method not allowed");
        struct MHD_Response *response = MHD_create_response_from_buffer(
            resp->body_len, resp->body, MHD_RESPMEM_MUST_COPY);
        MHD_add_response_header(response, "Content-Type", "application/json");
        add_cors_headers(response);
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
        MHD_add_response_header(response, "Content-Type", "application/json");
        add_cors_headers(response);
        int ret = MHD_queue_response(connection, resp->status_code, response);
        MHD_destroy_response(response);
        response_free(resp);
        return ret;
    }
    
    yyjson_mut_doc *doc = yyjson_mut_doc_new(NULL);
    yyjson_mut_val *users_array = yyjson_mut_arr(doc);
    
    while ((rc = sqlite3_step(stmt)) == SQLITE_ROW) {
        yyjson_mut_val *user = yyjson_mut_obj(doc);
        
        const char *id = (const char *)sqlite3_column_text(stmt, 0);
        const char *email = (const char *)sqlite3_column_text(stmt, 1);
        int is_admin = sqlite3_column_int(stmt, 2);
        const char *created_at = (const char *)sqlite3_column_text(stmt, 3);
        
        if (id) yyjson_mut_obj_add_str(doc, user, "id", id);
        if (email) yyjson_mut_obj_add_str(doc, user, "email", email);
        yyjson_mut_obj_add_bool(doc, user, "is_admin", is_admin);
        if (created_at) yyjson_mut_obj_add_str(doc, user, "created_at", created_at);
        
        yyjson_mut_arr_append(users_array, user);
    }
    
    sqlite3_finalize(stmt);
    
    yyjson_mut_val *result = yyjson_mut_obj(doc);
    yyjson_mut_doc_set_root(doc, result);
    yyjson_mut_obj_add_val(doc, result, "users", users_array);
    yyjson_mut_obj_add_uint(doc, result, "total", yyjson_mut_arr_size(users_array));
    
    size_t json_len;
    char *json_str = yyjson_mut_write(doc, 0, &json_len);
    yyjson_mut_doc_free(doc);
    
    struct MHD_Response *response = MHD_create_response_from_buffer(
        json_len, json_str, MHD_RESPMEM_MUST_FREE);
    MHD_add_response_header(response, "Content-Type", "application/json");
    add_cors_headers(response);
    int ret = MHD_queue_response(connection, MHD_HTTP_OK, response);
    MHD_destroy_response(response);
    
    return ret;
}

int handle_applications(struct MHD_Connection *connection, request_ctx_t *ctx __attribute__((unused)),
                        const char *method, const char *upload_data, size_t *upload_data_size) {
    (void)upload_data;
    (void)upload_data_size;
    
    if (strcmp(method, "GET") != 0) {
        response_t *resp = response_error(MHD_HTTP_METHOD_NOT_ALLOWED, "Method not allowed");
        struct MHD_Response *response = MHD_create_response_from_buffer(
            resp->body_len, resp->body, MHD_RESPMEM_MUST_COPY);
        MHD_add_response_header(response, "Content-Type", "application/json");
        add_cors_headers(response);
        int ret = MHD_queue_response(connection, resp->status_code, response);
        MHD_destroy_response(response);
        response_free(resp);
        return ret;
    }
    
    // Query applications
    yyjson_mut_doc *doc = yyjson_mut_doc_new(NULL);
    yyjson_mut_val *apps_array = yyjson_mut_arr(doc);
    
    yyjson_mut_val *result = yyjson_mut_obj(doc);
    yyjson_mut_doc_set_root(doc, result);
    yyjson_mut_obj_add_val(doc, result, "applications", apps_array);
    
    size_t json_len;
    char *json_str = yyjson_mut_write(doc, 0, &json_len);
    yyjson_mut_doc_free(doc);
    
    struct MHD_Response *response = MHD_create_response_from_buffer(
        json_len, json_str, MHD_RESPMEM_MUST_FREE);
    MHD_add_response_header(response, "Content-Type", "application/json");
    add_cors_headers(response);
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
    
    yyjson_mut_doc *doc = yyjson_mut_doc_new(NULL);
    yyjson_mut_val *result = yyjson_mut_obj(doc);
    yyjson_mut_doc_set_root(doc, result);
    yyjson_mut_obj_add_val(doc, result, "endpoints", yyjson_mut_arr(doc));
    
    size_t json_len;
    char *json_str = yyjson_mut_write(doc, 0, &json_len);
    yyjson_mut_doc_free(doc);
    
    struct MHD_Response *response = MHD_create_response_from_buffer(
        json_len, json_str, MHD_RESPMEM_MUST_FREE);
    MHD_add_response_header(response, "Content-Type", "application/json");
    add_cors_headers(response);
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
        MHD_add_response_header(response, "Content-Type", "application/json");
        add_cors_headers(response);
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
    
    clickhouse_result_t *result = NULL;
    
    eth_error_t err = clickhouse_query(ctx->ch_client, query, &result);
    if (err != ETH_OK || !result || !result->data) {
        LOG_ERROR("ClickHouse query failed");
        if (result) clickhouse_result_free(result);
        response_t *resp = response_error(MHD_HTTP_INTERNAL_SERVER_ERROR, "Failed to query events");
        struct MHD_Response *response = MHD_create_response_from_buffer(
            resp->body_len, resp->body, MHD_RESPMEM_MUST_COPY);
        MHD_add_response_header(response, "Content-Type", "application/json");
        add_cors_headers(response);
        int ret = MHD_queue_response(connection, resp->status_code, response);
        MHD_destroy_response(response);
        response_free(resp);
        return ret;
    }
    
    char *response_body = result->data;
    
    // Parse JSONEachRow format into array
    yyjson_mut_doc *doc = yyjson_mut_doc_new(NULL);
    yyjson_mut_val *events_array = yyjson_mut_arr(doc);
    char *line = response_body;
    char *next_line = NULL;
    
    while (line && *line) {
        next_line = strchr(line, '\n');
        if (next_line) {
            *next_line = '\0';
            next_line++;
        }
        
        if (*line) {
            yyjson_doc *line_doc = yyjson_read(line, strlen(line), 0);
            if (line_doc) {
                yyjson_val *event_val = yyjson_doc_get_root(line_doc);
                if (event_val) {
                    // Convert immutable to mutable
                    yyjson_mut_val *event = yyjson_val_mut_copy(doc, event_val);
                    yyjson_mut_arr_append(events_array, event);
                }
                yyjson_doc_free(line_doc);
            }
        }
        
        line = next_line;
    }
    
    clickhouse_result_free(result);
    
    // Build response
    yyjson_mut_val *result_obj = yyjson_mut_obj(doc);
    yyjson_mut_doc_set_root(doc, result_obj);
    yyjson_mut_obj_add_val(doc, result_obj, "events", events_array);
    yyjson_mut_obj_add_uint(doc, result_obj, "total", yyjson_mut_arr_size(events_array));
    yyjson_mut_obj_add_int(doc, result_obj, "limit", limit);
    yyjson_mut_obj_add_int(doc, result_obj, "offset", offset);
    
    size_t json_len;
    char *json_str = yyjson_mut_write(doc, 0, &json_len);
    yyjson_mut_doc_free(doc);
    
    struct MHD_Response *response = MHD_create_response_from_buffer(
        json_len, json_str, MHD_RESPMEM_MUST_FREE);
    MHD_add_response_header(response, "Content-Type", "application/json");
    add_cors_headers(response);
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
        MHD_add_response_header(response, "Content-Type", "application/json");
        add_cors_headers(response);
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
    
    clickhouse_result_t *result = NULL;
    
    eth_error_t err = clickhouse_query(ctx->ch_client, query, &result);
    if (err != ETH_OK || !result || !result->data) {
        LOG_ERROR("ClickHouse query failed");
        if (result) clickhouse_result_free(result);
        response_t *resp = response_error(MHD_HTTP_INTERNAL_SERVER_ERROR, "Failed to query deliveries");
        struct MHD_Response *response = MHD_create_response_from_buffer(
            resp->body_len, resp->body, MHD_RESPMEM_MUST_COPY);
        MHD_add_response_header(response, "Content-Type", "application/json");
        add_cors_headers(response);
        int ret = MHD_queue_response(connection, resp->status_code, response);
        MHD_destroy_response(response);
        response_free(resp);
        return ret;
    }
    
    char *response_body = result->data;
    
    // Parse JSONEachRow format into array
    yyjson_mut_doc *doc = yyjson_mut_doc_new(NULL);
    yyjson_mut_val *deliveries_array = yyjson_mut_arr(doc);
    char *line = response_body;
    char *next_line = NULL;
    
    while (line && *line) {
        next_line = strchr(line, '\n');
        if (next_line) {
            *next_line = '\0';
            next_line++;
        }
        
        if (*line) {
            yyjson_doc *line_doc = yyjson_read(line, strlen(line), 0);
            if (line_doc) {
                yyjson_val *delivery_val = yyjson_doc_get_root(line_doc);
                if (delivery_val) {
                    // Convert immutable to mutable
                    yyjson_mut_val *delivery = yyjson_val_mut_copy(doc, delivery_val);
                    yyjson_mut_arr_append(deliveries_array, delivery);
                }
                yyjson_doc_free(line_doc);
            }
        }
        
        line = next_line;
    }
    
    clickhouse_result_free(result);
    
    // Build response
    yyjson_mut_val *result_obj = yyjson_mut_obj(doc);
    yyjson_mut_doc_set_root(doc, result_obj);
    yyjson_mut_obj_add_val(doc, result_obj, "deliveries", deliveries_array);
    yyjson_mut_obj_add_uint(doc, result_obj, "total", yyjson_mut_arr_size(deliveries_array));
    yyjson_mut_obj_add_int(doc, result_obj, "limit", limit);
    yyjson_mut_obj_add_int(doc, result_obj, "offset", offset);
    
    size_t json_len;
    char *json_str = yyjson_mut_write(doc, 0, &json_len);
    yyjson_mut_doc_free(doc);
    
    struct MHD_Response *response = MHD_create_response_from_buffer(
        json_len, json_str, MHD_RESPMEM_MUST_FREE);
    MHD_add_response_header(response, "Content-Type", "application/json");
    add_cors_headers(response);
    int ret = MHD_queue_response(connection, MHD_HTTP_OK, response);
    MHD_destroy_response(response);
    
    return ret;
}
