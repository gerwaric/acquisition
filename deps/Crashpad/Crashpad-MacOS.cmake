add_executable(CrashpadHandler IMPORTED)    
set_target_properties(CrashpadHandler PROPERTIES
    IMPORTED_LOCATION "${CMAKE_CURRENT_SOURCE_DIR}/Crashpad/Bin/MacOS/${CMAKE_SYSTEM_PROCESSOR}/crashpad_handler"
)

function(setup_macos_target target libname)
    
    # Point to the precompiled libraries.
    set(${target}_PREBUILT_LIBS
        "${CMAKE_CURRENT_SOURCE_DIR}/Crashpad/Libraries/MacOS/arm64/lib${libname}.a"
        "${CMAKE_CURRENT_SOURCE_DIR}/Crashpad/Libraries/MacOS/x86_64/lib${libname}.a"
    )
    
    # Define the output library, which will be multi-architecture.
    set(${target}_OUTPUT_LIB "${CMAKE_BINARY_DIR}/crashpad/lib${libname}.a")
    
    # Use a custom target to cause lipo to combine the two input libraries.
    add_custom_target(${target}_MULTIARCH
        DEPENDS ${${target}_PREBUILT_LIBS}
        BYPRODUCTS ${${target}_OUTPUT_LIB}
        COMMAND lipo ${${target}_PREBUILT_LIBS} -create -output ${${target}_OUTPUT_LIB}
    )
    
    # Create the target
    add_library(${target} SHARED IMPORTED)
    
    # Point the target to the generated multi-architecture library.
    set_target_properties(${target} PROPERTIES IMPORTED_LOCATION ${${target}_OUTPUT_LIB})
    
endfunction()

setup_macos_target(CrashpadBase base)
setup_macos_target(CrashpadClient client)
setup_macos_target(CrashpadCommon common)
setup_macos_target(CrashpadMigOutput mig_output)
setup_macos_target(CrashpadUtil util)

# Add libraries that were not specified in the top-level CMakeLists.txt
target_link_libraries(Crashpad PRIVATE CrashpadMigOutput)
target_link_libraries(Crashpad PRIVATE bsm "-framework AppKit" "-framework Security")
