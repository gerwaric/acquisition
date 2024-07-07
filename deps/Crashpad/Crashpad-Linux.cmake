add_executable(CrashpadHandler IMPORTED)    
set_target_properties(CrashpadHandler PROPERTIES
    IMPORTED_LOCATION "${PROJECT_SOURCE_DIR}/deps/Crashpad/Bin/Linux/crashpad_handler"
)

function(setup_linux_target target libname)
    
    # Create the target.
    add_library(${target} SHARED IMPORTED)
    
    # Set the target location.
	set_target_properties(${target} PROPERTIES
        IMPORTED_IMPLIB  "${PROJECT_SOURCE_DIR}/deps/Crashpad/Libraries/Linux/lib${libname}.a"
    )
    
endfunction()

setup_linux_target(CrashpadBase base)
setup_linux_target(CrashpadCommon common)
setup_linux_target(CrashpadClient client)
setup_linux_target(CrashpadUtil util)
