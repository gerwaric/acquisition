# Phase-0 QCoro integration spike (network-redesign spec, phasing step 0).
#
# Standalone throwaway project — NOT part of the acquisition build.
# Configure separately:
#   cmake -S spikes/qcoro -B build-spike -DCMAKE_PREFIX_PATH=$HOME/Qt/6.11.1/macos
#   cmake --build build-spike
#   ./build-spike/qcoro_spike
#
# Exercises the spec's spike question list: destroy-while-suspended on each
# awaitable, queued-vs-synchronous resumption, unawaited-task exceptions,
# result() vs takeResult(), promise-completion re-entrancy, and the
# FetchContent hygiene flags below (QCoro examples off, its tests out of CTest).

cmake_minimum_required(VERSION 3.28)
project(qcoro_spike LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_AUTOMOC ON)

find_package(Qt6 6.8 REQUIRED COMPONENTS Core Network)

include(FetchContent)

# FetchContent hygiene (spec "Dependency: QCoro"): QCoro's examples default ON
# and its tests inherit BUILD_TESTING, which acquisition enables globally.
# Both must be forced off. The spike verifies these are the right knobs.
set(QCORO_BUILD_EXAMPLES OFF CACHE BOOL "" FORCE)
set(QCORO_BUILD_TESTING OFF CACHE BOOL "" FORCE)
set(BUILD_TESTING OFF CACHE BOOL "" FORCE)
set(QCORO_WITH_QTDBUS OFF CACHE BOOL "" FORCE)
set(QCORO_WITH_QTWEBSOCKETS OFF CACHE BOOL "" FORCE)
set(QCORO_WITH_QTQUICK OFF CACHE BOOL "" FORCE)
set(QCORO_WITH_QML OFF CACHE BOOL "" FORCE)

FetchContent_Declare(
    qcoro
    GIT_REPOSITORY https://github.com/qcoro/qcoro.git
    GIT_TAG        v0.13.0
)
FetchContent_MakeAvailable(qcoro)

add_executable(qcoro_spike main.cpp)
target_link_libraries(qcoro_spike PRIVATE
    Qt6::Core
    Qt6::Network
    QCoro6::Core
    QCoro6::Network
)

option(SPIKE_ASAN "Build the spike with AddressSanitizer" OFF)
if(SPIKE_ASAN)
    target_compile_options(qcoro_spike PRIVATE -fsanitize=address -fno-omit-frame-pointer)
    target_link_options(qcoro_spike PRIVATE -fsanitize=address)
endif()
