/**
 * @file utils.c
 * @brief Utility functions
 * Single Translation Unit
 */

#include <stdlib.h>
#include <string.h>
#include <ctype.h>

/* String trim */
char *str_trim(char *str) {
    if (!str) return NULL;

    /* Trim leading */
    while (isspace((unsigned char)*str)) str++;

    if (*str == 0) return str;

    /* Trim trailing */
    char *end = str + strlen(str) - 1;
    while (end > str && isspace((unsigned char)*end)) end--;
    end[1] = '\0';

    return str;
}

/* Case-insensitive string comparison */
int str_case_cmp(const char *a, const char *b) {
    if (!a || !b) return -1;
    return strcasecmp(a, b);
}
