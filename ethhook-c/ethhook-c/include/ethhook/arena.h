/**
 * @file arena.h
 * @brief Arena Memory Allocator - Fast, predictable memory management
 *
 * Arena allocation provides:
 * - O(1) allocation (just bump a pointer)
 * - O(1) deallocation (free entire arena at once)
 * - Zero fragmentation
 * - Excellent cache locality
 * - Thread-safe per-arena
 *
 * Perfect for request/response cycles, temporary buffers, and structured data.
 *
 * Usage:
 * @code
 * arena_t *arena = arena_create(1024 * 1024);  // 1MB arena
 * char *buf = arena_alloc(arena, 256);
 * json_object_t *obj = arena_alloc(arena, sizeof(json_object_t));
 * // No need to free individual allocations
 * arena_destroy(arena);  // Free everything at once
 * @endcode
 */

#ifndef ETHHOOK_ARENA_H
#define ETHHOOK_ARENA_H

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/** Opaque arena structure */
typedef struct arena arena_t;

/** Arena statistics for monitoring */
typedef struct {
    size_t total_capacity;    /**< Total arena capacity in bytes */
    size_t bytes_used;        /**< Bytes currently allocated */
    size_t bytes_available;   /**< Bytes still available */
    size_t num_allocations;   /**< Number of allocations made */
    size_t peak_usage;        /**< Peak memory usage */
} arena_stats_t;

/**
 * Create a new arena with specified capacity
 *
 * @param capacity Initial capacity in bytes (will be page-aligned)
 * @return Pointer to new arena, or NULL on failure
 *
 * Example:
 * @code
 * arena_t *arena = arena_create(4096);  // 4KB arena
 * if (!arena) {
 *     perror("Failed to create arena");
 *     return -1;
 * }
 * @endcode
 */
arena_t *arena_create(size_t capacity);

/**
 * Allocate memory from arena
 *
 * @param arena Arena to allocate from
 * @param size Number of bytes to allocate
 * @return Pointer to allocated memory, or NULL if arena is full
 *
 * Note: Memory is automatically aligned to 8-byte boundaries
 *
 * Example:
 * @code
 * char *buf = arena_alloc(arena, 256);
 * if (!buf) {
 *     fprintf(stderr, "Arena exhausted\n");
 *     return -1;
 * }
 * @endcode
 */
void *arena_alloc(arena_t *arena, size_t size);

/**
 * Allocate and zero-initialize memory from arena
 *
 * @param arena Arena to allocate from
 * @param size Number of bytes to allocate
 * @return Pointer to zero-initialized memory, or NULL if arena is full
 */
void *arena_calloc(arena_t *arena, size_t size);

/**
 * Allocate aligned memory from arena
 *
 * @param arena Arena to allocate from
 * @param size Number of bytes to allocate
 * @param alignment Required alignment (must be power of 2)
 * @return Pointer to aligned memory, or NULL on failure
 */
void *arena_alloc_aligned(arena_t *arena, size_t size, size_t alignment);

/**
 * Duplicate a string in the arena
 *
 * @param arena Arena to allocate from
 * @param str String to duplicate (null-terminated)
 * @return Pointer to duplicated string, or NULL on failure
 */
char *arena_strdup(arena_t *arena, const char *str);

/**
 * Duplicate a string with length limit
 *
 * @param arena Arena to allocate from
 * @param str String to duplicate
 * @param n Maximum number of bytes to copy
 * @return Pointer to duplicated string, or NULL on failure
 */
char *arena_strndup(arena_t *arena, const char *str, size_t n);

/**
 * Reset arena (free all allocations, keep memory)
 *
 * @param arena Arena to reset
 *
 * This is faster than destroy + create because it reuses the memory.
 * Useful for processing multiple requests with the same arena.
 */
void arena_reset(arena_t *arena);

/**
 * Get arena statistics
 *
 * @param arena Arena to query
 * @param stats Pointer to stats structure to fill
 */
void arena_get_stats(arena_t *arena, arena_stats_t *stats);

/**
 * Check if arena can satisfy allocation
 *
 * @param arena Arena to check
 * @param size Required size in bytes
 * @return true if allocation would succeed, false otherwise
 */
bool arena_can_alloc(arena_t *arena, size_t size);

/**
 * Destroy arena and free all memory
 *
 * @param arena Arena to destroy (can be NULL)
 *
 * Safe to call with NULL pointer.
 */
void arena_destroy(arena_t *arena);

/**
 * Create a temporary arena with automatic cleanup (for RAII-style patterns)
 *
 * Use with cleanup attribute for automatic destruction:
 * @code
 * void process_request(void) {
 *     arena_t *arena __attribute__((cleanup(arena_auto_destroy))) = arena_create(4096);
 *     // Arena automatically destroyed when function exits
 * }
 * @endcode
 */
void arena_auto_destroy(arena_t **arena);

#ifdef __cplusplus
}
#endif

#endif /* ETHHOOK_ARENA_H */
