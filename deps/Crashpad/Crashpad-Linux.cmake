set(CrashpadHandlerExe "crashpad_handler.exe")
set(CrashpadHandlerDestination "${CMAKE_BINARY_DIR}")

add_executable(CrashpadHandler IMPORTED)    
set_target_properties(CrashpadHandler PROPERTIES
    IMPORTED_LOCATION "${CMAKE_CURRENT_SOURCE_DIR}/Crashpad/Bin/Linux/${CrashpadHandlerExe}"
)

function(setup_linux_target target libname)
    
    # Create the target.
    add_library(${target} SHARED IMPORTED)
    
    # Set the target location.
    set_target_properties(${target} PROPERTIES
        IMPORTED_IMPLIB  "${CMAKE_CURRENT_SOURCE_DIR}/Crashpad/Libraries/Linux/lib${libname}.a"
    )
    
endfunction()

setup_linux_target(CrashpadBase base)
setup_linux_target(CrashpadCommon common)
setup_linux_target(CrashpadClient client)
setup_linux_target(CrashpadUtil util)
