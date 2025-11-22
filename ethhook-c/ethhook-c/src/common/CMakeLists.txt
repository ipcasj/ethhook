# Common library - shared code for all services
# Single Translation Unit approach - each module is one .c file

add_library(ethhook-common STATIC
    arena.c
    log.c
    config.c
    json.c
    crypto.c
    metrics.c
    utils.c
)

target_include_directories(ethhook-common PUBLIC
    ${CMAKE_SOURCE_DIR}/include
)

target_link_libraries(ethhook-common PUBLIC
    ${COMMON_LIBS}
)

# Optional: Build arena tests standalone
if(ENABLE_TESTS)
    add_executable(test_arena_standalone arena.c)
    target_compile_definitions(test_arena_standalone PRIVATE ARENA_ENABLE_TESTS)
    target_include_directories(test_arena_standalone PRIVATE ${CMAKE_SOURCE_DIR}/include)
    add_test(NAME arena_standalone COMMAND test_arena_standalone)
endif()
