# Admin API - REST API server

add_executable(admin-api
    main.c
)

target_link_libraries(admin-api PRIVATE
    ethhook-common
    ${LIBPQ_LIBRARIES}
    ${COMMON_LIBS}
)

install(TARGETS admin-api RUNTIME DESTINATION bin)
