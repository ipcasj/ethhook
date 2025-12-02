#include "ethhook/ingestor.h"
#include <libwebsockets.h>
#include "yyjson.h"
#include <string.h>
#include <stdlib.h>

#define MAX_PAYLOAD_SIZE (64 * 1024)

typedef struct {
    ws_connection_t *conn;
    char buffer[MAX_PAYLOAD_SIZE];
    size_t buffer_len;
    bool subscribed;
} ws_session_data_t;

static int ws_callback(struct lws *wsi, enum lws_callback_reasons reason,
                       void *user, void *in, size_t len) {
    ws_session_data_t *session = (ws_session_data_t *)user;
    
    switch (reason) {
        case LWS_CALLBACK_CLIENT_ESTABLISHED:
            LOG_INFO("WebSocket connection established for chain %lu", 
                    session->conn->chain_id);
            
            // Send subscription request
            {
                yyjson_mut_doc *doc = yyjson_mut_doc_new(NULL);
                yyjson_mut_val *sub_request = yyjson_mut_obj(doc);
                yyjson_mut_doc_set_root(doc, sub_request);
                
                yyjson_mut_obj_add_str(doc, sub_request, "jsonrpc", "2.0");
                yyjson_mut_obj_add_int(doc, sub_request, "id", 1);
                yyjson_mut_obj_add_str(doc, sub_request, "method", "eth_subscribe");
                
                yyjson_mut_val *params = yyjson_mut_arr(doc);
                yyjson_mut_arr_add_str(doc, params, "logs");
                yyjson_mut_arr_add_val(params, yyjson_mut_obj(doc)); // Empty filter
                yyjson_mut_obj_add_val(doc, sub_request, "params", params);
                
                size_t json_len;
                char *json_str = yyjson_mut_write(doc, 0, &json_len);
                yyjson_mut_doc_free(doc);
                
                if (json_str) {
                    unsigned char buf[LWS_PRE + json_len];
                    memcpy(&buf[LWS_PRE], json_str, json_len);
                    lws_write(wsi, &buf[LWS_PRE], json_len, LWS_WRITE_TEXT);
                    free(json_str);
                    
                    LOG_INFO("Sent subscription request for chain %lu", 
                            session->conn->chain_id);
                }
            }
            circuit_breaker_success(&session->conn->circuit_breaker);
            break;
            
        case LWS_CALLBACK_CLIENT_RECEIVE:
            if (len > 0 && session->buffer_len + len < MAX_PAYLOAD_SIZE) {
                memcpy(session->buffer + session->buffer_len, in, len);
                session->buffer_len += len;
                
                // Check if message is complete
                if (lws_is_final_fragment(wsi)) {
                    session->buffer[session->buffer_len] = '\0';
                    
                    // Parse JSON
                    yyjson_doc *doc = yyjson_read(session->buffer, session->buffer_len, 0);
                    
                    if (doc) {
                        yyjson_val *root = yyjson_doc_get_root(doc);
                        
                        // Check if this is a subscription confirmation
                        yyjson_val *result = yyjson_obj_get(root, "result");
                        if (result && yyjson_is_str(result)) {
                            if (!session->subscribed) {
                                session->subscribed = true;
                                LOG_INFO("Subscription confirmed for chain %lu: %s",
                                        session->conn->chain_id, yyjson_get_str(result));
                            }
                        }
                        
                        // Check if this is an event notification
                        yyjson_val *method = yyjson_obj_get(root, "method");
                        yyjson_val *params = yyjson_obj_get(root, "params");
                        
                        if (method && yyjson_is_str(method) && 
                            strcmp(yyjson_get_str(method), "eth_subscription") == 0 &&
                            params && yyjson_is_obj(params)) {
                            
                            yyjson_val *result_obj = yyjson_obj_get(params, "result");
                            if (result_obj && yyjson_is_obj(result_obj)) {
                                atomic_fetch_add(&session->conn->events_received, 1);
                                
                                // TODO: Publish to Redis
                                // For now, just log
                                size_t event_len;
                                char *event_str = yyjson_val_write(result_obj, 0, &event_len);
                                if (event_str) {
                                    LOG_DEBUG("Received event for chain %lu: %s",
                                            session->conn->chain_id, event_str);
                                    
                                    // TODO: Publish to Redis stream
                                    // Format: XADD events:chain_id * event <json>
                                    
                                    atomic_fetch_add(&session->conn->events_published, 1);
                                    free(event_str);
                                }
                            }
                        }
                        
                        yyjson_doc_free(doc);
                    } else {
                        LOG_ERROR("Failed to parse JSON from chain %lu",
                                session->conn->chain_id);
                        atomic_fetch_add(&session->conn->errors, 1);
                    }
                    
                    session->buffer_len = 0;
                }
            }
            break;
            
        case LWS_CALLBACK_CLIENT_CONNECTION_ERROR:
            LOG_ERROR("WebSocket connection error for chain %lu: %s",
                    session->conn->chain_id, in ? (char *)in : "unknown");
            circuit_breaker_failure(&session->conn->circuit_breaker);
            atomic_fetch_add(&session->conn->errors, 1);
            break;
            
        case LWS_CALLBACK_CLIENT_CLOSED:
            LOG_INFO("WebSocket connection closed for chain %lu", 
                    session->conn->chain_id);
            session->subscribed = false;
            break;
            
        default:
            break;
    }
    
    return 0;
}

