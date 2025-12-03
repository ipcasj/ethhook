# Local Development Guide - macOS

## Quick Start

### 1. Install Dependencies

Run the setup script (one-time setup):

```bash
cd /Users/igor/rust_projects/capstone0/ethhook-c
./setup-macos.sh
```

This installs via Homebrew:
- Build tools (CMake, GCC, pkg-config)
- Required libraries (libevent, hiredis, curl, etc.)
- Development tools (cppcheck, clang-format, valgrind)

### 2. Build the Project

**Standard build (optimized):**

```bash
rm -rf build
cmake -B build
cmake --build build -j8
```

**Development build (with sanitizers):**

```bash
rm -rf build
cmake -B build \
  -DENABLE_ASAN=ON \
  -DENABLE_UBSAN=ON \
  -DCMAKE_BUILD_TYPE=Debug
cmake --build build -j8
```

**Thread sanitizer build (detects race conditions):**

```bash
rm -rf build
cmake -B build \
  -DENABLE_TSAN=ON \
  -DCMAKE_BUILD_TYPE=Debug
cmake --build build -j8
```

### 3. Run Services

```bash
# Admin API
./build/ethhook-admin-api config.toml

# Event Ingestor
./build/ethhook-ingestor config.toml

# Message Processor
./build/ethhook-processor config.toml

# Webhook Delivery
./build/ethhook-delivery config.toml
```

## Development Workflow

### Standard Workflow

```bash
# 1. Edit code
vim src/common/clickhouse.c

# 2. Build (incremental)
cmake --build build -j8

# 3. Test locally
./build/ethhook-admin-api --help

# 4. Commit when working
git add -A
git commit -m "fix: your change"
git push
```

### With Sanitizers (Recommended)

```bash
# 1. Build with ASAN+UBSAN
cmake -B build -DENABLE_ASAN=ON -DENABLE_UBSAN=ON
cmake --build build

# 2. Run your service
./build/ethhook-admin-api config.toml

# Sanitizers will print errors if detected:
# - Memory leaks
# - Buffer overflows
# - Use-after-free
# - Undefined behavior
# - Integer overflows
```

### Code Quality Checks

```bash
# Static analysis with cppcheck
cd build
make cppcheck

# Check code formatting
make format-check

# Auto-format code
make format

# Clang static analyzer (if installed)
make analyze
```

## JWT Authentication Implementation

The admin API uses JWT (JSON Web Tokens) for authentication, implemented directly using OpenSSL's HMAC-SHA256 functions. This eliminates external JWT library dependencies while maintaining security and cross-platform compatibility.

### JWT Features

- **Algorithm**: HS256 (HMAC-SHA256)
- **Base64URL encoding**: RFC 4648 compliant
- **Claims**: `sub` (user ID), `admin` (boolean), `exp` (expiration), `iat` (issued at)
- **Security**: Signature verification, expiration checking, tampering detection

### Implementation Details

Located in `src/admin-api/auth.c` (~150 lines):
- `jwt_create()`: Creates signed JWT tokens with expiry
- `jwt_verify()`: Verifies signature and extracts claims
- `base64url_encode()`: Converts binary data to base64url format
- `base64url_decode()`: Decodes base64url to binary

### Example Usage

```c
// Create token (24 hour expiry)
char *token = jwt_create("user_123", 1, "secret_key", 24);
if (!token) {
    fprintf(stderr, "Failed to create JWT\n");
    return -1;
}
printf("Token: %s\n", token);

// Verify token
char *user_id = NULL;
int is_admin = 0;
int result = jwt_verify(token, "secret_key", &user_id, &is_admin);

if (result == 0) {
    printf("Valid token for user: %s (admin: %d)\n", user_id, is_admin);
    free(user_id);
} else {
    printf("Token verification failed: %d\n", result);
}

free(token);
```

### Error Codes

- `0`: Success - token valid
- `-1`: Invalid input or malformed token structure
- `-2`: Invalid signature (tampering detected)
- `-3`: Token expired

