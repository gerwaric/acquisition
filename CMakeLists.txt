#[[
	Copyright 2014 Ilya Zhuravlev and 2024 Gerwaric

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

cmake_minimum_required(VERSION 3.21.1)

# NOTE: For automatic update detection to work, the 'app_version' variable
# defined below should match a tag in github. Historically, the github tags
# have been identical to the 'app_version' as defined below, except the 
# tags on github are prefixed with a 'v'.
#
# The version code that used to be used for update detection is no longer
# needed, except that some users with version of acquisition prior to
# version 0.10.5 will still need it, so we can't outright get rid of it.

project(acquisition
    VERSION 0.11.1
    DESCRIPTION "Stash and forum shop management for Path of Exile \\256"
    HOMEPAGE_URL "https://github.com/gerwaric/acquisition"
    LANGUAGES CXX
)

# Use this to define pre-releases
set(version_postfix "-alpha.2")

# Variables used to generate version_defines.h and installer.iss
set(version_code        61) # This doesn't need to be changed with the new update checks.
set(app_name            "${CMAKE_PROJECT_NAME}")
set(app_version         "${CMAKE_PROJECT_VERSION}")
set(app_version_string  "${CMAKE_PROJECT_VERSION}${version_postfix}")
set(app_url             "${CMAKE_PROJECT_HOMEPAGE_URL}")
set(app_publisher       "GERWARIC")
set(app_publisher_email "gerwaric@gmail.com")
set(app_copyright       "Copyright \\251 2014 Ilya Zhuravlev and 2023 Gerwaric")
set(app_trademark       "Path of Exile is a trademark of Grinding Gear Games, Ltd.")

include(GNUInstallDirs)

set(CMAKE_AUTOUIC ON)
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)

set(CMAKE_AUTOUIC_SEARCH_PATHS forms/)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(Qt6 6.5.3 REQUIRED COMPONENTS
    Core
    Gui
    HttpServer
    Network
    Sql
    Test
    WebSockets
    Widgets
)

qt_standard_project_setup()

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
    src/filesystem.cpp
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
    src/oauthmanager.cpp
    src/oauthtoken.cpp
    src/ratelimit.cpp
    src/ratelimitdialog.cpp
    src/ratelimiter.cpp
    src/ratelimitmanager.cpp
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
    src/filesystem.h
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
    src/oauthmanager.h
    src/oauthtoken.h
    src/rapidjson_util.h
    src/ratelimit.h
    src/ratelimitdialog.h
    src/ratelimiter.h
    src/ratelimitmanager.h
    src/replytimeout.h
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
    "${CMAKE_CURRENT_BINARY_DIR}/version_defines.h"
)

# Automatically populate and include the versions header file.
configure_file("src/version_defines.h.in" "version_defines.h" @ONLY)

# Automatically populate the VERSIONINFO file for windows.
if(WIN32)

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
    set(app_COMPANY_NAME	    "")
    set(app_FILE_DESCRIPTION    "")
    set(app_FILE_VERSION        "${app_version_string}")
    set(app_INTERNAL_NAME       "${CMAKE_PROJECT_NAME}")
    set(app_LEGAL_COPYRIGHT     "${app_copyright}")
    set(app_LEGAL_TRADEMARK     "${app_trademark}")
    set(app_ORIGINAL_FILENAME   "${CMAKE_PROJECT_NAME}.exe")
    set(app_PRODUCT_NAME        "${CMAKE_PROJECT_NAME}")
    set(app_PRODUCT_VERSION     "${app_version_string}")
    set(app_ICON                "${PROJECT_SOURCE_DIR}/assets/icon.ico")
    configure_file(version_info.rc.in version_info.rc @ONLY)
    target_sources(acquisition PUBLIC
        "${CMAKE_CURRENT_BINARY_DIR}/version_info.rc"
        "${PROJECT_SOURCE_DIR}/assets/icon.ico"
    )
endif()

target_include_directories(acquisition PRIVATE
    deps/
    deps/QsLog/
    deps/boost-headers-only/
    src/
    test/
    "${CMAKE_CURRENT_BINARY_DIR}"
)

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
    deps/QsLog/
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

get_target_property(CrashpadHandlerPath CrashpadHandler IMPORTED_LOCATION)
add_custom_command(TARGET acquisition
    POST_BUILD
    COMMAND "${CMAKE_COMMAND}" -E copy "${CrashpadHandlerPath}" "${CMAKE_CURRENT_BINARY_DIR}"
)

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
)

# On Linux we need to build with OpenSSL 3.x
if(UNIX AND (NOT APPLE))
    set(OPENSSL_ROOT_DIR ~/openssl-3.1.5)
    set(OPENSSL_USE_STATIC_LIBS TRUE)
    find_package(OpenSSL 3.1.5 REQUIRED)
    target_link_libraries(acquisition PUBLIC
        OpenSSL::SSL
        OpenSSL::Crypto
    )
endif()

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

target_link_libraries(QsLog PRIVATE
    Qt::Core
)

# Add dependency: darkstyle
add_library(qdarkstyle STATIC
    deps/qdarkstyle/dark/darkstyle.qrc
    deps/qdarkstyle/light/lightstyle.qrc
)

if(MSVC)
    
    # Visual Studio 2022 needs this to work with boost.
    target_compile_options(acquisition PRIVATE /EHsc)

    # Enable more warnings than default
    target_compile_options(acquisition PRIVATE /W4)

    # Create .pdb files for debugging
    target_compile_options(acquisition PRIVATE /Zi)
    
    # Tell linker to include symbol data
    set_target_properties(acquisition PROPERTIES
        LINK_FLAGS "/INCREMENTAL:NO /DEBUG /OPT:REF /OPT:ICF"
        COMPILE_PDB_NAME acquisition
        COMPILE_PDB_OUTPUT_DIR "${CMAKE_BINARY_DIR}"
    )

endif()

if(APPLE)

    # Add the macOS application icon
    set(MACOSX_BUNDLE_ICON_FILE icon.icns)
    set(macos_app_icon "${CMAKE_CURRENT_SOURCE_DIR}/assets/icon.icns")
    set_source_files_properties("${macos_app_icon}" PROPERTIES MACOSX_PACKAGE_LOCATION "Resources")
    target_sources(acquisition PRIVATE "${macos_app_icon}")

    # Generate debugging symbols file.
    target_compile_options(acquisition PRIVATE -g)
    set_target_properties(acquisition PROPERTIES
        XCODE_ATTRIBUTE_GCC_GENERATE_DEBUGGING_SYMBOLS "YES"
        XCODE_ATTRIBUTE_DEBUG_INFORMATION_FORMAT "dwarf-with-dsym"
        XCODE_ATTRIBUTE_DEPLOYMENT_POSTPROCESING "YES"
        XCODE_ATTRIBUTE_STRIP_INSTALLED_PRODUCT "YES"
    )
    add_custom_command(TARGET acquisition
        POST_BUILD
        COMMAND xcrun dsymutil acquisition.app/Contents/MacOS/acquisition -o acquisition.dSYM
    )


endif()

set_target_properties(acquisition PROPERTIES
    ${BUNDLE_ID_OPTION}
    MACOSX_BUNDLE_BUNDLE_VERSION ${PROJECT_VERSION}
    MACOSX_BUNDLE_SHORT_VERSION_STRING ${PROJECT_VERSION_MAJOR}.${PROJECT_VERSION_MINOR}
    MACOSX_BUNDLE TRUE
    WIN32_EXECUTABLE TRUE
)
