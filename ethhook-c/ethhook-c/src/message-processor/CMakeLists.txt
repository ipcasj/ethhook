# Message Processor - Event filtering and routing

add_executable(message-processor
    main.c
)

target_link_libraries(message-processor PRIVATE
    ethhook-common
    ${HIREDIS_LIBRARIES}
    ${LIBPQ_LIBRARIES}
    ${COMMON_LIBS}
)

install(TARGETS message-processor RUNTIME DESTINATION bin)
