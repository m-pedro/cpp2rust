cmake_minimum_required(VERSION 3.16)
project(cpp2rust VERSION 0.1.0 LANGUAGES C CXX)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

option(ENABLE_SANITIZERS "Enable AddressSanitizer and UndefinedBehaviorSanitizer (Day 4)" OFF)

find_package(Threads REQUIRED)

add_compile_options(-Wall -Wextra -Wpedantic)

if(ENABLE_SANITIZERS)
    add_compile_options(-fsanitize=address,undefined -fno-omit-frame-pointer -g -O1)
    add_link_options(-fsanitize=address,undefined)
else()
    add_compile_options(-O3)
endif()

include_directories(${CMAKE_CURRENT_SOURCE_DIR}/cpp/include)

# Static library containing C mmap reader, C++ parser, and C++ multithreaded aggregator
add_library(legacy STATIC
    cpp/src/mmap_reader.c
    cpp/src/parser.cpp
    cpp/src/aggregator.cpp
)
target_link_libraries(legacy PUBLIC Threads::Threads)

# Standalone C++ CLI executable
add_executable(logengine_cpp
    cpp/src/main.cpp
)
target_link_libraries(logengine_cpp PRIVATE legacy)

# Day 4: Unit & Golden Parity Test Suite
enable_testing()
add_executable(test_engine
    cpp/tests/test_main.cpp
)
target_link_libraries(test_engine PRIVATE legacy)
add_test(NAME LegacyEngineTests COMMAND test_engine)

add_library(legacy STATIC
    cpp/src/mmap_reader.c
    cpp/src/parser.cpp
    cpp/src/aggregator.cpp
    cpp/src/c_api.cpp
)
target_link_libraries(legacy PUBLIC Threads::Threads)

