add_library(Crashpad STATIC)
target_include_directories(Crashpad INTERFACE
    Crashpad/Include/crashpad/
    Crashpad/Include/crashpad/out/Default/gen/
    Crashpad/Include/crashpad/third_party/mini_chromium/mini_chromium/
)

if(WIN32)
    include(Crashpad-Windows.cmake)
elseif(APPLE)
    include(Crashpad-MacOS.cmake)
elseif(UNIX)
    include(Crashpad-Linux.cmake)
endif()
