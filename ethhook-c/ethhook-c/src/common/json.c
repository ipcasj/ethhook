/**
 * @file json.c
 * @brief JSON parsing utilities (wrapper around cJSON)
 * Single Translation Unit - includes vendored cJSON
 */

/* We'll vendor cJSON for single translation unit approach */
/* For now, this is a placeholder - in full implementation,
 * we'd include cJSON.c directly here */

#include <stdlib.h>
#include <string.h>

/* Placeholder - real implementation would include cJSON library inline */
void *json_parse(const char *str) {
    /* TODO: Implement using cJSON */
    return NULL;
}

void json_free(void *json) {
    /* TODO: Implement */
}

const char *json_get_string(void *json, const char *key) {
    /* TODO: Implement */
    return NULL;
}

int json_get_int(void *json, const char *key, int default_val) {
    /* TODO: Implement */
    return default_val;
}
