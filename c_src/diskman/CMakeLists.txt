project(diskman C)

add_library(diskman STATIC diskman.c)  # Build as static library

# Set output directories for all platforms
set_target_properties(diskman PROPERTIES
    ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib
    LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib
    RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib
)

install(TARGETS diskman
    ARCHIVE DESTINATION ${CMAKE_BINARY_DIR}/lib
)
