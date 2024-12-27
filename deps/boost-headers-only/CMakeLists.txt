add_library(boost-headers-only INTERFACE)

target_include_directories(boost-headers-only INTERFACE
    "${CMAKE_CURRENT_SOURCE_DIR}/boost-headers-only"
)
