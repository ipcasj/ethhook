cmake_minimum_required(VERSION 3.20)
project(ethhook C)

set(CMAKE_C_STANDARD 17)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_C_EXTENSIONS OFF)

# Compiler flags
add_compile_options(
    -Wall
    -Wextra
    -Wpedantic
    -Werror
    -O3
    -march=native
    -fno-strict-aliasing
)

# Enable LTO for release builds
include(CheckIPOSupported)
check_ipo_supported(RESULT IPO_SUPPORTED)
if(IPO_SUPPORTED)
    set(CMAKE_INTERPROCEDURAL_OPTIMIZATION_RELEASE ON)
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

# libcurl
pkg_check_modules(LIBCURL REQUIRED libcurl>=7.68)

# libmicrohttpd
pkg_check_modules(LIBMICROHTTPD REQUIRED libmicrohttpd>=0.9.70)

# libjwt
pkg_check_modules(LIBJWT REQUIRED libjwt>=1.12)

# OpenSSL (for crypto)
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
    ${LIBJWT_INCLUDE_DIRS}
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
)

target_link_libraries(ethhook-admin-api
    ethhook-common
    ${LIBMICROHTTPD_LIBRARIES}
    ${LIBJWT_LIBRARIES}
    ${OpenSSL_LIBRARIES}
    Threads::Threads
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