### Testing JWT Implementation

All JWT functions are tested on both GCC and Clang. The implementation:
- Creates RFC-compliant JWT tokens
- Verifies HMAC-SHA256 signatures correctly
- Rejects tampered tokens
- Checks token expiration
- Extracts claims properly

## IDE Setup

### VSCode

Install extensions:
- C/C++ (Microsoft)
- CMake Tools
- C/C++ Advanced Lint

Add to `.vscode/settings.json`:

```json
{
  "C_Cpp.default.configurationProvider": "ms-vscode.cmake-tools",
  "cmake.buildDirectory": "${workspaceFolder}/build",
  "C_Cpp.default.compilerPath": "/usr/bin/gcc",
  "C_Cpp.cppcheck.enable": true,
  "editor.formatOnSave": true,
  "[c]": {
    "editor.defaultFormatter": "ms-vscode.cpptools"
  }
}
```

### CLion

1. Open the `ethhook-c` folder
2. CLion will detect CMakeLists.txt automatically
3. Set CMake options in Preferences → Build → CMake:
   - Debug profile: `-DENABLE_ASAN=ON -DENABLE_UBSAN=ON`
   - Release profile: (default)

## Debugging

### GDB (Linux-style)

```bash
# Build with debug symbols
cmake -B build -DCMAKE_BUILD_TYPE=Debug
cmake --build build

# Run with GDB
gdb ./build/ethhook-admin-api
(gdb) break main
(gdb) run config.toml
(gdb) next
(gdb) print variable_name
```

### LLDB (macOS native)

```bash
# Build with debug symbols
cmake -B build -DCMAKE_BUILD_TYPE=Debug
cmake --build build

# Run with LLDB
lldb ./build/ethhook-admin-api
(lldb) breakpoint set --name main
(lldb) run config.toml
(lldb) next
(lldb) print variable_name
```

## Common Issues

### "Library not found" errors

```bash
# Check what libraries are being looked for
otool -L build/ethhook-admin-api

# If Homebrew libraries aren't found, set PKG_CONFIG_PATH:
export PKG_CONFIG_PATH="/opt/homebrew/lib/pkgconfig:/usr/local/lib/pkgconfig"

# Then rebuild
rm -rf build
cmake -B build
cmake --build build
```

### "Command line tools not found"

```bash
xcode-select --install
```

### CMake can't find a library

```bash
# Check if library is installed
brew list | grep libevent

# If not installed
brew install libevent

# If installed but not found, help CMake:
export PKG_CONFIG_PATH="/opt/homebrew/lib/pkgconfig:/usr/local/lib/pkgconfig"
cmake -B build
```

## Performance Comparison: Local vs Server

| Aspect | Local (Mac) | Server (Ubuntu) |
|--------|-------------|-----------------|
| Initial build | ~30s | ~45s |
| Incremental build | 1-3s | 5-10s + SSH |
| Debug cycle | Instant | 30s+ (push/pull) |
| Sanitizers | Native | Via SSH |
| IDE integration | Full | Limited |
| Internet required | No | Yes |

**Recommendation:** Develop locally, test on server for final validation.

## What to Build Where

### Local Development ✅
- Feature development
- Bug fixes
- Unit testing
- Sanitizer testing
- Code refactoring
- Static analysis

### Server Testing ✅
- Final integration testing
- Performance benchmarking
- Production deployment
- Multi-service testing
- Load testing
- Linux-specific issues

## Next Steps

After local setup works:

1. **Test basic compilation:**
   ```bash
   ./setup-macos.sh
   cmake -B build && cmake --build build
   ```

2. **Verify binaries:**
   ```bash
   ls -lh build/ethhook-*
   file build/ethhook-admin-api
   ```

3. **Run with sanitizers:**
   ```bash
   cmake -B build -DENABLE_ASAN=ON
   cmake --build build
   ./build/ethhook-admin-api --help
   ```

4. **Push changes to server for final testing**
