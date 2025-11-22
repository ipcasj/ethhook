# Example webhook receiver in C

add_executable(webhook-receiver
    main.c
)

target_link_libraries(webhook-receiver PRIVATE
    ethhook-common
    ${COMMON_LIBS}
)
