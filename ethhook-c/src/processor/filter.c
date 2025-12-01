#include "ethhook/processor.h"
#include <string.h>
#include <strings.h>

// Helper to check if topics match
static bool topics_match(char **endpoint_topics, size_t num_endpoint_topics,
                         char **event_topics, size_t num_event_topics) {
    // If no topic filters, match all
    if (num_endpoint_topics == 0) {
        return true;
    }
    
    // Event must have at least as many topics as the filter
    if (num_event_topics < num_endpoint_topics) {
        return false;
    }
    
    // Check each topic filter
    for (size_t i = 0; i < num_endpoint_topics; i++) {
        const char *filter = endpoint_topics[i];
        const char *topic = event_topics[i];
        
        // NULL filter means "any topic"
        if (!filter || strcmp(filter, "") == 0 || strcmp(filter, "null") == 0) {
            continue;
        }
        
        // Exact match (case-insensitive for hex strings)
        if (strcasecmp(filter, topic) != 0) {
            return false;
        }
    }
    
    return true;
}

bool filter_matches(endpoint_t *endpoint, event_t *event) {
    if (!endpoint || !event) {
        return false;
    }
    
    // Check chain ID
    if (endpoint->chain_id != event->chain_id) {
        return false;
    }
    
    // Check address (case-insensitive)
    if (endpoint->address && event->contract_address[0] != '\0') {
        if (strcasecmp(endpoint->address, event->contract_address) != 0) {
            return false;
        }
    }
    
    // Check topics
    if (!topics_match(endpoint->topics, endpoint->num_topics,
                      event->topics, event->num_topics)) {
        return false;
    }
    
    return true;
}
