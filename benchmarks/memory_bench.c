// Memory Usage Benchmark - C Implementation
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <sys/resource.h>

typedef struct {
    int id;
    char data[256];
    struct timespec timestamp;
} Record;

int main(int argc, char *argv[]) {
    int count = 100000;
    if (argc > 1) {
        count = atoi(argv[1]);
    }
    
    struct rusage usage_start, usage_end;
    getrusage(RUSAGE_SELF, &usage_start);
    
    // Allocate array
    Record *records = malloc(count * sizeof(Record));
    if (!records) {
        fprintf(stderr, "Failed to allocate memory\n");
        return 1;
    }
    
    // Initialize records
    for (int i = 0; i < count; i++) {
        records[i].id = i;
        snprintf(records[i].data, sizeof(records[i].data), 
                "Record %d with some data", i);
        clock_gettime(CLOCK_MONOTONIC, &records[i].timestamp);
    }
    
    // Process records (simulate some work)
    long sum = 0;
    for (int i = 0; i < count; i++) {
        sum += records[i].id;
    }
    
    getrusage(RUSAGE_SELF, &usage_end);
    
    long mem_kb = usage_end.ru_maxrss / 1024;
    
    printf("Memory Usage Benchmark (C)\n");
    printf("Records allocated: %d\n", count);
    printf("Record size: %zu bytes\n", sizeof(Record));
    printf("Total allocated: %.2f MB\n", (count * sizeof(Record)) / (1024.0 * 1024.0));
    printf("Max RSS: %ld MB\n", mem_kb / 1024);
    printf("Sum check: %ld\n", sum);
    
    free(records);
    return 0;
}
