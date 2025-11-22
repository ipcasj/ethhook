/**
 * @file arena.c
 * @brief Arena Memory Allocator Implementation
 *
 * Single Translation Unit - all implementation in one file.
 */

#include "ethhook/arena.h"
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <assert.h>
#include <unistd.h>
#include <sys/mman.h>

/* ========================================================================
 * Internal Structures
 * ======================================================================== */

/** Default alignment for allocations (8 bytes for 64-bit pointers) */
#define ARENA_DEFAULT_ALIGN 8

/** Minimum arena size (4KB - one page) */
#define ARENA_MIN_SIZE 4096

/** Round up to next multiple of alignment */
#define ALIGN_UP(n, align) (((n) + (align) - 1) & ~((align) - 1))

/** Check if value is power of 2 */
#define IS_POWER_OF_2(x) (((x) != 0) && (((x) & ((x) - 1)) == 0))

/** Arena structure (private) */
struct arena {
    uint8_t *base;           /**< Start of arena memory */
    uint8_t *cursor;         /**< Current allocation position */
    uint8_t *end;            /**< End of arena memory */
    size_t capacity;         /**< Total capacity in bytes */
    size_t num_allocations;  /**< Number of allocations made */
    size_t peak_usage;       /**< Peak memory usage */
};

/* ========================================================================
 * Public API Implementation
 * ======================================================================== */

