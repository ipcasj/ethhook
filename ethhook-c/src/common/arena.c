#include "ethhook/common.h"
#include <stdlib.h>
#include <string.h>
#include <pthread.h>

typedef struct arena_block {
    size_t size;
    size_t used;
    struct arena_block *next;
    char data[];
} arena_block_t;

struct eth_arena {
    arena_block_t *current;
    size_t default_block_size;
    pthread_mutex_t lock;
};

eth_arena_t *eth_arena_create(size_t initial_size) {
    eth_arena_t *arena = malloc(sizeof(eth_arena_t));
    if (!arena) {
        return NULL;
    }
    
    arena->default_block_size = initial_size;
    pthread_mutex_init(&arena->lock, NULL);
    
    arena_block_t *block = malloc(sizeof(arena_block_t) + initial_size);
    if (!block) {
        free(arena);
        return NULL;
    }
    
    block->size = initial_size;
    block->used = 0;
    block->next = NULL;
    arena->current = block;
    
    return arena;
}

void *eth_arena_alloc(eth_arena_t *arena, size_t size) {
    if (!arena || size == 0) {
        return NULL;
    }
    
    // Align to 8 bytes
    size = (size + 7) & ~((size_t)7);
    
    pthread_mutex_lock(&arena->lock);
    
    arena_block_t *block = arena->current;
    
    // Check if current block has space
    if (block->used + size > block->size) {
        // Allocate new block
        size_t block_size = size > arena->default_block_size ? size : arena->default_block_size;
        arena_block_t *new_block = malloc(sizeof(arena_block_t) + block_size);
        if (!new_block) {
            pthread_mutex_unlock(&arena->lock);
            return NULL;
        }
        
        new_block->size = block_size;
        new_block->used = 0;
        new_block->next = block;
        arena->current = new_block;
        block = new_block;
    }
    
    void *ptr = &block->data[block->used];
    block->used += size;
    
    pthread_mutex_unlock(&arena->lock);
    
    return ptr;
}

void eth_arena_reset(eth_arena_t *arena) {
    if (!arena) {
        return;
    }
    
    pthread_mutex_lock(&arena->lock);
    
    // Reset all blocks except the first one
    arena_block_t *block = arena->current;
    arena_block_t *first = NULL;
    
    while (block) {
        if (!block->next) {
            first = block;
        }
        block->used = 0;
        block = block->next;
    }
    
    // Free all blocks except first
    block = arena->current;
    while (block && block != first) {
        arena_block_t *next = block->next;
        free(block);
        block = next;
    }
    
    arena->current = first;
    
    pthread_mutex_unlock(&arena->lock);
}

void eth_arena_destroy(eth_arena_t *arena) {
    if (!arena) {
        return;
    }
    
    arena_block_t *block = arena->current;
    while (block) {
        arena_block_t *next = block->next;
        free(block);
        block = next;
    }
    
    pthread_mutex_destroy(&arena->lock);
    free(arena);
}
