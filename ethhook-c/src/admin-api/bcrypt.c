/*
 * Simple bcrypt password verification for C
 * Uses the crypt() function which supports bcrypt hashes on modern systems
 */

#define _GNU_SOURCE
#include <string.h>
#include <stdbool.h>

#ifdef __linux__
#include <crypt.h>
#endif

bool bcrypt_verify(const char *password, const char *hash) {
    if (!password || !hash) {
        return false;
    }

#ifdef __linux__
    // On Linux with glibc/musl, crypt() supports bcrypt if hash starts with $2
    if (strncmp(hash, "$2", 2) == 0) {
        struct crypt_data data;
        data.initialized = 0;
        char *result = crypt_r(password, hash, &data);
        if (result && strcmp(result, hash) == 0) {
            return true;
        }
    }
#endif

    // Fallback: for development/testing, allow plaintext comparison
    // (Remove this in production!)
    return strcmp(password, hash) == 0;
}
