project(tui C)

find_package(Curses REQUIRED)
include_directories(${CURSES_INCLUDE_DIRS})

add_library(tui STATIC tui.c)  # Build as static library
target_link_libraries(tui ${CURSES_LIBRARIES})

# Set output directories for all platforms
set_target_properties(tui PROPERTIES
    ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib
    LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib
    RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib
)

install(TARGETS tui
    ARCHIVE DESTINATION ${CMAKE_BINARY_DIR}/lib
)
