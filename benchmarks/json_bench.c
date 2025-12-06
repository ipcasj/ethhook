// JSON Parsing Benchmark - C Implementation
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include "yyjson.h"

const char *sample_event = 
"{"
"  \"id\": 12345,"
"  \"chain_id\": 1,"
"  \"block_number\": 17000000,"
"  \"transaction_hash\": \"0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef\","
"  \"contract_address\": \"0xabcdef1234567890abcdef1234567890abcdef12\","
"  \"event_name\": \"Transfer\","
"  \"event_data\": {"
"    \"from\": \"0x0000000000000000000000000000000000000000\","
"    \"to\": \"0xabcdef1234567890abcdef1234567890abcdef12\","
"    \"value\": \"1000000000000000000\""
"  },"
"  \"timestamp\": 1638360000"
"}";

int main(int argc, char *argv[]) {
    int iterations = 10000;
    if (argc > 1) {
        iterations = atoi(argv[1]);
    }
    
    struct timespec start, end;
    clock_gettime(CLOCK_MONOTONIC, &start);
    
    int parsed = 0;
    for (int i = 0; i < iterations; i++) {
        yyjson_doc *doc = yyjson_read(sample_event, strlen(sample_event), 0);
        if (doc) {
            yyjson_val *root = yyjson_doc_get_root(doc);
            if (root) {
                // Access some fields
                yyjson_val *id = yyjson_obj_get(root, "id");
                yyjson_val *chain = yyjson_obj_get(root, "chain_id");
                yyjson_val *block = yyjson_obj_get(root, "block_number");
                if (id && chain && block) {
                    parsed++;
                }
            }
            yyjson_doc_free(doc);
        }
    }
    
    clock_gettime(CLOCK_MONOTONIC, &end);
    
    double elapsed = (end.tv_sec - start.tv_sec) + (end.tv_nsec - start.tv_nsec) / 1e9;
    double ops_per_sec = iterations / elapsed;
    
    printf("JSON Parsing Benchmark (C - yyjson)\n");
    printf("Iterations: %d\n", iterations);
    printf("Successfully parsed: %d\n", parsed);
    printf("Total time: %.3f seconds\n", elapsed);
    printf("Operations/sec: %.0f\n", ops_per_sec);
    printf("Time per operation: %.3f µs\n", (elapsed / iterations) * 1e6);
    
    return 0;
}
