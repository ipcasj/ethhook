cmake_minimum_required(VERSION 3.20)
project(ethhook C)

set(CMAKE_C_STANDARD 17)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_C_EXTENSIONS OFF)

# Modern C development options
option(ENABLE_ASAN "Enable AddressSanitizer for memory error detection" OFF)
option(ENABLE_UBSAN "Enable UndefinedBehaviorSanitizer" OFF)
option(ENABLE_TSAN "Enable ThreadSanitizer for race condition detection" OFF)
option(ENABLE_STATIC_ANALYSIS "Enable additional static analysis warnings" ON)

# Compiler flags - aggressive warnings for code quality (cross-platform compatible)
add_compile_options(
    -Wall                         # Enable all common warnings
    -Wextra                       # Enable extra warnings
    -Wpedantic                    # Strict ISO C compliance
    -Werror                       # Treat warnings as errors
    -Wformat=2                    # Enhanced format string checking
    -Wformat-security             # Warn about insecure format strings
    -Wnull-dereference            # Warn about potential NULL dereferences
    -Wwrite-strings               # Warn about writing to string constants
    -Wshadow                      # Warn about variable shadowing
    -fstack-protector-strong      # Stack canaries for buffer overflow protection
)

# _FORTIFY_SOURCE conflicts with sanitizers, only enable for non-sanitizer builds
if(NOT ENABLE_ASAN AND NOT ENABLE_UBSAN AND NOT ENABLE_TSAN)
    add_compile_options(-D_FORTIFY_SOURCE=2)  # Runtime buffer overflow detection
endif()

# Platform-specific warning flags
if(CMAKE_C_COMPILER_ID STREQUAL "GNU")
    # GCC-specific warnings
    add_compile_options(
        -Warray-bounds=2          # Enhanced array bounds checking (GCC only)
        -Wstrict-overflow=2       # Warn about overflow in comparisons
    )
elseif(CMAKE_C_COMPILER_ID MATCHES "Clang")
    # Clang/AppleClang-specific adjustments
    add_compile_options(
        -Wstrict-overflow=2       # Warn about overflow in comparisons
    )
endif()

# Optimization flags (excluding sanitizers which conflict with -O3)
if(NOT ENABLE_ASAN AND NOT ENABLE_UBSAN AND NOT ENABLE_TSAN)
    add_compile_options(
        -O3
        -march=native
        -fno-strict-aliasing
    )
else()
    add_compile_options(
        -O1                       # Moderate optimization for sanitizers
        -g                        # Debug symbols for better error reports
        -fno-omit-frame-pointer   # Keep frame pointers for stack traces
        -fno-optimize-sibling-calls
    )
endif()

# AddressSanitizer - detects memory errors (use-after-free, buffer overflows, leaks)
if(ENABLE_ASAN)
    message(STATUS "AddressSanitizer enabled")
    add_compile_options(-fsanitize=address)
    add_link_options(-fsanitize=address)
endif()

# UndefinedBehaviorSanitizer - detects undefined behavior
if(ENABLE_UBSAN)
    message(STATUS "UndefinedBehaviorSanitizer enabled")
    add_compile_options(-fsanitize=undefined)
    add_link_options(-fsanitize=undefined)
endif()

# ThreadSanitizer - detects data races (cannot combine with ASAN)
if(ENABLE_TSAN)
    if(ENABLE_ASAN)
        message(FATAL_ERROR "ThreadSanitizer and AddressSanitizer cannot be used together")
    endif()
    message(STATUS "ThreadSanitizer enabled")
    add_compile_options(-fsanitize=thread)
    add_link_options(-fsanitize=thread)
endif()

# Enable LTO for release builds (when not using sanitizers)
if(NOT ENABLE_ASAN AND NOT ENABLE_UBSAN AND NOT ENABLE_TSAN)
    include(CheckIPOSupported)
    check_ipo_supported(RESULT IPO_SUPPORTED)
    if(IPO_SUPPORTED)
        set(CMAKE_INTERPROCEDURAL_OPTIMIZATION_RELEASE ON)
        message(STATUS "Link-time optimization (LTO) enabled")
    endif()
endif()

# Find or fetch dependencies
find_package(PkgConfig REQUIRED)

# SQLite3
find_package(SQLite3 REQUIRED)

# libevent
pkg_check_modules(LIBEVENT REQUIRED libevent>=2.1)

# libwebsockets
pkg_check_modules(LIBWEBSOCKETS REQUIRED libwebsockets>=4.0)

# hiredis (Ubuntu 22.04 has 0.14.1, which is sufficient)
pkg_check_modules(HIREDIS REQUIRED hiredis>=0.14)

# libcurl
pkg_check_modules(LIBCURL REQUIRED libcurl>=7.68)

# libmicrohttpd
pkg_check_modules(LIBMICROHTTPD REQUIRED libmicrohttpd>=0.9.70)

