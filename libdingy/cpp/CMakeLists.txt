cmake_minimum_required(VERSION 3.16)
project(libdingy_test LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 14)

add_executable(libdingy_test test.cpp)

target_link_directories(libdingy_test PUBLIC ../target/debug)
target_link_libraries(libdingy_test PUBLIC libdingy)
