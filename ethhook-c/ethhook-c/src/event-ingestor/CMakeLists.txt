# Event Ingestor - WebSocket listener for blockchain events
# Single executable with all dependencies

add_executable(event-ingestor
    main.c
)

target_link_libraries(event-ingestor PRIVATE
    ethhook-common
    ${LIBWEBSOCKETS_LIBRARIES}
    ${HIREDIS_LIBRARIES}
    ${LIBPQ_LIBRARIES}
    ${COMMON_LIBS}
)

target_include_directories(event-ingestor PRIVATE
    ${LIBWEBSOCKETS_INCLUDE_DIRS}
    ${HIREDIS_INCLUDE_DIRS}
    ${LIBPQ_INCLUDE_DIRS}
)

# Install
install(TARGETS event-ingestor RUNTIME DESTINATION bin)