arena_t *arena_create(size_t capacity) {
    /* Enforce minimum size */
    if (capacity < ARENA_MIN_SIZE) {
        capacity = ARENA_MIN_SIZE;
    }

    /* Align capacity to page size for mmap efficiency */
    const size_t page_size = (size_t)sysconf(_SC_PAGESIZE);
    capacity = ALIGN_UP(capacity, page_size);

    /* Allocate arena descriptor on heap (small, persistent) */
    arena_t *arena = (arena_t *)malloc(sizeof(arena_t));
    if (!arena) {
        return NULL;
    }

    /* Allocate arena memory using mmap for large, page-aligned allocations
     * This is faster than malloc for large blocks and returns memory to OS */
    void *mem = mmap(NULL, capacity, PROT_READ | PROT_WRITE,
                     MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    if (mem == MAP_FAILED) {
        free(arena);
        return NULL;
    }

    /* Initialize arena */
    arena->base = (uint8_t *)mem;
    arena->cursor = arena->base;
    arena->end = arena->base + capacity;
    arena->capacity = capacity;
    arena->num_allocations = 0;
    arena->peak_usage = 0;

    return arena;
}

void *arena_alloc(arena_t *arena, size_t size) {
    if (!arena || size == 0) {
        return NULL;
    }

    /* Align size to default alignment */
    size = ALIGN_UP(size, ARENA_DEFAULT_ALIGN);

    /* Check if we have enough space */
    const size_t available = (size_t)(arena->end - arena->cursor);
    if (size > available) {
        return NULL;  /* Arena exhausted */
    }

    /* Bump allocate */
    void *ptr = arena->cursor;
    arena->cursor += size;
    arena->num_allocations++;

    /* Update peak usage */
    const size_t current_usage = (size_t)(arena->cursor - arena->base);
    if (current_usage > arena->peak_usage) {
        arena->peak_usage = current_usage;
    }

    return ptr;
}

void *arena_calloc(arena_t *arena, size_t size) {
    void *ptr = arena_alloc(arena, size);
    if (ptr) {
        memset(ptr, 0, size);
    }
    return ptr;
}

void *arena_alloc_aligned(arena_t *arena, size_t size, size_t alignment) {
    if (!arena || size == 0 || !IS_POWER_OF_2(alignment)) {
        return NULL;
    }

    /* Calculate aligned cursor position */
    const uintptr_t cursor_addr = (uintptr_t)arena->cursor;
    const uintptr_t aligned_addr = ALIGN_UP(cursor_addr, alignment);
    const size_t padding = aligned_addr - cursor_addr;

    /* Check if we have enough space (including padding) */
    const size_t total_size = padding + size;
    const size_t available = (size_t)(arena->end - arena->cursor);
    if (total_size > available) {
        return NULL;
    }

    /* Bump allocate with alignment */
    arena->cursor = (uint8_t *)aligned_addr;
    void *ptr = arena->cursor;
    arena->cursor += size;
    arena->num_allocations++;

    /* Update peak usage */
    const size_t current_usage = (size_t)(arena->cursor - arena->base);
    if (current_usage > arena->peak_usage) {
        arena->peak_usage = current_usage;
    }

    return ptr;
}

char *arena_strdup(arena_t *arena, const char *str) {
    if (!arena || !str) {
        return NULL;
    }

    const size_t len = strlen(str);
    char *copy = (char *)arena_alloc(arena, len + 1);
    if (copy) {
        memcpy(copy, str, len + 1);  /* Include null terminator */
    }
    return copy;
}

char *arena_strndup(arena_t *arena, const char *str, size_t n) {
    if (!arena || !str) {
        return NULL;
    }

    /* Find actual length (may be less than n) */
    size_t len = 0;
    while (len < n && str[len] != '\0') {
        len++;
    }

    char *copy = (char *)arena_alloc(arena, len + 1);
    if (copy) {
        memcpy(copy, str, len);
        copy[len] = '\0';
    }
    return copy;
}

void arena_reset(arena_t *arena) {
    if (!arena) {
        return;
    }

    /* Reset cursor to beginning */
    arena->cursor = arena->base;
    arena->num_allocations = 0;
    /* Keep peak_usage for monitoring */

    /* Optional: zero memory for security (disabled for performance)
     * memset(arena->base, 0, arena->capacity);
     */
}

void arena_get_stats(arena_t *arena, arena_stats_t *stats) {
    if (!arena || !stats) {
        return;
    }

    const size_t used = (size_t)(arena->cursor - arena->base);
    stats->total_capacity = arena->capacity;
    stats->bytes_used = used;
    stats->bytes_available = arena->capacity - used;
    stats->num_allocations = arena->num_allocations;
    stats->peak_usage = arena->peak_usage;
}

bool arena_can_alloc(arena_t *arena, size_t size) {
    if (!arena) {
        return false;
    }

    size = ALIGN_UP(size, ARENA_DEFAULT_ALIGN);
    const size_t available = (size_t)(arena->end - arena->cursor);
    return size <= available;
}

void arena_destroy(arena_t *arena) {
    if (!arena) {
        return;
    }

    /* Unmap arena memory */
    if (arena->base) {
        munmap(arena->base, arena->capacity);
    }

    /* Free arena descriptor */
    free(arena);
}

void arena_auto_destroy(arena_t **arena_ptr) {
    if (arena_ptr && *arena_ptr) {
        arena_destroy(*arena_ptr);
        *arena_ptr = NULL;
    }
}

/* ========================================================================
 * Unit Tests (compiled only in test builds)
 * ======================================================================== */

#ifdef ARENA_ENABLE_TESTS

#include <stdio.h>

static int test_basic_allocation(void) {
    arena_t *arena = arena_create(4096);
    assert(arena != NULL);

    void *ptr1 = arena_alloc(arena, 100);
    assert(ptr1 != NULL);

    void *ptr2 = arena_alloc(arena, 200);
    assert(ptr2 != NULL);
    assert(ptr2 > ptr1);

    arena_stats_t stats;
    arena_get_stats(arena, &stats);
    assert(stats.num_allocations == 2);
    assert(stats.bytes_used > 0);

    arena_destroy(arena);
    return 0;
}

static int test_alignment(void) {
    arena_t *arena = arena_create(4096);

    void *ptr = arena_alloc_aligned(arena, 100, 64);
    assert(ptr != NULL);
    assert(((uintptr_t)ptr % 64) == 0);

    arena_destroy(arena);
    return 0;
}

static int test_strdup(void) {
    arena_t *arena = arena_create(4096);

    const char *original = "Hello, World!";
    char *copy = arena_strdup(arena, original);
    assert(copy != NULL);
    assert(strcmp(copy, original) == 0);
    assert(copy != original);

    arena_destroy(arena);
    return 0;
}

static int test_reset(void) {
    arena_t *arena = arena_create(4096);

    arena_alloc(arena, 1000);
    arena_stats_t stats1;
    arena_get_stats(arena, &stats1);
    assert(stats1.bytes_used > 0);

    arena_reset(arena);
    arena_stats_t stats2;
    arena_get_stats(arena, &stats2);
    assert(stats2.bytes_used == 0);

    arena_destroy(arena);
    return 0;
}

int main(void) {
    printf("Running arena tests...\n");

    assert(test_basic_allocation() == 0);
    printf("✓ Basic allocation\n");

    assert(test_alignment() == 0);
    printf("✓ Aligned allocation\n");

    assert(test_strdup() == 0);
    printf("✓ String duplication\n");

    assert(test_reset() == 0);
    printf("✓ Arena reset\n");

    printf("All arena tests passed!\n");
    return 0;
}

#endif /* ARENA_ENABLE_TESTS */
