add_executable(CrashpadHandler IMPORTED)    
set_target_properties(CrashpadHandler PROPERTIES
	IMPORTED_LOCATION "${PROJECT_SOURCE_DIR}/deps/Crashpad/Bin/Linux/crashpad_handler")

add_library(CrashpadBase SHARED IMPORTED)
set_target_properties(CrashpadBase PROPERTIES
    IMPORTED_IMPLIB  "${PROJECT_SOURCE_DIR}/deps/Crashpad/Libraries/Linux/libbase.a")

add_library(CrashpadCommon SHARED IMPORTED)
set_target_properties(CrashpadCommon PROPERTIES
    IMPORTED_IMPLIB  "${PROJECT_SOURCE_DIR}/deps/Crashpad/Libraries/Linux/libcommon.a")

add_library(CrashpadClient SHARED IMPORTED)
set_target_properties(CrashpadClient PROPERTIES
    IMPORTED_IMPLIB  "${PROJECT_SOURCE_DIR}/deps/Crashpad/Libraries/Linux/libclient.a")

add_library(CrashpadUtil SHARED IMPORTED)
set_target_properties(CrashpadUtil PROPERTIES
    IMPORTED_IMPLIB  "${PROJECT_SOURCE_DIR}/deps/Crashpad/Libraries/Linux/libutil.a")
