add_executable(CrashpadHandler IMPORTED)    
set_target_properties(CrashpadHandler PROPERTIES
    IMPORTED_LOCATION "${PROJECT_SOURCE_DIR}/deps/Crashpad/Bin/MacOS/${CMAKE_SYSTEM_PROCESSOR}/crashpad_handler"
)

function(setup_macos_target target libname)

    # Point to the precompiled libraries.
    set(${target}_PREBUILT_LIBS
        "${PROJECT_SOURCE_DIR}/deps/Crashpad/Libraries/MacOS/arm64/${libname}.a"
        "${PROJECT_SOURCE_DIR}/deps/Crashpad/Libraries/MacOS/x86_64/${libname}.a"
    )

    # Define the output library, which will be multi-architecture.
    set(${target}_OUTPUT_LIB "${CMAKE_BINARY_DIR}/libs/${libname}.a")

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

setup_macos_target(CrashpadBase libbase)
setup_macos_target(CrashpadClient libclient)
setup_macos_target(CrashpadCommon libcommon)
setup_macos_target(CrashpadMigOutput libmig_output)
setup_macos_target(CrashpadUtil libutil)

# Add libraries that were not specified in the top-level CMakeLists.txt
target_link_libraries(Crashpad PRIVATE CrashpadMigOutput)
target_link_libraries(Crashpad PRIVATE bsm "-framework AppKit" "-framework Security")
