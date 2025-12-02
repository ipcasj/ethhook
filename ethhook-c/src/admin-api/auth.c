#define _GNU_SOURCE
#include "ethhook/admin_api.h"
#include <jwt.h>
#include <string.h>
#include <stdlib.h>
#include <time.h>

char *jwt_create(const char *user_id, bool is_admin, const char *secret, int expiry_hours) {
    jwt_t *jwt = NULL;
    
    if (jwt_new(&jwt) != 0) {
        return NULL;
    }
    
    // Set algorithm
    if (jwt_set_alg(jwt, JWT_ALG_HS256, (unsigned char *)secret, strlen(secret)) != 0) {
        jwt_free(jwt);
        return NULL;
    }
    
    // Set claims
    jwt_add_grant(jwt, "sub", user_id);
    jwt_add_grant(jwt, "admin", is_admin ? "true" : "false");
    
    // Set expiration
    time_t now = time(NULL);
    time_t exp = now + (expiry_hours * 3600);
    jwt_add_grant_int(jwt, "exp", exp);
    jwt_add_grant_int(jwt, "iat", now);
    
    // Encode
    char *token = jwt_encode_str(jwt);
    jwt_free(jwt);
    
    return token; // Caller must free
}

eth_error_t jwt_verify(const char *token, const char *secret, char **user_id, bool *is_admin) {
    jwt_t *jwt = NULL;
    
    if (jwt_decode(&jwt, token, (unsigned char *)secret, strlen(secret)) != 0) {
        return ETH_ERROR_AUTH;
    }
    
    // Check expiration
    time_t exp = jwt_get_grant_int(jwt, "exp");
    time_t now = time(NULL);
    
    if (exp < now) {
        jwt_free(jwt);
        return ETH_ERROR_AUTH;
    }
    
    // Extract claims
    const char *sub = jwt_get_grant(jwt, "sub");
    if (sub && user_id) {
        *user_id = strdup(sub);
    }
    
    const char *admin = jwt_get_grant(jwt, "admin");
    if (admin && is_admin) {
        *is_admin = (strcmp(admin, "true") == 0);
    }
    
    jwt_free(jwt);
    return ETH_OK;
}