# Add library directories for pkg-config managed libraries
# MUST be after all pkg_check_modules calls
link_directories(
    ${HIREDIS_LIBRARY_DIRS}
    ${LIBWEBSOCKETS_LIBRARY_DIRS}
    ${LIBEVENT_LIBRARY_DIRS}
    ${LIBCURL_LIBRARY_DIRS}
    ${LIBMICROHTTPD_LIBRARY_DIRS}
)

# yyjson (single-header library, included in source)
# Download if not present
if(NOT EXISTS "${CMAKE_SOURCE_DIR}/include/yyjson.h")
    message(STATUS "Downloading yyjson...")
    file(DOWNLOAD
        "https://raw.githubusercontent.com/ibireme/yyjson/master/src/yyjson.h"
        "${CMAKE_SOURCE_DIR}/include/yyjson.h"
        SHOW_PROGRESS)
    file(DOWNLOAD
        "https://raw.githubusercontent.com/ibireme/yyjson/master/src/yyjson.c"
        "${CMAKE_SOURCE_DIR}/src/common/yyjson.c"
        SHOW_PROGRESS)
endif()

# OpenSSL (for crypto and JWT signing)
# Note: We implement JWT directly using OpenSSL's HMAC instead of libjwt
# This is more portable and avoids dependency on libjwt v3 API changes
find_package(OpenSSL REQUIRED)

# Threads
find_package(Threads REQUIRED)

# Include directories
include_directories(
    ${CMAKE_SOURCE_DIR}/include
    ${LIBEVENT_INCLUDE_DIRS}
    ${LIBWEBSOCKETS_INCLUDE_DIRS}
    ${HIREDIS_INCLUDE_DIRS}
    ${LIBCURL_INCLUDE_DIRS}
    ${LIBMICROHTTPD_INCLUDE_DIRS}
    ${OPENSSL_INCLUDE_DIR}
    ${SQLite3_INCLUDE_DIRS}
)

# Optional: io_uring support (Linux only, requires kernel 5.1+)
option(ENABLE_IO_URING "Enable io_uring for zero-copy I/O (Linux only)" ON)
if(ENABLE_IO_URING AND CMAKE_SYSTEM_NAME STREQUAL "Linux")
    pkg_check_modules(LIBURING liburing>=2.0)
    if(LIBURING_FOUND)
        add_compile_definitions(HAVE_IO_URING)
        message(STATUS "io_uring support enabled")
    else()
        message(STATUS "liburing not found, using libevent fallback")
    endif()
else()
    message(STATUS "io_uring disabled (not Linux or explicitly disabled)")
endif()

# Common library
add_library(ethhook-common STATIC
    src/common/config.c
    src/common/error.c
    src/common/logging.c
    src/common/arena.c
    src/common/circuit_breaker.c
    src/common/database.c
    src/common/clickhouse.c
    src/common/yyjson.c
    src/common/json.c
)

target_link_libraries(ethhook-common
    ${SQLite3_LIBRARIES}
    ${LIBEVENT_LIBRARIES}
    ${LIBCURL_LIBRARIES}
    Threads::Threads
    m
)

# Event Ingestor
add_executable(ethhook-ingestor
    src/ingestor/main.c
    src/ingestor/websocket.c
    src/ingestor/worker.c
    src/ingestor/redis_publisher.c
)

target_link_libraries(ethhook-ingestor
    ethhook-common
    ${LIBWEBSOCKETS_LIBRARIES}
    ${LIBEVENT_LIBRARIES}
    ${HIREDIS_LIBRARIES}
    ${OpenSSL_LIBRARIES}
    Threads::Threads
)

# Message Processor
add_executable(ethhook-processor
    src/processor/main.c
    src/processor/redis_consumer.c
    src/processor/matcher.c
    src/processor/filter.c
)

target_link_libraries(ethhook-processor
    ethhook-common
    ${HIREDIS_LIBRARIES}
    ${LIBEVENT_LIBRARIES}
    Threads::Threads
)

# Webhook Delivery
add_executable(ethhook-delivery
    src/delivery/main.c
    src/delivery/http_client.c
    src/delivery/retry.c
    src/delivery/worker.c
)

target_link_libraries(ethhook-delivery
    ethhook-common
    ${LIBCURL_LIBRARIES}
    ${HIREDIS_LIBRARIES}
    ${LIBEVENT_LIBRARIES}
    Threads::Threads
)

# Admin API
add_executable(ethhook-admin-api
    src/admin-api/main.c
    src/admin-api/handlers.c
    src/admin-api/auth.c
    src/admin-api/routes.c
    src/admin-api/json_response.c
    src/admin-api/bcrypt.c
)

target_link_libraries(ethhook-admin-api
    ethhook-common
    ${LIBMICROHTTPD_LIBRARIES}
    OpenSSL::SSL
    OpenSSL::Crypto
    Threads::Threads
    crypt
)

