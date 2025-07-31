project(tui C)

find_package(Curses REQUIRED)
include_directories(${CURSES_INCLUDE_DIRS})

add_library(tui SHARED tui.c)
target_link_libraries(tui ${CURSES_LIBRARIES})

# Install the library to a predictable location
install(TARGETS tui
    LIBRARY DESTINATION ${CMAKE_BINARY_DIR}/lib
    RUNTIME DESTINATION ${CMAKE_BINARY_DIR}/lib
)

# Also set the build-time location
set_target_properties(tui PROPERTIES 
    LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib
    RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib
)
