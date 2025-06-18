set(CrashpadHandlerExe "crashpad_handler.exe")
set(CrashpadHandlerDestination "${CMAKE_BINARY_DIR}")

add_executable(CrashpadHandler IMPORTED)
set_target_properties(CrashpadHandler PROPERTIES
    IMPORTED_LOCATION "${CMAKE_CURRENT_SOURCE_DIR}/Crashpad/Bin/Windows/${CrashpadHandlerExe}"
)

function(setup_windows_target target libname)
    
    # Create the target
    add_library(${target} STATIC IMPORTED)
    
    # Set the correct library locations depending upon the release.
    set_target_properties(${target} PROPERTIES
        IMPORTED_LOCATION_DEBUG  "${CMAKE_CURRENT_SOURCE_DIR}/Crashpad/Libraries/Windows/MDd/${libname}.lib"
        IMPORTED_LOCATION_RELEASE "${CMAKE_CURRENT_SOURCE_DIR}/Crashpad/Libraries/Windows/MD/${libname}.lib"
        IMPORTED_LOCATION_RELWITHDEBINFO "${CMAKE_CURRENT_SOURCE_DIR}/Crashpad/Libraries/Windows/MDd/${libname}.lib"
        IMPORTED_LOCATION_MINSIZEREL "${CMAKE_CURRENT_SOURCE_DIR}/Crashpad/Libraries/Windows/MD/${libname}.lib"
    )
    
endfunction()

setup_windows_target(CrashpadBase base)
setup_windows_target(CrashpadClient client)
setup_windows_target(CrashpadCommon common)
setup_windows_target(CrashpadUtil util)

# Add libraries that were not specified in the top-level CMakeLists.txt
target_link_libraries(Crashpad PRIVATE Advapi32)
