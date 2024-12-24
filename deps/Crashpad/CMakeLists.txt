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
    QsLog
    CrashpadCommon
    CrashpadClient
    CrashpadUtil
    CrashpadBase
)

# Make sure the crash_handler executable is copied to the build directory.
set_target_properties(CrashpadHandler PROPERTIES
    RUNTIME_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}"
)
