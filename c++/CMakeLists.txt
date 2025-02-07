cmake_minimum_required(VERSION 3.10)
project(wing_osc_sender)

set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/build)
set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/build)
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/build)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

include(FetchContent)
FetchContent_Declare(
  fmt
  GIT_REPOSITORY https://github.com/fmtlib/fmt.git
  GIT_TAG        11.1.3
)
FetchContent_MakeAvailable(fmt)

# Create static library
add_library(wingstatic STATIC
    WingConsole.cpp
    WingNode.cpp
    wing_c_api.cpp
)

add_library(wing SHARED
    WingConsole.cpp
    WingNode.cpp
    wing_c_api.cpp
)

# Create executable that uses the library
add_executable(wingschema wingschema.cpp)
target_link_libraries(wingschema PRIVATE wingstatic fmt)

# Create executable that uses the library
add_executable(wingmon wingmon.cpp)
target_link_libraries(wingmon PRIVATE wingstatic fmt)
