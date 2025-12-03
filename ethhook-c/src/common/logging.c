#define _GNU_SOURCE
#include "ethhook/common.h"
#include <syslog.h>
#include <stdarg.h>
#include <stdio.h>
#include <time.h>

static eth_log_level_t current_log_level = ETH_LOG_INFO;

void eth_log_init(const char *ident) {
    openlog(ident, LOG_PID | LOG_CONS, LOG_USER);
}

void eth_log(eth_log_level_t level, const char *fmt, ...) {
    if (level < current_log_level) {
        return;
    }
    
    va_list args;
    va_start(args, fmt);
    
    int syslog_level;
    const char *level_str;
    
    switch (level) {
        case ETH_LOG_DEBUG:
            syslog_level = LOG_DEBUG;
            level_str = "DEBUG";
            break;
        case ETH_LOG_INFO:
            syslog_level = LOG_INFO;
            level_str = "INFO";
            break;
        case ETH_LOG_WARN:
            syslog_level = LOG_WARNING;
            level_str = "WARN";
            break;
        case ETH_LOG_ERROR:
            syslog_level = LOG_ERR;
            level_str = "ERROR";
            break;
        default:
            syslog_level = LOG_INFO;
            level_str = "INFO";
    }
    
    // Log to syslog
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wformat-nonliteral"
    vsyslog(syslog_level, fmt, args);
    va_end(args);
    
    // Also log to stderr for development
    va_start(args, fmt);
    time_t now = time(NULL);
    struct tm *tm_info = localtime(&now);
    char time_buf[32];
    strftime(time_buf, sizeof(time_buf), "%Y-%m-%d %H:%M:%S", tm_info);
    
    fprintf(stderr, "[%s] %s: ", time_buf, level_str);
    vfprintf(stderr, fmt, args);
#pragma GCC diagnostic pop
    fprintf(stderr, "\n");
    va_end(args);
}
