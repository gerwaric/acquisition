set(CrashpadHandlerExe "crashpad_handler")
set(CrashpadHandlerDestination "${CMAKE_BINARY_DIR}/acquisition.app/Contents/MacOS")

add_executable(CrashpadHandler IMPORTED)    
set_target_properties(CrashpadHandler PROPERTIES
    IMPORTED_LOCATION "${CMAKE_CURRENT_SOURCE_DIR}/Crashpad/Bin/MacOS/${CMAKE_SYSTEM_PROCESSOR}/${CrashpadHandlerExe}"
)

function(setup_macos_target target libname)

    set(${target}_PREBUILT_LIBS
        "${CMAKE_CURRENT_SOURCE_DIR}/Crashpad/Libraries/MacOS/arm64/lib${libname}.a"
        "${CMAKE_CURRENT_SOURCE_DIR}/Crashpad/Libraries/MacOS/x86_64/lib${libname}.a"
    )

    set(${target}_OUTPUT_LIB "${CMAKE_BINARY_DIR}/crashpad/lib${libname}.a")

    add_custom_command(
        OUTPUT ${${target}_OUTPUT_LIB}
        DEPENDS ${${target}_PREBUILT_LIBS}
        COMMAND ${CMAKE_COMMAND} -E make_directory ${CMAKE_BINARY_DIR}/crashpad
        COMMAND lipo ${${target}_PREBUILT_LIBS} -create -output ${${target}_OUTPUT_LIB}
        COMMENT "Creating universal lib${libname}.a via lipo"
        VERBATIM
    )

    add_custom_target(${target}_lipo ALL
        DEPENDS ${${target}_OUTPUT_LIB}
    )

    add_library(${target} STATIC IMPORTED GLOBAL)
    set_target_properties(${target} PROPERTIES
        IMPORTED_LOCATION ${${target}_OUTPUT_LIB}
    )

    add_dependencies(${target} ${target}_lipo)
    
endfunction()

setup_macos_target(CrashpadBase base)
setup_macos_target(CrashpadClient client)
setup_macos_target(CrashpadCommon common)
setup_macos_target(CrashpadMigOutput mig_output)
setup_macos_target(CrashpadUtil util)

# Add libraries that were not specified in the top-level CMakeLists.txt
target_link_libraries(Crashpad PRIVATE CrashpadMigOutput)
target_link_libraries(Crashpad PRIVATE bsm "-framework AppKit" "-framework Security")
