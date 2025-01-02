add_library(json_struct INTERFACE)

target_include_directories(json_struct INTERFACE
    ${CMAKE_CURRENT_SOURCE_DIR}
)

set_target_properties(json_struct PROPERTIES
    INTERFACE_COMPILE_FEATURES cxx_std_17
    INTERFACE_CXX_EXTENSIONS OFF
)

target_link_libraries(json_struct INTERFACE
    Qt::Core
    Qt::Network
)
