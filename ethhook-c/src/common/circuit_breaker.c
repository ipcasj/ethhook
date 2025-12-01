#define _POSIX_C_SOURCE 199309L
#include "ethhook/common.h"
#include <time.h>
#include <sys/time.h>

void circuit_breaker_init(circuit_breaker_t *cb, uint32_t failure_threshold, uint32_t timeout_ms) {
    atomic_init(&cb->state, CB_STATE_CLOSED);
    atomic_init(&cb->failure_count, 0);
    atomic_init(&cb->success_count, 0);
    atomic_init(&cb->last_failure_time, 0);
    cb->failure_threshold = failure_threshold;
    cb->timeout_ms = timeout_ms;
    cb->half_open_max_calls = 3;
}

static uint64_t get_time_ms(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (uint64_t)ts.tv_sec * 1000 + ts.tv_nsec / 1000000;
}

bool circuit_breaker_allow(circuit_breaker_t *cb) {
    cb_state_t state = atomic_load(&cb->state);
    uint64_t now = get_time_ms();
    
    switch (state) {
        case CB_STATE_CLOSED:
            return true;
            
        case CB_STATE_OPEN: {
            uint64_t last_failure = atomic_load(&cb->last_failure_time);
            if (now - last_failure >= cb->timeout_ms) {
                // Transition to half-open
                if (atomic_compare_exchange_strong(&cb->state, &state, CB_STATE_HALF_OPEN)) {
                    atomic_store(&cb->success_count, 0);
                    atomic_store(&cb->failure_count, 0);
                    return true;
                }
            }
            return false;
        }
            
        case CB_STATE_HALF_OPEN: {
            uint64_t success = atomic_load(&cb->success_count);
            uint64_t failure = atomic_load(&cb->failure_count);
            return (success + failure) < cb->half_open_max_calls;
        }
    }
    
    return false;
}

void circuit_breaker_success(circuit_breaker_t *cb) {
    cb_state_t state = atomic_load(&cb->state);
    
    if (state == CB_STATE_HALF_OPEN) {
        uint64_t success = atomic_fetch_add(&cb->success_count, 1) + 1;
        if (success >= cb->half_open_max_calls) {
            // Transition to closed
            atomic_store(&cb->state, CB_STATE_CLOSED);
            atomic_store(&cb->failure_count, 0);
            atomic_store(&cb->success_count, 0);
        }
    } else if (state == CB_STATE_CLOSED) {
        // Reset failure count on success
        atomic_store(&cb->failure_count, 0);
    }
}

void circuit_breaker_failure(circuit_breaker_t *cb) {
    cb_state_t state = atomic_load(&cb->state);
    uint64_t now = get_time_ms();
    
    atomic_store(&cb->last_failure_time, now);
    
    if (state == CB_STATE_HALF_OPEN) {
        // Immediate transition to open
        atomic_store(&cb->state, CB_STATE_OPEN);
        atomic_store(&cb->failure_count, 0);
        atomic_store(&cb->success_count, 0);
    } else if (state == CB_STATE_CLOSED) {
        uint64_t failures = atomic_fetch_add(&cb->failure_count, 1) + 1;
        if (failures >= cb->failure_threshold) {
            // Transition to open
            atomic_store(&cb->state, CB_STATE_OPEN);
        }
    }
}

cb_state_t circuit_breaker_state(circuit_breaker_t *cb) {
    return atomic_load(&cb->state);
}
