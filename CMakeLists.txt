#[[
	Copyright (C) 2014-2024 Acquisition Contributors

	This file is part of Acquisition.

	Acquisition is free software: you can redistribute it and/or modify
	it under the terms of the GNU General Public License as published by
	the Free Software Foundation, either version 3 of the License, or
	(at your option) any later version.

	Acquisition is distributed in the hope that it will be useful,
	but WITHOUT ANY WARRANTY; without even the implied warranty of
	MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
	GNU General Public License for more details.

	You should have received a copy of the GNU General Public License
	along with Acquisition.  If not, see <http://www.gnu.org/licenses/>.
#]]

cmake_minimum_required(VERSION 3.29)

# NOTE: For automatic update detection to work, the 'app_version' variable
# defined below should match a tag in github. Historically, the github tags
# have been identical to the 'app_version' as defined below, except the 
# tags on github are prefixed with a 'v'.
#
# The version code that used to be used for update detection is no longer
# needed, except that some users with version of acquisition prior to
# version 0.10.5 will still need it, so we can't outright get rid of it.

project(acquisition
    VERSION 0.12.1
    DESCRIPTION "Stash and forum shop management for Path of Exile (TM)"
    HOMEPAGE_URL "https://github.com/gerwaric/acquisition"
    LANGUAGES CXX
)

# Use this to define pre-releases
set(version_postfix "")

# Variables used to generate version_defines.h and installer.iss
set(version_code        63) # This doesn't need to be changed with the new update checks.
set(app_name            "${CMAKE_PROJECT_NAME}")
set(app_version         "${CMAKE_PROJECT_VERSION}")
set(app_version_string  "${CMAKE_PROJECT_VERSION}${version_postfix}")
set(app_url             "${CMAKE_PROJECT_HOMEPAGE_URL}")
set(app_publisher       "GERWARIC")
set(app_publisher_email "gerwaric@gmail.com")
set(app_copyright       "Copyright (C) 2014-2024 Ilya Zhuravlev and other Acquisition Contributors")
set(app_trademark       "Path of Exile is a trademark of Grinding Gear Games, Ltd.")

set(CMAKE_AUTOUIC ON)
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)

set(CMAKE_AUTOUIC_SEARCH_PATHS forms/)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# When building with Qt Creator in Windows, the compiler and linker options
# need to be set before qt_add_executable(), otherwise these options won't be used.
# (This is apparently not an issue with Visual Studio 2022).

if(WIN32)

    # Enable more warnings by default
    add_compile_options(/W4)
    add_compile_options(/EHsc)

    # Create .pdb files for debugging
    add_compile_options("$<$<NOT:$<CONFIG:Debug>>:/Zi>")
    add_link_options("$<$<NOT:$<CONFIG:Debug>>:/DEBUG>")
    add_link_options("$<$<NOT:$<CONFIG:Debug>>:/OPT:REF>")
    add_link_options("$<$<NOT:$<CONFIG:Debug>>:/OPT:ICF>")

endif()

find_package(Qt6 REQUIRED COMPONENTS Core Gui HttpServer Network Sql Test WebSockets Widgets)

qt_standard_project_setup(REQUIRES 6.8.0)