# Install targets
install(TARGETS 
    ethhook-ingestor 
    ethhook-processor 
    ethhook-delivery 
    ethhook-admin-api
    RUNTIME DESTINATION bin
)

# Tests (optional)
option(BUILD_TESTS "Build unit tests" OFF)
if(BUILD_TESTS)
    enable_testing()
    pkg_check_modules(CHECK check>=0.15)
    if(CHECK_FOUND)
        add_subdirectory(tests)
    endif()
endif()

# =============================================================================
# Modern C Development Tooling
# =============================================================================

# Static Analysis with cppcheck
find_program(CPPCHECK_EXECUTABLE NAMES cppcheck)
if(CPPCHECK_EXECUTABLE)
    message(STATUS "cppcheck found: ${CPPCHECK_EXECUTABLE}")
    add_custom_target(cppcheck
        COMMAND ${CPPCHECK_EXECUTABLE}
            --enable=all
            --suppress=missingIncludeSystem
            --suppress=unmatchedSuppression
            --inline-suppr
            --error-exitcode=1
            --std=c17
            -I ${CMAKE_SOURCE_DIR}/include
            ${CMAKE_SOURCE_DIR}/src
        COMMENT "Running cppcheck static analysis"
        VERBATIM
    )
else()
    message(STATUS "cppcheck not found - install with: sudo apt-get install cppcheck")
endif()

# Code formatting with clang-format
find_program(CLANG_FORMAT_EXECUTABLE NAMES clang-format clang-format-14)
if(CLANG_FORMAT_EXECUTABLE)
    message(STATUS "clang-format found: ${CLANG_FORMAT_EXECUTABLE}")
    
    file(GLOB_RECURSE ALL_SOURCE_FILES
        ${CMAKE_SOURCE_DIR}/src/*.c
        ${CMAKE_SOURCE_DIR}/src/*.h
        ${CMAKE_SOURCE_DIR}/include/*.h
    )
    
    add_custom_target(format
        COMMAND ${CLANG_FORMAT_EXECUTABLE}
            -i
            -style=file
            ${ALL_SOURCE_FILES}
        COMMENT "Formatting code with clang-format"
        VERBATIM
    )
    
    add_custom_target(format-check
        COMMAND ${CLANG_FORMAT_EXECUTABLE}
            --dry-run
            --Werror
            -style=file
            ${ALL_SOURCE_FILES}
        COMMENT "Checking code formatting"
        VERBATIM
    )
else()
    message(STATUS "clang-format not found - install with: sudo apt-get install clang-format")
endif()

# Clang static analyzer
find_program(CLANG_ANALYZER NAMES scan-build scan-build-14)
if(CLANG_ANALYZER)
    message(STATUS "scan-build found: ${CLANG_ANALYZER}")
    add_custom_target(analyze
        COMMAND ${CLANG_ANALYZER}
            -o ${CMAKE_BINARY_DIR}/scan-build-report
            --status-bugs
            cmake --build ${CMAKE_BINARY_DIR} --clean-first
        COMMENT "Running Clang static analyzer"
        VERBATIM
    )
else()
    message(STATUS "scan-build not found - install with: sudo apt-get install clang-tools")
endif()

# Memory leak detection with Valgrind
find_program(VALGRIND_EXECUTABLE NAMES valgrind)
if(VALGRIND_EXECUTABLE AND BUILD_TESTS)
    message(STATUS "valgrind found: ${VALGRIND_EXECUTABLE}")
    add_custom_target(memcheck
        COMMAND ${VALGRIND_EXECUTABLE}
            --leak-check=full
            --show-leak-kinds=all
            --track-origins=yes
            --verbose
            --error-exitcode=1
            ${CMAKE_BINARY_DIR}/ethhook-admin-api --help
        COMMENT "Running Valgrind memory check"
        VERBATIM
    )
else()
    message(STATUS "valgrind not found or tests disabled - install with: sudo apt-get install valgrind")
endif()

# Print build configuration summary
message(STATUS "")
message(STATUS "=== EthHook C Build Configuration ===")
message(STATUS "C Standard: C${CMAKE_C_STANDARD}")
message(STATUS "Build Type: ${CMAKE_BUILD_TYPE}")
message(STATUS "AddressSanitizer: ${ENABLE_ASAN}")
message(STATUS "UndefinedBehaviorSanitizer: ${ENABLE_UBSAN}")
message(STATUS "ThreadSanitizer: ${ENABLE_TSAN}")
message(STATUS "Link-Time Optimization: ${IPO_SUPPORTED}")
message(STATUS "Static Analysis Available: ${CPPCHECK_EXECUTABLE}")
message(STATUS "Code Formatting Available: ${CLANG_FORMAT_EXECUTABLE}")
message(STATUS "=====================================")
message(STATUS "")
