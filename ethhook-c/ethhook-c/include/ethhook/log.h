/**
 * @file log.h
 * @brief Structured logging with JSON output support
 *
 * Provides leveled logging (DEBUG, INFO, WARN, ERROR) with structured fields
 * for cloud-native environments (Kubernetes, Docker).
 *
 * Usage:
 * @code
 * LOG_INFO("server_started", "port", 8080, "version", "1.0.0");
 * LOG_ERROR("connection_failed", "error", error_msg, "retries", 3);
 * @endcode
 */

#ifndef ETHHOOK_LOG_H
#define ETHHOOK_LOG_H

#include <stdarg.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/** Log levels */
typedef enum {
    LOG_LEVEL_DEBUG = 0,
    LOG_LEVEL_INFO  = 1,
    LOG_LEVEL_WARN  = 2,
    LOG_LEVEL_ERROR = 3,
} log_level_t;

/** Log output format */
typedef enum {
    LOG_FORMAT_TEXT,  /**< Human-readable text */
    LOG_FORMAT_JSON,  /**< JSON for machine parsing */
} log_format_t;

/**
 * Initialize logging system
 *
 * @param level Minimum log level to output
 * @param format Output format (text or JSON)
 * @param service_name Name of the service (for structured logging)
 */
void log_init(log_level_t level, log_format_t format, const char *service_name);

/**
 * Log a message with structured fields
 *
 * @param level Log level
 * @param file Source file (use __FILE__)
 * @param line Line number (use __LINE__)
 * @param message Log message
 * @param ... Key-value pairs (must be even number of args, ends with NULL)
 *
 * Example:
 * @code
 * log_write(LOG_LEVEL_INFO, __FILE__, __LINE__, "user_login",
 *          "user_id", "123", "ip", "192.168.1.1", NULL);
 * @endcode
 */
void log_write(log_level_t level, const char *file, int line,
               const char *message, ...);

/** Convenience macros */
#define LOG_DEBUG(msg, ...) \
    log_write(LOG_LEVEL_DEBUG, __FILE__, __LINE__, msg, ##__VA_ARGS__, NULL)

#define LOG_INFO(msg, ...) \
    log_write(LOG_LEVEL_INFO, __FILE__, __LINE__, msg, ##__VA_ARGS__, NULL)

#define LOG_WARN(msg, ...) \
    log_write(LOG_LEVEL_WARN, __FILE__, __LINE__, msg, ##__VA_ARGS__, NULL)

#define LOG_ERROR(msg, ...) \
    log_write(LOG_LEVEL_ERROR, __FILE__, __LINE__, msg, ##__VA_ARGS__, NULL)

/**
 * Set log level at runtime
 */
void log_set_level(log_level_t level);

/**
 * Get current log level
 */
log_level_t log_get_level(void);

/**
 * Parse log level from string
 *
 * @param str Log level string ("debug", "info", "warn", "error")
 * @return Log level, or LOG_LEVEL_INFO if invalid
 */
log_level_t log_level_from_string(const char *str);

#ifdef __cplusplus
}
#endif

#endif /* ETHHOOK_LOG_H */
