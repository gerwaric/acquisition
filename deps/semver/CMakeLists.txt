add_library(semver INTERFACE
    semver/semver.hpp
)

target_include_directories(semver INTERFACE
    "${CMAKE_CURRENT_SOURCE_DIR}"
)
