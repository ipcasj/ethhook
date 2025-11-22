/**
 * @file crypto.c
 * @brief Cryptographic functions (HMAC-SHA256, JWT)
 * Single Translation Unit
 */

#include <openssl/hmac.h>
#include <openssl/sha.h>
#include <string.h>
#include <stdio.h>

/**
 * Compute HMAC-SHA256
 *
 * @param key HMAC secret key
 * @param key_len Length of key
 * @param data Data to sign
 * @param data_len Length of data
 * @param out Output buffer (must be at least 32 bytes)
 * @return 0 on success, -1 on error
 */
int crypto_hmac_sha256(const uint8_t *key, size_t key_len,
                       const uint8_t *data, size_t data_len,
                       uint8_t out[32]) {
    unsigned int len = 32;
    if (!HMAC(EVP_sha256(), key, (int)key_len, data, data_len, out, &len)) {
        return -1;
    }
    return 0;
}

/**
 * Constant-time comparison (prevents timing attacks)
 */
int crypto_constant_time_compare(const uint8_t *a, const uint8_t *b, size_t len) {
    uint8_t result = 0;
    for (size_t i = 0; i < len; i++) {
        result |= a[i] ^ b[i];
    }
    return result == 0 ? 0 : -1;
}

/**
 * Hex encode
 */
void crypto_hex_encode(const uint8_t *data, size_t len, char *out) {
    for (size_t i = 0; i < len; i++) {
        sprintf(out + (i * 2), "%02x", data[i]);
    }
    out[len * 2] = '\0';
}
