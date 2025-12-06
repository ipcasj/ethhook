#ifndef ETHHOOK_ADMIN_API_H
#define ETHHOOK_ADMIN_API_H

#include "common.h"
#include "clickhouse.h"
#include <microhttpd.h>

// Request context
typedef struct {
    eth_db_t *db;
    clickhouse_client_t *ch_client;
    const char *jwt_secret;
    char *user_id;
    bool is_admin;
    char *post_data;
    size_t post_data_size;
} request_ctx_t;

// Response builder
typedef struct {
    char *body;
    size_t body_len;
    int status_code;
    const char *content_type;
} response_t;

// API handlers
int handle_login(struct MHD_Connection *connection, request_ctx_t *ctx,
                 const char *method, const char *upload_data, size_t *upload_data_size);

int handle_users(struct MHD_Connection *connection, request_ctx_t *ctx,
                 const char *method, const char *upload_data, size_t *upload_data_size);

int handle_applications(struct MHD_Connection *connection, request_ctx_t *ctx,
                        const char *method, const char *upload_data, size_t *upload_data_size);

int handle_endpoints(struct MHD_Connection *connection, request_ctx_t *ctx,
                     const char *method, const char *upload_data, size_t *upload_data_size);

int handle_events(struct MHD_Connection *connection, request_ctx_t *ctx,
                  const char *method, const char *upload_data, size_t *upload_data_size);

int handle_deliveries(struct MHD_Connection *connection, request_ctx_t *ctx,
                      const char *method, const char *upload_data, size_t *upload_data_size);

// JWT functions
char *jwt_create(const char *user_id, bool is_admin, const char *secret, int expiry_hours);
eth_error_t jwt_verify(const char *token, const char *secret, char **user_id, bool *is_admin);

// Password verification
bool bcrypt_verify(const char *password, const char *hash);

// JSON response helpers
response_t *response_json(int status_code, const char *json);
response_t *response_error(int status_code, const char *message);
void response_free(response_t *resp);

// CORS helper
void add_cors_headers(struct MHD_Response *response);

// Admin API server
typedef struct admin_api_ctx admin_api_ctx_t;

eth_error_t admin_api_ctx_create(eth_config_t *config, admin_api_ctx_t **ctx);
void admin_api_ctx_destroy(admin_api_ctx_t *ctx);
eth_error_t admin_api_run(admin_api_ctx_t *ctx);
void admin_api_stop(admin_api_ctx_t *ctx);

#endif // ETHHOOK_ADMIN_API_H
