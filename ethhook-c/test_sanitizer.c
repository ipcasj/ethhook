#include <stdlib.h>
#include <string.h>

int main(void) {
    // Test 1: Simple allocation and free (should pass)
    char *buf = malloc(100);
    if (buf) {
        strcpy(buf, "Hello, ASAN!");
        free(buf);
    }
    
    // Test 2: Use-after-free (ASAN should catch this)
    // Uncomment to test: buf[0] = 'x';
    
    // Test 3: Buffer overflow (ASAN should catch this)
    // Uncomment to test: buf[100] = 'x';
    
    return 0;
}
