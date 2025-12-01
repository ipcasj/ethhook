/**
 * High-Performance JSON Utilities
 * 
 * Wrapper around yyjson for 2-3x faster JSON parsing than jansson.
 * Uses SIMD instructions when available.
 */

#ifndef ETHHOOK_JSON_H
#define ETHHOOK_JSON_H

#include "ethhook/common.h"
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// Forward declarations
typedef struct json_doc json_doc_t;
typedef struct json_value json_value_t;
typedef struct json_object json_object_t;
typedef struct json_array json_array_t;

/**
 * Parse JSON document from string
 * 
 * Uses yy json for 2-3x faster parsing.
 * Thread-safe.
 * 
 * @param json JSON string
 * @param len JSON string length
 * @param doc Output document pointer
 * @return ETH_OK on success
 */
eth_error_t json_parse(const char *json, size_t len, json_doc_t **doc);

/**
 * Free JSON document
 * 
 * @param doc Document to free
 */
void json_doc_free(json_doc_t *doc);

/**
 * Get root value from document
 * 
 * @param doc JSON document
 * @return Root value or NULL
 */
json_value_t *json_doc_get_root(json_doc_t *doc);

/**
 * Get object from value
 * 
 * @param value JSON value
 * @return Object or NULL if not an object
 */
json_object_t *json_value_get_object(json_value_t *value);

/**
 * Get array from value
 * 
 * @param value JSON value
 * @return Array or NULL if not an array
 */
json_array_t *json_value_get_array(json_value_t *value);

/**
 * Get string from value
 * 
 * @param value JSON value
 * @return String or NULL if not a string
 */
const char *json_value_get_string(json_value_t *value);

/**
 * Get integer from value
 * 
 * @param value JSON value
 * @param default_val Default value if not an integer
 * @return Integer value
 */
int64_t json_value_get_int(json_value_t *value, int64_t default_val);

/**
 * Get unsigned integer from value
 * 
 * @param value JSON value
 * @param default_val Default value if not an integer
 * @return Unsigned integer value
 */
uint64_t json_value_get_uint(json_value_t *value, uint64_t default_val);

/**
 * Get boolean from value
 * 
 * @param value JSON value
 * @param default_val Default value if not a boolean
 * @return Boolean value
 */
bool json_value_get_bool(json_value_t *value, bool default_val);

/**
 * Get value from object by key
 * 
 * @param obj JSON object
 * @param key Object key
 * @return Value or NULL if key not found
 */
json_value_t *json_object_get(json_object_t *obj, const char *key);

/**
 * Get array length
 * 
 * @param arr JSON array
 * @return Array length
 */
size_t json_array_get_length(json_array_t *arr);

/**
 * Get array element by index
 * 
 * @param arr JSON array
 * @param index Element index
 * @return Value or NULL if index out of bounds
 */
json_value_t *json_array_get(json_array_t *arr, size_t index);

// =============================================================================
// JSON GENERATION (WRITER)
// =============================================================================

typedef struct json_writer json_writer_t;

/**
 * Create JSON writer
 * 
 * @param writer Output writer pointer
 * @return ETH_OK on success
 */
eth_error_t json_writer_create(json_writer_t **writer);

/**
 * Free JSON writer
 * 
 * @param writer Writer to free
 */
void json_writer_free(json_writer_t *writer);

/**
 * Begin object
 * 
 * @param writer JSON writer
 * @return ETH_OK on success
 */
eth_error_t json_writer_begin_object(json_writer_t *writer);

/**
 * End object
 * 
 * @param writer JSON writer
 * @return ETH_OK on success
 */
eth_error_t json_writer_end_object(json_writer_t *writer);

/**
 * Begin array
 * 
 * @param writer JSON writer
 * @return ETH_OK on success
 */
eth_error_t json_writer_begin_array(json_writer_t *writer);

/**
 * End array
 * 
 * @param writer JSON writer
 * @return ETH_OK on success
 */
eth_error_t json_writer_end_array(json_writer_t *writer);

/**
 * Write object key
 * 
 * @param writer JSON writer
 * @param key Object key
 * @return ETH_OK on success
 */
eth_error_t json_writer_key(json_writer_t *writer, const char *key);

/**
 * Write string value
 * 
 * @param writer JSON writer
 * @param value String value
 * @return ETH_OK on success
 */
eth_error_t json_writer_string(json_writer_t *writer, const char *value);

/**
 * Write integer value
 * 
 * @param writer JSON writer
 * @param value Integer value
 * @return ETH_OK on success
 */
eth_error_t json_writer_int(json_writer_t *writer, int64_t value);

/**
 * Write unsigned integer value
 * 
 * @param writer JSON writer
 * @param value Unsigned integer value
 * @return ETH_OK on success
 */
eth_error_t json_writer_uint(json_writer_t *writer, uint64_t value);

/**
 * Write boolean value
 * 
 * @param writer JSON writer
 * @param value Boolean value
 * @return ETH_OK on success
 */
eth_error_t json_writer_bool(json_writer_t *writer, bool value);

/**
 * Write null value
 * 
 * @param writer JSON writer
 * @return ETH_OK on success
 */
eth_error_t json_writer_null(json_writer_t *writer);

/**
 * Get JSON string from writer
 * 
 * Caller must free the returned string.
 * 
 * @param writer JSON writer
 * @param json Output JSON string pointer
 * @param len Output JSON string length
 * @return ETH_OK on success
 */
eth_error_t json_writer_get_string(json_writer_t *writer, char **json, size_t *len);

#ifdef __cplusplus
}
#endif

#endif // ETHHOOK_JSON_H
