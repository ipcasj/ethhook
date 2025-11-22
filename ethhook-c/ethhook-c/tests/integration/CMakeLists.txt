# Integration tests (require PostgreSQL + Redis)

# Example:
# add_executable(test_ingestor_integration test_ingestor.c)
# target_link_libraries(test_ingestor_integration ethhook-common ${LIBPQ_LIBRARIES} ${HIREDIS_LIBRARIES})
# add_test(NAME ingestor_integration COMMAND test_ingestor_integration)
