
add_executable(CrashpadHandler IMPORTED)
set_target_properties(CrashpadHandler PROPERTIES
    IMPORTED_LOCATION "${CMAKE_CURRENT_SOURCE_DIR}/Crashpad/Bin/MacOS/${CMAKE_SYSTEM_PROCESSOR}/crashpad_handler"
    RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}
)

function(setup_macos_target targetname libname)
    
    # Point to the precompiled libraries.
    set(${targetname}_PREBUILT_LIBS
        "${CMAKE_CURRENT_SOURCE_DIR}/Crashpad/Libraries/MacOS/arm64/lib${libname}.a"
        "${CMAKE_CURRENT_SOURCE_DIR}/Crashpad/Libraries/MacOS/x86_64/lib${libname}.a"
    )
    
    # Define the output library, which will be multi-architecture.
    set(${target}_OUTPUT_LIB "${CMAKE_BINARY_DIR}/crashpad/lib${libname}.a")
    
    # Use a custom target to cause lipo to combine the two input libraries.
    add_custom_target(${targetname}_MULTIARCH
        DEPENDS ${${targetname}_PREBUILT_LIBS}
        BYPRODUCTS ${${targetname}_OUTPUT_LIB}
        COMMAND lipo ${${targetname}_PREBUILT_LIBS} -create -output ${${targetname}_OUTPUT_LIB}
    )

    # Add the library target and point it to the generated library.
    add_library(${targetname} STATIC IMPORTED)
    set_target_properties(${targetname} PROPERTIES
        IMPORTED_LOCATION ${${targetname}_OUTPUT_LIB}
    )

endfunction()

setup_macos_target(CrashpadBase base)
setup_macos_target(CrashpadClient client)
setup_macos_target(CrashpadCommon common)
setup_macos_target(CrashpadMigOutput mig_output)
setup_macos_target(CrashpadUtil util)

target_link_libraries(Crashpad PUBLIC
    CrashpadHandler
    CrashpadCommon
    CrashpadClient
    CrashpadUtil
    CrashpadBase
)

# Add some extra libraries needed for crashpad on macOS.
target_link_libraries(Crashpad PRIVATE bsm "-framework AppKit" "-framework Security")
