/**
 * @file log.c
 * @brief Structured logging implementation - Single Translation Unit
 */

#include "ethhook/log.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <pthread.h>
#include <unistd.h>

/* Global logging state */
static struct {
    log_level_t level;
    log_format_t format;
    char service_name[64];
    pthread_mutex_t mutex;
    bool initialized;
} g_log_state = {
    .level = LOG_LEVEL_INFO,
    .format = LOG_FORMAT_TEXT,
    .service_name = "ethhook",
    .mutex = PTHREAD_MUTEX_INITIALIZER,
    .initialized = false
};

/* ========================================================================
 * Internal Helpers
 * ======================================================================== */

static const char *level_to_string(log_level_t level) {
    switch (level) {
        case LOG_LEVEL_DEBUG: return "DEBUG";
        case LOG_LEVEL_INFO:  return "INFO";
        case LOG_LEVEL_WARN:  return "WARN";
        case LOG_LEVEL_ERROR: return "ERROR";
        default:              return "UNKNOWN";
    }
}

static const char *level_to_color(log_level_t level) {
    if (!isatty(STDERR_FILENO)) {
        return "";  /* No colors for non-TTY */
    }

    switch (level) {
        case LOG_LEVEL_DEBUG: return "\033[36m";  /* Cyan */
        case LOG_LEVEL_INFO:  return "\033[32m";  /* Green */
        case LOG_LEVEL_WARN:  return "\033[33m";  /* Yellow */
        case LOG_LEVEL_ERROR: return "\033[31m";  /* Red */
        default:              return "\033[0m";   /* Reset */
    }
}

static void get_iso8601_timestamp(char *buf, size_t size) {
    time_t now = time(NULL);
    struct tm *tm_info = gmtime(&now);
    strftime(buf, size, "%Y-%m-%dT%H:%M:%SZ", tm_info);
}

/* ========================================================================
 * Public API
 * ======================================================================== */

void log_init(log_level_t level, log_format_t format, const char *service_name) {
    pthread_mutex_lock(&g_log_state.mutex);

    g_log_state.level = level;
    g_log_state.format = format;

    if (service_name) {
        snprintf(g_log_state.service_name, sizeof(g_log_state.service_name),
                "%s", service_name);
    }

    g_log_state.initialized = true;

    pthread_mutex_unlock(&g_log_state.mutex);
}

void log_write(log_level_t level, const char *file, int line,
               const char *message, ...) {
    /* Check log level */
    if (level < g_log_state.level) {
        return;
    }

    /* Thread-safe logging */
    pthread_mutex_lock(&g_log_state.mutex);

    char timestamp[32];
    get_iso8601_timestamp(timestamp, sizeof(timestamp));

    if (g_log_state.format == LOG_FORMAT_JSON) {
        /* JSON structured logging */
        fprintf(stderr, "{\"timestamp\":\"%s\",\"level\":\"%s\",\"service\":\"%s\","
                       "\"message\":\"%s\",\"file\":\"%s\",\"line\":%d",
                timestamp, level_to_string(level), g_log_state.service_name,
                message, file, line);

        /* Add structured fields */
        va_list args;
        va_start(args, message);

        const char *key;
        while ((key = va_arg(args, const char *)) != NULL) {
            const char *value = va_arg(args, const char *);
            fprintf(stderr, ",\"%s\":\"%s\"", key, value);
        }

        va_end(args);
        fprintf(stderr, "}\n");

    } else {
        /* Human-readable text logging */
        const char *color = level_to_color(level);
        const char *reset = isatty(STDERR_FILENO) ? "\033[0m" : "";

        fprintf(stderr, "%s[%s] %s%s - %s (%s:%d)%s",
                color, timestamp, g_log_state.service_name,
                level_to_string(level), message, file, line, reset);

        /* Add structured fields */
        va_list args;
        va_start(args, message);

        const char *key;
        bool first = true;
        while ((key = va_arg(args, const char *)) != NULL) {
            const char *value = va_arg(args, const char *);
            fprintf(stderr, "%s %s=%s", first ? " [" : ",", key, value);
            first = false;
        }

        if (!first) {
            fprintf(stderr, "]");
        }

        va_end(args);
        fprintf(stderr, "\n");
    }

    fflush(stderr);
    pthread_mutex_unlock(&g_log_state.mutex);
}

void log_set_level(log_level_t level) {
    pthread_mutex_lock(&g_log_state.mutex);
    g_log_state.level = level;
    pthread_mutex_unlock(&g_log_state.mutex);
}

log_level_t log_get_level(void) {
    return g_log_state.level;
}

log_level_t log_level_from_string(const char *str) {
    if (!str) {
        return LOG_LEVEL_INFO;
    }

    if (strcasecmp(str, "debug") == 0) return LOG_LEVEL_DEBUG;
    if (strcasecmp(str, "info") == 0)  return LOG_LEVEL_INFO;
    if (strcasecmp(str, "warn") == 0)  return LOG_LEVEL_WARN;
    if (strcasecmp(str, "error") == 0) return LOG_LEVEL_ERROR;

    return LOG_LEVEL_INFO;  /* Default */
}
