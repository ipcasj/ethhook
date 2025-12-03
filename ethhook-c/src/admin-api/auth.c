#define _GNU_SOURCE
#include "ethhook/admin_api.h"
#include <openssl/hmac.h>
#include <openssl/evp.h>
#include <openssl/bio.h>
#include <openssl/buffer.h>
#include <string.h>
#include <stdlib.h>
#include <stdio.h>
#include <time.h>

// Simple JWT implementation using OpenSSL (HS256 only)
// More maintainable and portable than dealing with libjwt v3 API changes

static char *base64url_encode(const unsigned char *data, size_t len) {
    BIO *bio = NULL;
    BIO *b64 = NULL;
    BUF_MEM *buffer = NULL;
    
    b64 = BIO_new(BIO_f_base64());
    bio = BIO_new(BIO_s_mem());
    bio = BIO_push(b64, bio);
    
    BIO_set_flags(bio, BIO_FLAGS_BASE64_NO_NL);
    BIO_write(bio, data, (int)len);
    (void)BIO_flush(bio);
    BIO_get_mem_ptr(bio, &buffer);
    
    // Convert base64 to base64url: + -> -, / -> _, remove =
    char *result = malloc(buffer->length + 1);
    if (!result) {
        BIO_free_all(bio);
        return NULL;
    }
    
    size_t out_len = 0;
    for (size_t i = 0; i < buffer->length; i++) {
        char c = buffer->data[i];
        if (c == '+') result[out_len++] = '-';
        else if (c == '/') result[out_len++] = '_';
        else if (c != '=') result[out_len++] = c;
    }
    result[out_len] = '\0';
    
    BIO_free_all(bio);
    return result;
}

static unsigned char *base64url_decode(const char *data, size_t *out_len) {
    // Convert base64url to base64: - -> +, _ -> /
    size_t len = strlen(data);
    char *b64 = malloc(len + 4); // +4 for padding
    if (!b64) return NULL;
    
    for (size_t i = 0; i < len; i++) {
        char c = data[i];
        if (c == '-') b64[i] = '+';
        else if (c == '_') b64[i] = '/';
        else b64[i] = c;
    }
    
    // Add padding
    size_t padding = (4 - (len % 4)) % 4;
    for (size_t i = 0; i < padding; i++) {
        b64[len + i] = '=';
    }
    b64[len + padding] = '\0';
    
    // Decode
    BIO *bio = BIO_new_mem_buf(b64, -1);
    BIO *b64_bio = BIO_new(BIO_f_base64());
    bio = BIO_push(b64_bio, bio);
    BIO_set_flags(bio, BIO_FLAGS_BASE64_NO_NL);
    
    unsigned char *buffer = malloc(len);
    if (!buffer) {
        BIO_free_all(bio);
        free(b64);
        return NULL;
    }
    
    *out_len = (size_t)BIO_read(bio, buffer, (int)len);
    BIO_free_all(bio);
    free(b64);
    
    return buffer;
}

char *jwt_create(const char *user_id, bool is_admin, const char *secret, int expiry_hours) {
    // Build header: {"alg":"HS256","typ":"JWT"}
    const char *header = "{\"alg\":\"HS256\",\"typ\":\"JWT\"}";
    
    // Build payload
    time_t now = time(NULL);
    time_t exp = now + (expiry_hours * 3600);
    
    char payload[512];
    snprintf(payload, sizeof(payload),
        "{\"sub\":\"%s\",\"admin\":\"%s\",\"exp\":%ld,\"iat\":%ld}",
        user_id, is_admin ? "true" : "false", (long)exp, (long)now);
    
    // Encode header and payload
    char *header_b64 = base64url_encode((unsigned char *)header, strlen(header));
    char *payload_b64 = base64url_encode((unsigned char *)payload, strlen(payload));
    
    if (!header_b64 || !payload_b64) {
        free(header_b64);
        free(payload_b64);
        return NULL;
    }
    
    // Create signature data: header.payload
    size_t data_len = strlen(header_b64) + strlen(payload_b64) + 2;
    char *data = malloc(data_len);
    if (!data) {
        free(header_b64);
        free(payload_b64);
        return NULL;
    }
    snprintf(data, data_len, "%s.%s", header_b64, payload_b64);
    
    // Calculate HMAC-SHA256 signature
    unsigned char signature[EVP_MAX_MD_SIZE];
    unsigned int sig_len = 0;
    
    HMAC(EVP_sha256(), secret, (int)strlen(secret),
         (unsigned char *)data, strlen(data), signature, &sig_len);
    
    // Encode signature
    char *sig_b64 = base64url_encode(signature, sig_len);
    if (!sig_b64) {
        free(header_b64);
        free(payload_b64);
        free(data);
        return NULL;
    }
    
    // Build final JWT: header.payload.signature
    size_t token_len = strlen(header_b64) + strlen(payload_b64) + strlen(sig_b64) + 3;
    char *token = malloc(token_len);
    if (!token) {
        free(header_b64);
        free(payload_b64);
        free(data);
        free(sig_b64);
        return NULL;
    }
    
    snprintf(token, token_len, "%s.%s.%s", header_b64, payload_b64, sig_b64);
    
    free(header_b64);
    free(payload_b64);
    free(data);
    free(sig_b64);
    
    return token; // Caller must free
}

