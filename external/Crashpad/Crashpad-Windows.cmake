
add_executable(CrashpadHandler IMPORTED)
set_target_properties(CrashpadHandler PROPERTIES
    IMPORTED_LOCATION "Crashpad/Bin/Windows/crashpad_handler.exe"
    RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}
)

set(CrashpadLibRoot "${CMAKE_CURRENT_SOURCE_DIR}/Crashpad/Libraries/Windows")

add_library(CrashpadBase STATIC IMPORTED)
set_target_properties(CrashpadBase PROPERTIES
    IMPORTED_LOCATION_RELEASE        "${CrashpadLibRoot}/MD/base.lib"
    IMPORTED_LOCATION_RELMINSIZE     "${CrashpadLibRoot}/MD/base.lib"
    IMPORTED_LOCATION_RELWITHDEBINFO "${CrashpadLibRoot}/MDd/base.lib"
    IMPORTED_LOCATION_DEBUG          "${CrashpadLibRoot}/MDd/base.lib"
)

add_library(CrashpadCommon STATIC IMPORTED)
set_target_properties(CrashpadCommon PROPERTIES
    IMPORTED_LOCATION_RELEASE        "${CrashpadLibRoot}/MD/common.lib"
    IMPORTED_LOCATION_RELMINSIZE     "${CrashpadLibRoot}/MD/common.lib"
    IMPORTED_LOCATION_RELWITHDEBINFO "${CrashpadLibRoot}/MDd/common.lib"
    IMPORTED_LOCATION_DEBUG          "${CrashpadLibRoot}/MDd/common.lib"
)

add_library(CrashpadClient STATIC IMPORTED)
set_target_properties(CrashpadClient PROPERTIES
    IMPORTED_LOCATION_RELEASE        "${CrashpadLibRoot}/MD/client.lib"
    IMPORTED_LOCATION_RELMINSIZE     "${CrashpadLibRoot}/MD/client.lib"
    IMPORTED_LOCATION_RELWITHDEBINFO "${CrashpadLibRoot}/MDd/client.lib"
    IMPORTED_LOCATION_DEBUG          "${CrashpadLibRoot}/MDd/client.lib"
)

add_library(CrashpadUtil STATIC IMPORTED)
set_target_properties(CrashpadUtil PROPERTIES
    IMPORTED_LOCATION_RELEASE        "${CrashpadLibRoot}/MD/util.lib"
    IMPORTED_LOCATION_RELMINSIZE     "${CrashpadLibRoot}/MD/util.lib"
    IMPORTED_LOCATION_RELWITHDEBINFO "${CrashpadLibRoot}/MDd/util.lib"
    IMPORTED_LOCATION_DEBUG          "${CrashpadLibRoot}/MDd/util.lib"
)

target_link_libraries(Crashpad PRIVATE
    CrashpadCommon
    CrashpadClient
    CrashpadUtil
    CrashpadBase
 )

 # Add etra libraries specific to Windows.
 target_link_libraries(Crashpad PRIVATE Advapi32)

