add_library(Crashpad STATIC)

target_include_directories(Crashpad PUBLIC
    Crashpad/Include/
    Crashpad/Include/crashpad/
    Crashpad/Include/crashpad/out/Default/gen/
    Crashpad/Include/crashpad/third_party/mini_chromium/
    Crashpad/Include/crashpad/third_party/mini_chromium/mini_chromium/
)

if(WIN32)
    include(Crashpad-Windows.cmake)
elseif(APPLE)
    include(Crashpad-MacOS.cmake)
elseif(UNIX)
    include(Crashpad-Linux.cmake)
endif()

target_link_libraries(Crashpad PRIVATE
    Qt::Core
    CrashpadCommon
    CrashpadClient
    CrashpadUtil
    CrashpadBase
    spdlog::spdlog_header_only
)

# Make sure the crash_handler executable is copied to the build directory.
get_target_property(CrashPadHandlerPath CrashpadHandler IMPORTED_LOCATION)

# Define the destination path
set(CrashpadHandlerOutput "${CrashpadHandlerDestination}/${CrashpadHandlerExe}")

# Copy it at build time, not configure time
add_custom_command(
    OUTPUT "${CrashpadHandlerOutput}"
    COMMAND ${CMAKE_COMMAND} -E copy "${CrashPadHandlerPath}" "${CrashpadHandlerOutput}"
    DEPENDS "${CrashPadHandlerPath}"
    COMMENT "Copying crashpad_handler to build directory"
    VERBATIM
)

# Add a custom target so the file gets copied during builds
add_custom_target(copy_crashpad_handler ALL
    DEPENDS "${CrashpadHandlerOutput}"
)