eth_error_t jwt_verify(const char *token, const char *secret, char **user_id, bool *is_admin) {
    // Split token into header.payload.signature
    char *token_copy = strdup(token);
    if (!token_copy) return ETH_ERROR_AUTH;
    
    char *header_b64 = strtok(token_copy, ".");
    char *payload_b64 = strtok(NULL, ".");
    char *signature_b64 = strtok(NULL, ".");
    
    if (!header_b64 || !payload_b64 || !signature_b64) {
        free(token_copy);
        return ETH_ERROR_AUTH;
    }
    
    // Verify signature
    size_t data_len = strlen(header_b64) + strlen(payload_b64) + 2;
    char *data = malloc(data_len);
    if (!data) {
        free(token_copy);
        return ETH_ERROR_AUTH;
    }
    snprintf(data, data_len, "%s.%s", header_b64, payload_b64);
    
    unsigned char expected_sig[EVP_MAX_MD_SIZE];
    unsigned int sig_len = 0;
    HMAC(EVP_sha256(), secret, (int)strlen(secret),
         (unsigned char *)data, strlen(data), expected_sig, &sig_len);
    
    char *expected_sig_b64 = base64url_encode(expected_sig, sig_len);
    free(data);
    
    if (!expected_sig_b64 || strcmp(signature_b64, expected_sig_b64) != 0) {
        free(token_copy);
        free(expected_sig_b64);
        return ETH_ERROR_AUTH;
    }
    free(expected_sig_b64);
    
    // Decode payload
    size_t payload_len = 0;
    unsigned char *payload = base64url_decode(payload_b64, &payload_len);
    free(token_copy);
    
    if (!payload) return ETH_ERROR_AUTH;
    
    // Parse JSON payload (simple parsing for exp, sub, admin)
    char *payload_str = malloc(payload_len + 1);
    if (!payload_str) {
        free(payload);
        return ETH_ERROR_AUTH;
    }
    memcpy(payload_str, payload, payload_len);
    payload_str[payload_len] = '\0';
    free(payload);
    
    // Check expiration
    char *exp_str = strstr(payload_str, "\"exp\":");
    if (exp_str) {
        long exp = atol(exp_str + 6);
        if (exp < (long)time(NULL)) {
            free(payload_str);
            return ETH_ERROR_AUTH;
        }
    }
    
    // Extract sub (user_id)
    if (user_id) {
        char *sub_str = strstr(payload_str, "\"sub\":\"");
        if (sub_str) {
            sub_str += 7; // Skip "sub":"
            char *end = strchr(sub_str, '"');
            if (end) {
                size_t len = (size_t)(end - sub_str);
                *user_id = malloc(len + 1);
                if (*user_id) {
                    memcpy(*user_id, sub_str, len);
                    (*user_id)[len] = '\0';
                }
            }
        }
    }
    
    // Extract admin
    if (is_admin) {
        char *admin_str = strstr(payload_str, "\"admin\":\"true\"");
        *is_admin = (admin_str != NULL);
    }
    
    free(payload_str);
    return ETH_OK;
}
