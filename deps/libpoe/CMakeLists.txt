add_library(libpoe INTERFACE)

target_include_directories(libpoe INTERFACE
    ${CMAKE_CURRENT_SOURCE_DIR}
)

set_target_properties(libpoe PROPERTIES
    INTERFACE_COMPILE_FEATURES cxx_std_17
    INTERFACE_CXX_EXTENSIONS OFF
)

target_link_libraries(libpoe INTERFACE
    Qt::Core
)