qt_add_executable(acquisition WIN32 MACOSX_BUNDLE
    # Source
    src/application.cpp
    src/bucket.cpp
    src/buyout.cpp
    src/buyoutmanager.cpp
    src/column.cpp
    src/currency.cpp
    src/currencymanager.cpp
    src/datastore.cpp
    src/fatalerror.cpp
    src/filters.cpp
    src/flowlayout.cpp
    src/imagecache.cpp
    src/influence.cpp
    src/item.cpp
    src/itemcategories.cpp
    src/itemconstants.cpp
    src/itemlocation.cpp
    src/items_model.cpp
    src/itemsmanager.cpp
    src/itemsmanagerworker.cpp
    src/itemtooltip.cpp
    src/logindialog.cpp
    src/logpanel.cpp
    src/main.cpp
    src/mainwindow.cpp
    src/memorydatastore.cpp
    src/modlist.cpp
    src/modsfilter.cpp
    src/networkmanager.cpp
    src/oauthmanager.cpp
    src/oauthtoken.cpp
    src/ratelimit.cpp
    src/ratelimitdialog.cpp
    src/ratelimiter.cpp
    src/ratelimitmanager.cpp
    src/repoe.cpp
    src/search.cpp
    src/searchcombobox.cpp
    src/shop.cpp
    src/sqlitedatastore.cpp
    src/updatechecker.cpp
    src/util.cpp
    src/verticalscrollarea.cpp
    test/testdata.cpp
    test/testitem.cpp
    test/testitemsmanager.cpp
    test/testmain.cpp
    test/testshop.cpp
    test/testutil.cpp
    # Headers
    src/application.h
    src/bucket.h
    src/buyout.h
    src/buyoutmanager.h
    src/column.h
    src/currency.h
    src/currencymanager.h
    src/datastore.h
    src/fatalerror.h
    src/filters.h
    src/flowlayout.h
    src/imagecache.h
    src/influence.h
    src/item.h
    src/itemcategories.h
    src/itemconstants.h
    src/itemlocation.h
    src/items_model.h
    src/itemsmanager.h
    src/itemsmanagerworker.h
    src/itemtooltip.h
    src/logindialog.h
    src/logpanel.h
    src/mainwindow.h
    src/memorydatastore.h
    src/modlist.h
    src/modsfilter.h
    src/network_info.h
    src/networkmanager.h
    src/oauthmanager.h
    src/oauthtoken.h
    src/rapidjson_util.h
    src/ratelimit.h
    src/ratelimitdialog.h
    src/ratelimiter.h
    src/ratelimitmanager.h
    src/replytimeout.h
    src/repoe.h
    src/search.h
    src/searchcombobox.h
    src/shop.h
    src/sqlitedatastore.h
    src/updatechecker.h
    src/util.h
    src/verticalscrollarea.h
    test/testdata.h
    test/testitem.h
    test/testitemsmanager.h
    test/testmain.h
    test/testshop.h
    test/testutil.h
    # Forms
    forms/logindialog.ui
    forms/mainwindow.ui
    # Resources
    resources.qrc
    # Generated version header
    "${CMAKE_BINARY_DIR}/version_defines.h"
)

# On Windows, we want to know which version of MSVC was used.
if(WIN32)

    if(NOT MSVC)
        message(FATAL_ERROR "Only the MSVC compiler is supported on Windows")
    endif()

    # On Windows there we do extra checks to catch incompatible older versions
    # of the Microsoft Visual Studio C++ Runtime, because these can cause
    # otherwise unexpected crashes (usually in mutex-related code).
    #
    # For this to work, we need to know the MSCV runtime version that acquisition
    # was built with, which is what the cmake code below this include does.
    target_sources(acquisition PRIVATE
        src/checkmsvc.cpp
        src/checkmsvc.h
    )

    # The Microsoft Visual C++ Compiler version 19 corresponds to runtime version 14.
    set(msvc_compiler_ver "19")
    set(msvc_runtime_ver "14")

    # Run the compiler executable to grab the version printed on the first line.
    execute_process(
        COMMAND cl
        OUTPUT_VARIABLE cl_output
        ERROR_VARIABLE cl_error
        OUTPUT_STRIP_TRAILING_WHITESPACE
    )

    # Extract the compiler version.
    string(REGEX MATCH "Version ([0-9]+\\.[0-9]+\\.[0-9]+)" msvc_version ${cl_error})
    string(REGEX REPLACE "^Version " "" msvc_version ${msvc_version})
    if (NOT msvc_version)
        message(FATAL_ERROR "Could not determine MSVC version from cl.exe output")
    endif()
    message(STATUS "MSVC Compiler: ${msvc_version}")

    # Derive the runtime version.
    string(REGEX REPLACE "^${msvc_compiler_ver}" "${msvc_runtime_ver}" msvc_runtime_version ${msvc_version})
    if (NOT msvc_runtime_version)
        message(FATAL_ERROR "Could not determine MSVC runtime version from cl.exe output")
    endif()
    message(STATUS "MSVC Runtime: ${msvc_runtime_version}")

endif()

# Automatically populate and include the versions header file.
configure_file("src/version_defines.h.in" "version_defines.h" @ONLY)

# Setup include directories.
target_include_directories(acquisition PRIVATE
    deps/
    deps/boost-headers-only/
    src/
    test/
    "${CMAKE_BINARY_DIR}"
)

