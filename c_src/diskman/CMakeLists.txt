project(diskman C)

add_library(diskman SHARED diskman.c)

# Install the library to a predictable location
install(TARGETS diskman
    LIBRARY DESTINATION ${CMAKE_BINARY_DIR}/lib
    RUNTIME DESTINATION ${CMAKE_BINARY_DIR}/lib
)

# Also set the build-time location
set_target_properties(diskman PROPERTIES 
    LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib
    RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib
)