static struct lws_protocols protocols[] = {
    {
        .name = "ethereum-json-rpc",
        .callback = ws_callback,
        .per_session_data_size = sizeof(ws_session_data_t),
        .rx_buffer_size = MAX_PAYLOAD_SIZE,
        .id = 0,
        .user = NULL,
        .tx_packet_size = 0
    },
    { NULL, NULL, 0, 0, 0, NULL, 0 }
};

eth_error_t ws_connection_init(ws_connection_t *conn, uint64_t chain_id,
                                const char *ws_url, const char *redis_host, int redis_port) {
    (void)redis_host;  // TODO: Will be used for Redis publishing
    (void)redis_port;  // TODO: Will be used for Redis publishing
    
    if (!conn || !ws_url) {
        return ETH_ERROR_INVALID_PARAM;
    }
    
    conn->chain_id = chain_id;
    conn->ws_url = strdup(ws_url);
    conn->wsi = NULL;
    
    // Initialize circuit breaker (5 failures, 30 second timeout)
    circuit_breaker_init(&conn->circuit_breaker, 5, 30000);
    
    // Initialize arena allocator
    conn->arena = eth_arena_create(1024 * 1024); // 1MB
    if (!conn->arena) {
        free(conn->ws_url);
        return ETH_ERROR_MEMORY;
    }
    
    // Initialize stats
    atomic_init(&conn->events_received, 0);
    atomic_init(&conn->events_published, 0);
    atomic_init(&conn->errors, 0);
    
    // TODO: Initialize Redis connection
    conn->redis_ctx = NULL;
    
    return ETH_OK;
}

eth_error_t ws_connection_start(ws_connection_t *conn) {
    if (!conn) {
        return ETH_ERROR_INVALID_PARAM;
    }
    
    // Check circuit breaker
    if (!circuit_breaker_allow(&conn->circuit_breaker)) {
        LOG_WARN("Circuit breaker open for chain %lu, skipping connection",
                conn->chain_id);
        return ETH_ERROR_CIRCUIT_OPEN;
    }
    
    struct lws_context_creation_info info;
    memset(&info, 0, sizeof(info));
    
    info.port = CONTEXT_PORT_NO_LISTEN;
    info.protocols = protocols;
    info.gid = -1;
    info.uid = -1;
    info.options = LWS_SERVER_OPTION_DO_SSL_GLOBAL_INIT;
    
    struct lws_context *context = lws_create_context(&info);
    if (!context) {
        LOG_ERROR("Failed to create libwebsockets context for chain %lu",
                conn->chain_id);
        circuit_breaker_failure(&conn->circuit_breaker);
        return ETH_ERROR_WEBSOCKET;
    }
    
    // Parse WebSocket URL
    struct lws_client_connect_info ccinfo;
    memset(&ccinfo, 0, sizeof(ccinfo));
    
    ccinfo.context = context;
    ccinfo.address = "eth-mainnet.g.alchemy.com"; // TODO: Parse from ws_url
    ccinfo.port = 443;
    ccinfo.path = "/v2/YOUR_KEY"; // TODO: Parse from ws_url
    ccinfo.host = ccinfo.address;
    ccinfo.origin = ccinfo.address;
    ccinfo.protocol = protocols[0].name;
    ccinfo.ssl_connection = LCCSCF_USE_SSL;
    
    conn->wsi = lws_client_connect_via_info(&ccinfo);
    if (!conn->wsi) {
        LOG_ERROR("Failed to connect WebSocket for chain %lu", conn->chain_id);
        lws_context_destroy(context);
        circuit_breaker_failure(&conn->circuit_breaker);
        return ETH_ERROR_WEBSOCKET;
    }
    
    // Event loop
    while (lws_service(context, 1000) >= 0) {
        // Check if we should stop
        // TODO: Add stop mechanism
    }
    
    lws_context_destroy(context);
    return ETH_OK;
}

void ws_connection_stop(ws_connection_t *conn) {
    if (conn && conn->wsi) {
        // TODO: Gracefully close WebSocket
    }
}

void ws_connection_cleanup(ws_connection_t *conn) {
    if (!conn) {
        return;
    }
    
    free(conn->ws_url);
    
    if (conn->arena) {
        eth_arena_destroy(conn->arena);
    }
    
    // TODO: Cleanup Redis connection
}
