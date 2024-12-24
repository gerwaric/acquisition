
add_executable(CrashpadHandler IMPORTED)
set_target_properties(CrashpadHandler PROPERTIES
    IMPORTED_LOCATION "${CMAKE_CURRENT_SOURCE_DIR}/Crashpad/Bin/Linux/crashpad_handler"
    RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}
)

add_library(CrashpadBase STATIC IMPORTED)
set_target_properties(CrashpadBase PROPERTIES
    IMPORTED_LOCATION "${CMAKE_CURRENT_SOURCE_DIR}/Crashpad/Libraries/Linux/libbase.a"
)

add_library(CrashpadCommon STATIC IMPORTED)
set_target_properties(CrashpadCommon PROPERTIES
    IMPORTED_LOCATION "${CMAKE_CURRENT_SOURCE_DIR}/Crashpad/Libraries/Linux/libcommon.a"
)

add_library(CrashpadClient STATIC IMPORTED)
set_target_properties(CrashpadClient PROPERTIES
    IMPORTED_LOCATION "${CMAKE_CURRENT_SOURCE_DIR}/Crashpad/Libraries/Linux/libclient.a"
)

add_library(CrashpadUtil STATIC IMPORTED)
set_target_properties(CrashpadUtil PROPERTIES
    IMPORTED_LOCATION "${CMAKE_CURRENT_SOURCE_DIR}/Crashpad/Libraries/Linux/libutil.a"
)

target_link_libraries(Crashpad PUBLIC
    CrashpadHandler
    CrashpadCommon
    CrashpadClient
    CrashpadUtil
    CrashpadBase
)
