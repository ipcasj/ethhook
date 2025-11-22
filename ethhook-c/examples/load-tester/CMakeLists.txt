# Load testing tool

add_executable(load-tester
    main.c
)

target_link_libraries(load-tester PRIVATE
    ethhook-common
    ${LIBCURL_LIBRARIES}
    ${COMMON_LIBS}
)
