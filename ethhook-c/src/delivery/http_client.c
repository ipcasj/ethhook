#include "ethhook/delivery.h"
#include <curl/curl.h>
#include <openssl/hmac.h>
#include <string.h>
#include <stdlib.h>

static size_t write_callback(void *contents, size_t size, size_t nmemb, void *userp) {
    (void)contents;
    (void)userp;
    return size * nmemb; // Discard response body
}

eth_error_t http_client_init(http_client_t *client) {
    if (!client) {
        return ETH_ERROR_INVALID_PARAM;
    }
    
    client->curl_handle = curl_easy_init();
    if (!client->curl_handle) {
        return ETH_ERROR_HTTP;
    }
    
    client->user_agent = strdup("EthHook-Delivery/1.0");
    client->circuit_breaker = NULL;
    
    return ETH_OK;
}

void http_client_cleanup(http_client_t *client) {
    if (!client) {
        return;
    }
    
    if (client->curl_handle) {
        curl_easy_cleanup(client->curl_handle);
    }
    
    free(client->user_agent);
}

// Generate HMAC-SHA256 signature
static char *generate_signature(const char *secret, const char *payload, size_t payload_len) {
    unsigned char hash[32];
    unsigned int hash_len = 32;
    
    HMAC(EVP_sha256(), secret, strlen(secret),
         (unsigned char *)payload, payload_len, hash, &hash_len);
    
    // Convert to hex string
    char *hex = malloc(65); // 32 * 2 + 1
    if (!hex) {
        return NULL;
    }
    
    for (unsigned int i = 0; i < hash_len; i++) {
        sprintf(&hex[i * 2], "%02x", hash[i]);
    }
    hex[64] = '\0';
    
    return hex;
}

eth_error_t http_client_post(http_client_t *client, const char *url,
                              const char *payload, size_t payload_len,
                              const char *signature, int timeout_ms,
                              int *http_status) {
    if (!client || !url || !payload) {
        return ETH_ERROR_INVALID_PARAM;
    }
    
    CURL *curl = client->curl_handle;
    
    // Reset curl handle
    curl_easy_reset(curl);
    
    // Set URL
    curl_easy_setopt(curl, CURLOPT_URL, url);
    
    // Set POST data
    curl_easy_setopt(curl, CURLOPT_POSTFIELDS, payload);
    curl_easy_setopt(curl, CURLOPT_POSTFIELDSIZE, payload_len);
    
    // Set headers
    struct curl_slist *headers = NULL;
    headers = curl_slist_append(headers, "Content-Type: application/json");
    
    if (signature) {
        char sig_header[256];
        snprintf(sig_header, sizeof(sig_header), "X-EthHook-Signature: sha256=%s", signature);
        headers = curl_slist_append(headers, sig_header);
    }
    
    if (client->user_agent) {
        curl_easy_setopt(curl, CURLOPT_USERAGENT, client->user_agent);
    }
    
    curl_easy_setopt(curl, CURLOPT_HTTPHEADER, headers);
    
    // Set timeout
    curl_easy_setopt(curl, CURLOPT_TIMEOUT_MS, timeout_ms);
    curl_easy_setopt(curl, CURLOPT_CONNECTTIMEOUT_MS, timeout_ms / 2);
    
    // Set write callback to discard response
    curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, write_callback);
    
    // Follow redirects (max 3)
    curl_easy_setopt(curl, CURLOPT_FOLLOWLOCATION, 1L);
    curl_easy_setopt(curl, CURLOPT_MAXREDIRS, 3L);
    
    // Perform request
    CURLcode res = curl_easy_perform(curl);
    
    curl_slist_free_all(headers);
    
    if (res != CURLE_OK) {
        LOG_ERROR("HTTP POST failed: %s", curl_easy_strerror(res));
        
        if (client->circuit_breaker) {
            circuit_breaker_failure(client->circuit_breaker);
        }
        
        if (res == CURLE_OPERATION_TIMEDOUT) {
            return ETH_ERROR_TIMEOUT;
        }
        
        return ETH_ERROR_HTTP;
    }
    
    // Get HTTP status code
    long status = 0;
    curl_easy_getinfo(curl, CURLINFO_RESPONSE_CODE, &status);
    
    if (http_status) {
        *http_status = (int)status;
    }
    
    // Check if successful (2xx)
    if (status >= 200 && status < 300) {
        if (client->circuit_breaker) {
            circuit_breaker_success(client->circuit_breaker);
        }
        return ETH_OK;
    }
    
    // Failed status code
    if (client->circuit_breaker) {
        circuit_breaker_failure(client->circuit_breaker);
    }
    
    LOG_ERROR("HTTP POST returned status %ld", status);
    return ETH_ERROR_HTTP;
}
