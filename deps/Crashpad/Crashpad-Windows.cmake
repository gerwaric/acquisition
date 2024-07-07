add_executable(CrashpadHandler IMPORTED)    
set_target_properties(CrashpadHandler PROPERTIES
    IMPORTED_LOCATION "${PROJECT_SOURCE_DIR}/deps/Crashpad/Bin/Windows/crashpad_handler.exe"
)

add_library(CrashpadBase SHARED IMPORTED)
set_target_properties(CrashpadBase PROPERTIES
    IMPORTED_IMPLIB_DEBUG  "${PROJECT_SOURCE_DIR}/deps/Crashpad/Libraries/Windows/MDd/base.lib"
    IMPORTED_IMPLIB_RELEASE "${PROJECT_SOURCE_DIR}/deps/Crashpad/Libraries/Windows/MD/base.lib"
    IMPORTED_IMPLIB_RELWITHDEBINFO "${PROJECT_SOURCE_DIR}/deps/Crashpad/Libraries/Windows/MDd/base.lib"
    IMPORTED_IMPLIB_MINSIZEREL "${PROJECT_SOURCE_DIR}/deps/Crashpad/Libraries/Windows/MD/base.lib"
)

add_library(CrashpadCommon SHARED IMPORTED)
set_target_properties(CrashpadCommon PROPERTIES
    IMPORTED_IMPLIB_DEBUG  "${PROJECT_SOURCE_DIR}/deps/Crashpad/Libraries/Windows/MDd/common.lib"
    IMPORTED_IMPLIB_RELEASE "${PROJECT_SOURCE_DIR}/deps/Crashpad/Libraries/Windows/MD/common.lib"
    IMPORTED_IMPLIB_RELWITHDEBINFO "${PROJECT_SOURCE_DIR}/deps/Crashpad/Libraries/Windows/MDd/common.lib"
    IMPORTED_IMPLIB_MINSIZEREL "${PROJECT_SOURCE_DIR}/deps/Crashpad/Libraries/Windows/MD/common.lib"
)

add_library(CrashpadClient SHARED IMPORTED)
set_target_properties(CrashpadClient PROPERTIES
    IMPORTED_IMPLIB_DEBUG  "${PROJECT_SOURCE_DIR}/deps/Crashpad/Libraries/Windows/MDd/client.lib"
    IMPORTED_IMPLIB_RELEASE "${PROJECT_SOURCE_DIR}/deps/Crashpad/Libraries/Windows/MD/client.lib"
    IMPORTED_IMPLIB_RELWITHDEBINFO "${PROJECT_SOURCE_DIR}/deps/Crashpad/Libraries/Windows/MDd/client.lib"
    IMPORTED_IMPLIB_MINSIZEREL "${PROJECT_SOURCE_DIR}/deps/Crashpad/Libraries/Windows/MD/client.lib"
)

add_library(CrashpadUtil SHARED IMPORTED)
set_target_properties(CrashpadUtil PROPERTIES
    IMPORTED_IMPLIB_DEBUG  "${PROJECT_SOURCE_DIR}/deps/Crashpad/Libraries/Windows/MDd/util.lib"
    IMPORTED_IMPLIB_RELEASE "${PROJECT_SOURCE_DIR}/deps/Crashpad/Libraries/Windows/MD/base.lib"
    IMPORTED_IMPLIB_RELWITHDEBINFO "${PROJECT_SOURCE_DIR}/deps/Crashpad/Libraries/Windows/MDd/util.lib"
    IMPORTED_IMPLIB_MINSIZEREL "${PROJECT_SOURCE_DIR}/deps/Crashpad/Libraries/Windows/MD/util.lib"
)
