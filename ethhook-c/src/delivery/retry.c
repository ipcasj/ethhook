#include "ethhook/delivery.h"
#include <stdlib.h>
#include <math.h>
#include <time.h>

uint64_t retry_calculate_delay(retry_policy_t *policy, uint32_t attempt) {
    if (!policy || attempt == 0) {
        return 0;
    }
    
    // Exponential backoff: base_delay * (multiplier ^ attempt)
    double delay = (double)policy->base_delay_ms * pow(policy->backoff_multiplier, attempt - 1);
    
    // Cap at max delay
    if (delay > policy->max_delay_ms) {
        delay = policy->max_delay_ms;
    }
    
    // Add jitter (±25%)
    srand(time(NULL));
    double jitter = ((double)rand() / RAND_MAX) * 0.5 - 0.25; // -0.25 to +0.25
    delay = delay * (1.0 + jitter);
    
    // Ensure minimum delay
    if (delay < policy->base_delay_ms) {
        delay = policy->base_delay_ms;
    }
    
    return (uint64_t)delay;
}
