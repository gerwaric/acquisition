add_library(json_struct INTERFACE
    json_struct/json_struct.h
    json_struct/json_struct_diff.h
)

target_include_directories(json_struct INTERFACE
    "${CMAKE_CURRENT_SOURCE_DIR}"
)
