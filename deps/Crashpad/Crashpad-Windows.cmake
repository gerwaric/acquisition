add_executable(CrashpadHandler IMPORTED)
set_target_properties(CrashpadHandler PROPERTIES
    IMPORTED_LOCATION "${PROJECT_SOURCE_DIR}/deps/Crashpad/Bin/Windows/crashpad_handler.exe"
)

function(setup_windows_target target libname)

    # Create the target
    add_library(${target} SHARED IMPORTED)

    # Set the correct library locations depending upon the release.
    set_target_properties(${target} PROPERTIES
        IMPORTED_IMPLIB_DEBUG  "${PROJECT_SOURCE_DIR}/deps/Crashpad/Libraries/Windows/MDd/${libname}.lib"
        IMPORTED_IMPLIB_RELEASE "${PROJECT_SOURCE_DIR}/deps/Crashpad/Libraries/Windows/MD/${libname}.lib"
        IMPORTED_IMPLIB_RELWITHDEBINFO "${PROJECT_SOURCE_DIR}/deps/Crashpad/Libraries/Windows/MDd/${libname}.lib"
        IMPORTED_IMPLIB_MINSIZEREL "${PROJECT_SOURCE_DIR}/deps/Crashpad/Libraries/Windows/MD/${libname}.lib"
    )

endfunction()

setup_windows_target(CrashpadBase base)
setup_windows_target(CrashpadClient client)
setup_windows_target(CrashpadCommon common)
setup_windows_target(CrashpadUtil util)

# Add libraries that were not specified in the top-level CMakeLists.txt
target_link_libraries(Crashpad PRIVATE Advapi32)
