// JWT Benchmark - C Implementation
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <openssl/hmac.h>
#include <openssl/evp.h>

// Base64url encoding table
static const char base64url_table[] = 
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

static size_t base64url_encode(const unsigned char *input, size_t len, char *output) {
    size_t i = 0, j = 0;
    for (i = 0; i < len - len % 3; i += 3) {
        output[j++] = base64url_table[(input[i] >> 2) & 0x3F];
        output[j++] = base64url_table[((input[i] & 0x03) << 4) | ((input[i+1] >> 4) & 0x0F)];
        output[j++] = base64url_table[((input[i+1] & 0x0F) << 2) | ((input[i+2] >> 6) & 0x03)];
        output[j++] = base64url_table[input[i+2] & 0x3F];
    }
    if (len % 3 == 1) {
        output[j++] = base64url_table[(input[i] >> 2) & 0x3F];
        output[j++] = base64url_table[(input[i] & 0x03) << 4];
    } else if (len % 3 == 2) {
        output[j++] = base64url_table[(input[i] >> 2) & 0x3F];
        output[j++] = base64url_table[((input[i] & 0x03) << 4) | ((input[i+1] >> 4) & 0x0F)];
        output[j++] = base64url_table[(input[i+1] & 0x0F) << 2];
    }
    output[j] = '\0';
    return j;
}

static int jwt_sign_hs256(const char *payload, const char *secret, char *output, size_t output_size) {
    const char *header = "{\"alg\":\"HS256\",\"typ\":\"JWT\"}";
    char header_b64[256];
    char payload_b64[512];
    
    size_t header_len = base64url_encode((const unsigned char *)header, strlen(header), header_b64);
    size_t payload_len = base64url_encode((const unsigned char *)payload, strlen(payload), payload_b64);
    
    char to_sign[1024];
    int to_sign_len = snprintf(to_sign, sizeof(to_sign), "%s.%s", header_b64, payload_b64);
    
    unsigned char signature[EVP_MAX_MD_SIZE];
    unsigned int sig_len = 0;
    
    HMAC(EVP_sha256(), secret, (int)strlen(secret),
         (unsigned char *)to_sign, to_sign_len,
         signature, &sig_len);
    
    char sig_b64[256];
    base64url_encode(signature, sig_len, sig_b64);
    
    snprintf(output, output_size, "%s.%s", to_sign, sig_b64);
    return 0;
}

int main(int argc, char *argv[]) {
    int iterations = 100000;
    if (argc > 1) {
        iterations = atoi(argv[1]);
    }
    
    const char *secret = "your-256-bit-secret";
    const char *payload = "{\"sub\":\"1234567890\",\"name\":\"John Doe\",\"iat\":1516239022}";
    
    char jwt[2048];
    
    struct timespec start, end;
    clock_gettime(CLOCK_MONOTONIC, &start);
    
    for (int i = 0; i < iterations; i++) {
        jwt_sign_hs256(payload, secret, jwt, sizeof(jwt));
    }
    
    clock_gettime(CLOCK_MONOTONIC, &end);
    
    double elapsed = (end.tv_sec - start.tv_sec) + (end.tv_nsec - start.tv_nsec) / 1e9;
    double ops_per_sec = iterations / elapsed;
    
    printf("JWT Signing Benchmark (C)\n");
    printf("Iterations: %d\n", iterations);
    printf("Total time: %.3f seconds\n", elapsed);
    printf("Operations/sec: %.0f\n", ops_per_sec);
    printf("Time per operation: %.3f µs\n", (elapsed / iterations) * 1e6);
    
    return 0;
}