# Link required libraries.
target_link_libraries(acquisition PRIVATE
    Qt::Core
    Qt::Gui
    Qt::HttpServer
    Qt::Network
    Qt::Widgets
    Qt::Sql
    Qt::Test
    Qt::WebSockets
    QsLog
    qdarkstyle
    Crashpad
    semver
)

add_library(semver STATIC
    deps/cpp-semver/semver.hpp
)

# Create the Crashpad library target.
add_library(Crashpad STATIC
    src/crashpad.cpp
    src/crashpad.h
)

target_link_libraries(Crashpad PRIVATE
    Qt::Core
    QsLog
    CrashpadCommon
    CrashpadClient
    CrashpadUtil
    CrashpadBase
)

target_include_directories(Crashpad PRIVATE
    deps/Crashpad/Include/
    deps/Crashpad/Include/crashpad/
    deps/Crashpad/Include/crashpad/out/Default/gen/
    deps/Crashpad/Include/crashpad/third_party/mini_chromium/
    deps/Crashpad/Include/crashpad/third_party/mini_chromium/mini_chromium/
)

if(WIN32)
    include(deps/Crashpad/Crashpad-Windows.cmake)
elseif(APPLE)
    include(deps/Crashpad/Crashpad-MacOS.cmake)
elseif(UNIX)
    include(deps/Crashpad/Crashpad-Linux.cmake)
endif()

# Make sure the crash_handler executable is copied to the build directory.
get_target_property(CrashpadHandlerPath CrashpadHandler IMPORTED_LOCATION)
add_custom_command(TARGET acquisition
    POST_BUILD
    COMMAND "${CMAKE_COMMAND}" -E copy "${CrashpadHandlerPath}" "${CMAKE_BINARY_DIR}"
)

# Add dependency: QsLog
qt_add_library(QsLog STATIC
    # Source
    deps/QsLog/QsLog.cpp
    deps/QsLog/QsLogDest.cpp
    deps/QsLog/QsLogDestConsole.cpp
    deps/QsLog/QsLogDestFile.cpp
    deps/QsLog/QsLogDestFunctor.cpp
    # Headers
    deps/QsLog/QsLog.h
    deps/QsLog/QsLogDest.h
    deps/QsLog/QsLogDestConsole.h
    deps/QsLog/QsLogDestFile.h
    deps/QsLog/QsLogDestFunctor.h
    deps/QsLog/QsLogDisableForThisFile.h
    deps/QsLog/QsLogLevel.h
)

target_include_directories(QsLog PUBLIC
    deps/
)

target_link_libraries(QsLog PRIVATE
    Qt::Core
)

# Add dependency: darkstyle
add_library(qdarkstyle STATIC
    deps/qdarkstyle/dark/darkstyle.qrc
    deps/qdarkstyle/light/lightstyle.qrc
)

#----------------------------------
# Platform-specific section
#----------------------------------

if(WIN32)
    
    # As of v0.11.8 I'm still seeing crashes in the MSVC mutex code. It looks like there
    # was a change in Visual Studio that causes this. The follwing define is an attempt
    # to fix it.
    #
    # See https://stackoverflow.com/questions/78598141/first-stdmutexlock-crashes-in-application-built-with-latest-visual-studio
    target_compile_definitions(acquisition PRIVATE _DISABLE_CONSTEXPR_MUTEX_CONSTRUCTOR)

    # Needed to check the MSVC runtime version
    target_link_libraries(acquisition PRIVATE Version)

    # Add the application icon.
    target_sources(acquisition PRIVATE "${PROJECT_SOURCE_DIR}/assets/icon.ico")

    # Use a function to set version variables so there can be a default value.
    function(setver name value default)
        if (NOT "${value}" STREQUAL "")
            set("${name}" "${value}" PARENT_SCOPE)
        else()
            set("${name}" "${default}" PARENT_SCOPE)
        endif()
    endfunction()

    # Define the version variables.
    setver(app_VERSION_MAJOR "${PROJECT_VERSION_MAJOR}" 1)
    setver(app_VERSION_MINOR "${PROJECT_VERSION_MINOR}" 0)
    setver(app_VERSION_PATCH "${PROJECT_VERSION_PATCH}" 0)
    setver(app_VERSION_TWEAK "${PROJECT_VERSION_TWEAK}" 0)

    # Define the reset of the versioninfo data.
    set(app_COMPANY_NAME        "")
    set(app_FILE_DESCRIPTION    "")
    set(app_FILE_VERSION        "${app_version_string}")
    set(app_INTERNAL_NAME       "${CMAKE_PROJECT_NAME}")
    set(app_LEGAL_COPYRIGHT     "${app_copyright}")
    set(app_LEGAL_TRADEMARK     "${app_trademark}")
    set(app_ORIGINAL_FILENAME   "${CMAKE_PROJECT_NAME}.exe")
    set(app_PRODUCT_NAME        "${CMAKE_PROJECT_NAME}")
    set(app_PRODUCT_VERSION     "${app_version_string}")
    set(app_ICON                "${PROJECT_SOURCE_DIR}/assets/icon.ico")

    # Create the windows VERSIONINFO resource file.
    configure_file(version_info.rc.in version_info.rc @ONLY)
    target_sources(acquisition PUBLIC "${CMAKE_BINARY_DIR}/version_info.rc")
        
