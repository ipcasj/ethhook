cmake_minimum_required(VERSION 3.20)
project(ethhook-c VERSION 1.0.0 LANGUAGES C)

# Use C17 standard
set(CMAKE_C_STANDARD 17)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_C_EXTENSIONS OFF)

# Export compile commands for clangd/LSP
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# Build options
option(ENABLE_TESTS "Build tests" ON)
option(ENABLE_ASAN "Enable AddressSanitizer" OFF)
option(ENABLE_UBSAN "Enable UndefinedBehaviorSanitizer" OFF)
option(ENABLE_EXAMPLES "Build examples" ON)
option(BUILD_STATIC "Build static binaries" OFF)

# Compiler flags
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -Wall -Wextra -Wpedantic -Werror")
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -Wno-unused-parameter -Wno-missing-field-initializers")

# Optimization flags
if(CMAKE_BUILD_TYPE STREQUAL "Release")
    set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -O3 -march=native -flto")
    set(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS} -flto")
endif()

# Debug flags
if(CMAKE_BUILD_TYPE STREQUAL "Debug")
    set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -g -O0")
endif()

# AddressSanitizer
if(ENABLE_ASAN)
    set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -fsanitize=address -fno-omit-frame-pointer")
    set(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS} -fsanitize=address")
endif()

# UndefinedBehaviorSanitizer
if(ENABLE_UBSAN)
    set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -fsanitize=undefined")
    set(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS} -fsanitize=undefined")
endif()

# Find required libraries
find_package(PkgConfig REQUIRED)

# libuv (async I/O, event loop)
pkg_check_modules(LIBUV REQUIRED libuv>=1.40)

# libcurl (HTTP client)
pkg_check_modules(LIBCURL REQUIRED libcurl>=7.68)

# PostgreSQL libpq
pkg_check_modules(LIBPQ REQUIRED libpq>=13)

# hiredis (Redis client)
pkg_check_modules(HIREDIS REQUIRED hiredis>=1.0)

# libwebsockets
pkg_check_modules(LIBWEBSOCKETS REQUIRED libwebsockets>=4.0)

# OpenSSL
pkg_check_modules(OPENSSL REQUIRED openssl>=1.1)

# libjwt (optional, for JWT support)
pkg_check_modules(LIBJWT libjwt>=1.12)

# cJSON (we'll use bundled version for simplicity)
# This allows single translation unit approach

# Include directories
include_directories(
    ${CMAKE_SOURCE_DIR}/include
    ${LIBUV_INCLUDE_DIRS}
    ${LIBCURL_INCLUDE_DIRS}
    ${LIBPQ_INCLUDE_DIRS}
    ${HIREDIS_INCLUDE_DIRS}
    ${LIBWEBSOCKETS_INCLUDE_DIRS}
    ${OPENSSL_INCLUDE_DIRS}
)

# Link directories
link_directories(
    ${LIBUV_LIBRARY_DIRS}
    ${LIBCURL_LIBRARY_DIRS}
    ${LIBPQ_LIBRARY_DIRS}
    ${HIREDIS_LIBRARY_DIRS}
    ${LIBWEBSOCKETS_LIBRARY_DIRS}
    ${OPENSSL_LIBRARY_DIRS}
)

# Common libraries for all targets
set(COMMON_LIBS
    ${LIBUV_LIBRARIES}
    ${OPENSSL_LIBRARIES}
    pthread
    m
)

# Output directories
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin)
set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)
set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)

# Subdirectories (modular build)
add_subdirectory(src/common)
add_subdirectory(src/event-ingestor)
add_subdirectory(src/message-processor)
add_subdirectory(src/webhook-delivery)
add_subdirectory(src/admin-api)

# Examples
if(ENABLE_EXAMPLES)
    add_subdirectory(examples/webhook-receiver)
    add_subdirectory(examples/load-tester)
endif()

# Tests
if(ENABLE_TESTS)
    enable_testing()
    add_subdirectory(tests)
endif()

# Installation
install(TARGETS event-ingestor message-processor webhook-delivery admin-api
    RUNTIME DESTINATION bin
    LIBRARY DESTINATION lib
    ARCHIVE DESTINATION lib
)

install(DIRECTORY include/ethhook
    DESTINATION include
)

# Uninstall target
if(NOT TARGET uninstall)
    configure_file(
        "${CMAKE_CURRENT_SOURCE_DIR}/cmake/cmake_uninstall.cmake.in"
        "${CMAKE_CURRENT_BINARY_DIR}/cmake_uninstall.cmake"
        IMMEDIATE @ONLY
    )

    add_custom_target(uninstall
        COMMAND ${CMAKE_COMMAND} -P ${CMAKE_CURRENT_BINARY_DIR}/cmake_uninstall.cmake
    )
endif()

# Print configuration summary
message(STATUS "")
message(STATUS "========== Configuration ==========")
message(STATUS "Project: ${PROJECT_NAME} v${PROJECT_VERSION}")
message(STATUS "Build type: ${CMAKE_BUILD_TYPE}")
message(STATUS "C Compiler: ${CMAKE_C_COMPILER}")
message(STATUS "C Standard: C${CMAKE_C_STANDARD}")
message(STATUS "")
message(STATUS "Options:")
message(STATUS "  ENABLE_TESTS: ${ENABLE_TESTS}")
message(STATUS "  ENABLE_ASAN: ${ENABLE_ASAN}")
message(STATUS "  ENABLE_UBSAN: ${ENABLE_UBSAN}")
message(STATUS "  ENABLE_EXAMPLES: ${ENABLE_EXAMPLES}")
message(STATUS "  BUILD_STATIC: ${BUILD_STATIC}")
message(STATUS "")
message(STATUS "Dependencies:")
message(STATUS "  libuv: ${LIBUV_VERSION}")
message(STATUS "  libcurl: ${LIBCURL_VERSION}")
message(STATUS "  libpq: ${LIBPQ_VERSION}")
message(STATUS "  hiredis: ${HIREDIS_VERSION}")
message(STATUS "  libwebsockets: ${LIBWEBSOCKETS_VERSION}")
message(STATUS "  OpenSSL: ${OPENSSL_VERSION}")
if(LIBJWT_FOUND)
    message(STATUS "  libjwt: ${LIBJWT_VERSION}")
else()
    message(STATUS "  libjwt: NOT FOUND (will use custom JWT implementation)")
endif()
message(STATUS "===================================")
message(STATUS "")
