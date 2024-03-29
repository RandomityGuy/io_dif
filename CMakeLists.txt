cmake_minimum_required(VERSION 3.6)
project(DifBuilderLib)

add_subdirectory("3rdparty/DifBuilder")

if (UNIX)
    set(CMAKE_CXX_FLAGS "-std=c++11")
else()
    set(CMAKE_CXX_FLAGS "/MT")
    set(CMAKE_CXX_FLAGS_RELEASE "/MT")
    set(CMAKE_CXX_FLAGS_DEBUG "/MTd /FS")
endif()

set(SOURCE_FILES DifBuilderLib.cpp)
add_library(DifBuilderLib SHARED ${SOURCE_FILES})

if (UNIX AND APPLE)
    set_target_properties(DifBuilder PROPERTIES COMPILE_FLAGS "-O0") # Disable optimizations cause it breaks
    set_target_properties(DifBuilderLib PROPERTIES COMPILE_FLAGS "-O0")
    set_target_properties(DifBuilderLib PROPERTIES PREFIX "")
elseif (UNIX AND NOT APPLE)
    set_target_properties(Dif PROPERTIES COMPILE_FLAGS "-O0 -fPIC")
    set_target_properties(DifBuilder PROPERTIES COMPILE_FLAGS "-O0 -fPIC") # Disable optimizations cause it breaks
    set_target_properties(DifBuilderLib PROPERTIES COMPILE_FLAGS "-O0 -fPIC")
    set_target_properties(DifBuilderLib PROPERTIES PREFIX "")
else ()
    set_target_properties(DifBuilder PROPERTIES COMPILE_FLAGS "/Od /Ob0") # Disable optimizations cause it breaks
    set_target_properties(DifBuilderLib PROPERTIES COMPILE_FLAGS "/Od /Ob0")
endif()
include_directories(3rdparty/DifBuilder/include)
include_directories(3rdparty/DifBuilder/3rdparty/)
include_directories(3rdparty/DifBuilder/3rdparty/Dif)
include_directories(3rdparty/DifBuilder/3rdparty/Dif/3rdparty/glm)
include_directories(3rdparty/DifBuilder/3rdparty/Dif/include)
target_link_libraries(DifBuilderLib DifBuilder Dif)