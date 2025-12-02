#define _GNU_SOURCE
#include "ethhook/admin_api.h"
#include "yyjson.h"
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
    yyjson_mut_doc *doc = yyjson_mut_doc_new(NULL);
    yyjson_mut_val *root = yyjson_mut_obj(doc);
    yyjson_mut_doc_set_root(doc, root);
    yyjson_mut_obj_add_str(doc, root, "error", message);
    
    size_t json_len;
    char *json_str = yyjson_mut_write(doc, 0, &json_len);
    yyjson_mut_doc_free(doc);
    
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
