# Contributing to ETHhook-C

Thank you for your interest in contributing to ETHhook-C! This document provides guidelines for contributing.

## Getting Started

1. **Fork the repository**
2. **Clone your fork**:
   ```bash
   git clone https://github.com/YOUR_USERNAME/ethhook-c.git
   cd ethhook-c
   ```
3. **Install dependencies**:
   ```bash
   ./scripts/install-deps.sh
   ```
4. **Build the project**:
   ```bash
   make build
   ```
5. **Run tests**:
   ```bash
   make test
   ```

## Development Workflow

### 1. Create a feature branch

```bash
git checkout -b feature/your-feature-name
```

### 2. Make your changes

Follow these guidelines:

- **Code Style**: Use clang-format (run `make format`)
- **Single Translation Unit**: Keep each module in one `.c` file
- **Memory Safety**: Use arena allocation where appropriate
- **Error Handling**: Always check return values
- **Documentation**: Add Doxygen comments for public APIs

### 3. Write tests

```c
// tests/unit/test_your_feature.c
#include <assert.h>
#include "ethhook/your_feature.h"

void test_basic_functionality(void) {
    // Arrange
    your_type_t *obj = your_create();

    // Act
    int result = your_function(obj);

    // Assert
    assert(result == 0);

    // Cleanup
    your_destroy(obj);
}

int main(void) {
    test_basic_functionality();
    printf("All tests passed!\n");
    return 0;
}
```

### 4. Run checks

```bash
# Format code
make format

# Run linter
make lint

# Run tests
make test

# Check for memory leaks
make valgrind
```

### 5. Commit your changes

Use conventional commit messages:

```
feat: Add WebSocket reconnection logic
fix: Fix memory leak in arena allocator
docs: Update API documentation
test: Add tests for event filtering
refactor: Simplify HTTP client code
```

### 6. Push and create a Pull Request

```bash
git push origin feature/your-feature-name
```

Then create a PR on GitHub with:
- Clear description of the change
- Link to any related issues
- Screenshots/examples if applicable

## Code Standards

### Memory Management

✅ **DO**: Use arena allocation for temporary data
```c
arena_t *arena = arena_create(4096);
char *buf = arena_alloc(arena, 256);
// No need to free
arena_destroy(arena);
```

❌ **DON'T**: Use malloc/free for request-scoped data
```c
char *buf = malloc(256);
// Easy to leak
free(buf);
```

### Error Handling

✅ **DO**: Check all return values
```c
arena_t *arena = arena_create(4096);
if (!arena) {
    LOG_ERROR("arena_create_failed");
    return -1;
}
```

❌ **DON'T**: Ignore errors
```c
arena_t *arena = arena_create(4096);
// What if arena is NULL?
char *buf = arena_alloc(arena, 256);
```

### Logging

✅ **DO**: Use structured logging
```c
LOG_INFO("user_login", "user_id", user_id, "ip", ip_address);
```

❌ **DON'T**: Use printf
```c
printf("User %s logged in from %s\n", user_id, ip_address);
```

## Performance Guidelines

1. **Avoid allocations in hot paths**: Use pre-allocated buffers
2. **Batch operations**: Reduce syscalls by batching
3. **Profile before optimizing**: Use perf, valgrind, etc.
4. **Document tradeoffs**: Explain performance-critical code

## Testing

### Unit Tests

Test individual functions in isolation:

```c
void test_arena_alloc(void) {
    arena_t *arena = arena_create(1024);
    void *ptr = arena_alloc(arena, 100);
    assert(ptr != NULL);
    arena_destroy(arena);
}
```

### Integration Tests

Test service interactions:

```bash
# Start dependencies
docker compose up -d postgres redis

# Run integration tests
./build/tests/integration/test_ingestor
```

### Memory Safety

All PRs must be Valgrind clean:

```bash
make valgrind
# Should show: "All heap blocks were freed -- no leaks are possible"
```

## Documentation

- **Public APIs**: Must have Doxygen comments
- **Complex algorithms**: Add explanatory comments
- **README updates**: For new features
- **Architecture docs**: For design changes

## Questions?

- **GitHub Discussions**: https://github.com/ipcasj/ethhook-c/discussions
- **Issues**: https://github.com/ipcasj/ethhook-c/issues
- **Email**: ihorpetroff@gmail.com

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
