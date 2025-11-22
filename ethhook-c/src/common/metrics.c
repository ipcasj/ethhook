/**
 * @file metrics.c
 * @brief Prometheus metrics exposition
 * Single Translation Unit
 */

#include <stdio.h>
#include <stdint.h>
#include <pthread.h>
#include <string.h>

/* Simple in-memory metrics store */
typedef struct {
    char name[128];
    char help[256];
    uint64_t value;
} metric_t;

#define MAX_METRICS 1024
static metric_t g_metrics[MAX_METRICS];
static size_t g_num_metrics = 0;
static pthread_mutex_t g_metrics_mutex = PTHREAD_MUTEX_INITIALIZER;

/**
 * Increment a counter metric
 */
void metrics_counter_inc(const char *name) {
    pthread_mutex_lock(&g_metrics_mutex);

    /* Find existing metric */
    for (size_t i = 0; i < g_num_metrics; i++) {
        if (strcmp(g_metrics[i].name, name) == 0) {
            g_metrics[i].value++;
            pthread_mutex_unlock(&g_metrics_mutex);
            return;
        }
    }

    /* Create new metric */
    if (g_num_metrics < MAX_METRICS) {
        strncpy(g_metrics[g_num_metrics].name, name, sizeof(g_metrics[0].name) - 1);
        g_metrics[g_num_metrics].value = 1;
        g_num_metrics++;
    }

    pthread_mutex_unlock(&g_metrics_mutex);
}

/**
 * Set a gauge metric
 */
void metrics_gauge_set(const char *name, uint64_t value) {
    pthread_mutex_lock(&g_metrics_mutex);

    for (size_t i = 0; i < g_num_metrics; i++) {
        if (strcmp(g_metrics[i].name, name) == 0) {
            g_metrics[i].value = value;
            pthread_mutex_unlock(&g_metrics_mutex);
            return;
        }
    }

    if (g_num_metrics < MAX_METRICS) {
        strncpy(g_metrics[g_num_metrics].name, name, sizeof(g_metrics[0].name) - 1);
        g_metrics[g_num_metrics].value = value;
        g_num_metrics++;
    }

    pthread_mutex_unlock(&g_metrics_mutex);
}

/**
 * Export metrics in Prometheus format
 */
void metrics_export(char *buf, size_t buf_size) {
    pthread_mutex_lock(&g_metrics_mutex);

    size_t offset = 0;
    for (size_t i = 0; i < g_num_metrics && offset < buf_size; i++) {
        offset += snprintf(buf + offset, buf_size - offset,
                          "# HELP %s %s\n%s %lu\n",
                          g_metrics[i].name, g_metrics[i].help,
                          g_metrics[i].name, g_metrics[i].value);
    }

    pthread_mutex_unlock(&g_metrics_mutex);
}
