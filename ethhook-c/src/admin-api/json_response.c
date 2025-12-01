#include "ethhook/admin_api.h"
#include <jansson.h>
#include <string.h>
#include <stdlib.h>

response_t *response_json(int status_code, const char *json) {
    response_t *resp = calloc(1, sizeof(response_t));
    if (!resp) {
        return NULL;
    }
    
    resp->status_code = status_code;
    resp->content_type = "application/json";
    resp->body = strdup(json);
    resp->body_len = strlen(json);
    
    return resp;
}

response_t *response_error(int status_code, const char *message) {
    json_t *root = json_object();
    json_object_set_new(root, "error", json_string(message));
    
    char *json_str = json_dumps(root, JSON_COMPACT);
    json_decref(root);
    
    if (!json_str) {
        return NULL;
    }
    
    response_t *resp = response_json(status_code, json_str);
    free(json_str);
    
    return resp;
}

void response_free(response_t *resp) {
    if (!resp) {
        return;
    }
    
    free(resp->body);
    free(resp);
}