elseif(APPLE)
    
    set(macosx_app_version "${PROJECT_VERSION_MAJOR}.${PROJECT_VERSION_MINOR}.${PROJECT_VERSION_PATCH}")

    # Setup the macOS bundle
    set_target_properties(acquisition PROPERTIES
        MACOSX_BUNDLE_BUNDLE_NAME "Acquisition"
        MACOSX_BUNDLE_BUNDLE_VERSION "${macosx_app_version}"
        MACOSX_BUNDLE_COPYRIGHT "${app_copyright}"
        #MACOSX_BUNDLE_GUI_IDENTIFIER ""
        MACOSX_BUNDLE_ICON_FILE "icon.icns"
        MACOSX_BUNDLE_INFO_STRING "${PROJECT_DESCRIPTION}"
        MACOSX_BUNDLE_SHORT_VERSION_STRING "${macosx_app_version}"
    )

    # Add the macOS application icon
    set(macos_app_icon "${PROJECT_SOURCE_DIR}/assets/icon.icns")
    set_source_files_properties("${macos_app_icon}" PROPERTIES MACOSX_PACKAGE_LOCATION "Resources")
    target_sources(acquisition PRIVATE "${macos_app_icon}")
    
    # Generate debugging symbols file.
    target_compile_options(acquisition PRIVATE "-g")
    set_target_properties(acquisition PROPERTIES
        XCODE_ATTRIBUTE_GCC_GENERATE_DEBUGGING_SYMBOLS "YES"
        XCODE_ATTRIBUTE_DEBUG_INFORMATION_FORMAT "dwarf-with-dsym"
        XCODE_ATTRIBUTE_DEPLOYMENT_POSTPROCESING "YES"
        XCODE_ATTRIBUTE_STRIP_INSTALLED_PRODUCT "YES"
    )
    set(macos_exe "${CMAKE_BINARY_DIR}/acquisition.app/Contents/MacOS/acquisition")
    set(macos_dsym "${CMAKE_BINARY_DIR}/acquisition_${app_version_string}.dSYM")
    add_custom_command(TARGET acquisition
        POST_BUILD
        COMMAND xcrun dsymutil "${macos_exe}" -o "${macos_dsym}"
    )
    
elseif(UNIX)

    # Setup OpenSSL using the version that is distributed with Qt Creator
    set(QT_OPENSSL_DIR "$ENV{HOME}/Qt/Tools/OpenSSLv3/src")

    # First define the library as imported targets because they are pre-built.
    add_library(libssl SHARED IMPORTED)
    add_library(libcrypto SHARED IMPORTED)

    # Second, point those libraries to the installed locations.
    set_target_properties(libssl PROPERTIES IMPORTED_LOCATION "${QT_OPENSSL_DIR}/libssl.so")
    set_target_properties(libcrypto PROPERTIES IMPORTED_LOCATION "${QT_OPENSSL_DIR}/libcrypto.so")

    # Finally, we can link acquisition against the OpenSSL libraries.
    target_link_libraries(acquisition PUBLIC libssl libcrypto)

    # Generate debug symbols.
    target_compile_options(acquisition PRIVATE -g)
    add_custom_command(TARGET acquisition
        POST_BUILD
        COMMAND objcopy --only-keep-debug acquisition acquisition.debug
        COMMAND strip --strip-debug --strip-unneeded acquisition
        COMMAND objcopy --add-gnu-debuglink=acquisition.debug acquisition
        COMMAND chmod -x acquisition.debug
    )
    
endif()
