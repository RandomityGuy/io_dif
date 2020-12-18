cmake_minimum_required(VERSION 3.6)
project(DifBuilderLib)

add_subdirectory("3rdparty/DifBuilder")

set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -std=c++11")

set(SOURCE_FILES DifBuilderLib.cpp)
add_library(DifBuilderLib ${SOURCE_FILES})

include_directories(3rdparty/DifBuilder/include)
include_directories(3rdparty/DifBuilder/3rdparty/)
include_directories(3rdparty/DifBuilder/3rdparty/Dif)
include_directories(3rdparty/DifBuilder/3rdparty/Dif/3rdparty/glm)
include_directories(3rdparty/DifBuilder/3rdparty/Dif/include)
target_link_libraries(DifBuilderLib DifBuilder Dif)