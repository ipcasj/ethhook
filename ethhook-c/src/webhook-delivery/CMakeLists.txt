# Webhook Delivery - HTTP webhook sender with retries

add_executable(webhook-delivery
    main.c
)

target_link_libraries(webhook-delivery PRIVATE
    ethhook-common
    ${LIBCURL_LIBRARIES}
    ${HIREDIS_LIBRARIES}
    ${LIBPQ_LIBRARIES}
    ${COMMON_LIBS}
)

install(TARGETS webhook-delivery RUNTIME DESTINATION bin)
