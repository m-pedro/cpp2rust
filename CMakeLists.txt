cmake_minimum_required(VERSION 3.16)
project(cpp2rust VERSION 0.1.0 LANGUAGES C CXX)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Export compile_commands.json for IDE / clangd support
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

option(ENABLE_SANITIZERS "Enable AddressSanitizer and UndefinedBehaviorSanitizer (Day 4)" OFF)

add_compile_options(-Wall -Wextra -Wpedantic)

if(ENABLE_SANITIZERS)
    add_compile_options(-fsanitize=address,undefined -fno-omit-frame-pointer -g)
    add_link_options(-fsanitize=address,undefined)
else()
    add_compile_options(-O3)
endif()

include_directories(${CMAKE_CURRENT_SOURCE_DIR}/cpp/include)

# Static library containing C mmap reader and C++ parser (used later by Rust FFI)
add_library(legacy STATIC
    cpp/src/mmap_reader.c
    cpp/src/parser.cpp
)

# Standalone C++ CLI executable
add_executable(logengine_cpp
    cpp/src/main.cpp
)
target_link_libraries(logengine_cpp PRIVATE legacy)
