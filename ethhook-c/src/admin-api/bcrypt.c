/*
 * Simple bcrypt password verification for C
 * Uses Python bcrypt library via system() call for verification
 * This is a temporary solution - in production, use a proper C bcrypt library
 */

#define _GNU_SOURCE
#include <string.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>

bool bcrypt_verify(const char *password, const char *hash) {
    if (!password || !hash) {
        return false;
    }

    // Validate bcrypt hash format: $2[a|b|y]$rounds$salt+hash (60 chars)
    if (strlen(hash) != 60 || strncmp(hash, "$2", 2) != 0) {
        return false;
    }

    // Create a temporary Python script for bcrypt verification
    char cmd[512];
    snprintf(cmd, sizeof(cmd),
        "python3 -c \"import bcrypt; "
        "import sys; "
        "result = bcrypt.checkpw(sys.argv[1].encode(), sys.argv[2].encode()); "
        "sys.exit(0 if result else 1)\" '%s' '%s' 2>/dev/null",
        password, hash);

    int ret = system(cmd);
    return (ret == 0);
}
