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

# Reproduce acquisition's situation: the root project enables testing
# globally (BUILD_TESTING=ON). The QCORO_* flags below must keep QCoro's
# tests and examples out WITHOUT touching the parent's BUILD_TESTING —
# forcing the global flag off here would disable the host project's own
# test suite when this file is used as the integration reference.
include(CTest)

include(FetchContent)

# FetchContent hygiene (spec "Dependency: QCoro"): QCoro's examples default
# ON, and its tests follow BUILD_TESTING unless QCORO_BUILD_TESTING is set —
# QCoro maps QCORO_BUILD_TESTING onto a directory-scoped BUILD_TESTING of its
# own, so the parent's flag stays untouched. Verified: with BUILD_TESTING=ON
# in the parent, `ctest -N` lists zero QCoro tests and no examples build.
set(QCORO_BUILD_EXAMPLES OFF CACHE BOOL "" FORCE)
set(QCORO_BUILD_TESTING OFF CACHE BOOL "" FORCE)
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

# Batch-scale measurement (phasing step 0's open measurement): peak memory and
# abort cost at the 2,000-tab scale. Separate binary so it gets a fresh
# process — clean memory baselines, away from qcoro_spike's deliberate leaks.
add_executable(qcoro_batch batch.cpp)
target_link_libraries(qcoro_batch PRIVATE
    Qt6::Core
    Qt6::Network
    QCoro6::Core
)

option(SPIKE_ASAN "Build the spike with AddressSanitizer" OFF)
if(SPIKE_ASAN)
    foreach(tgt qcoro_spike qcoro_batch)
        target_compile_options(${tgt} PRIVATE -fsanitize=address -fno-omit-frame-pointer)
        target_link_options(${tgt} PRIVATE -fsanitize=address)
    endforeach()
endif()
