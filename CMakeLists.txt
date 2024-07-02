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
set(version_postfix "-alpha.1")

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

set(CMAKE_AUTOUIC ON)
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)

set(CMAKE_AUTOUIC_SEARCH_PATHS forms/)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(Qt6 6.5.3 REQUIRED COMPONENTS
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
)

# Automatically populate and include the versions header file.
configure_file(src/version_defines.h.in version_defines.h @ONLY)
target_sources(acquisition PUBLIC "${CMAKE_CURRENT_BINARY_DIR}/version_defines.h")
target_include_directories(acquisition PUBLIC "${CMAKE_CURRENT_BINARY_DIR}")

# Automatically populate the VERSIONINFO file for windows.
if(WIN32)
    
    # Define the major version.
	if (NOT PROJECT_VERSION_MAJOR STREQUAL "")
        set(app_VERSION_MAJOR "${PROJECT_VERSION_MAJOR}")
    else()
        set(app_VERSION_MAJOR, 1)
    endif()
    
    # Define the minor version.
    if (NOT PROJECT_VERSION_MINOR STREQUAL "")
        set(app_VERSION_MINOR "${PROJECT_VERSION_MINOR}")
    else()
        set(app_VERSION_MINOR 0)
    endif()
    
    # Define the patch version.
    if (NOT PROJECT_VERSION_PATCH STREQUAL "")
        set(app_VERSION_PATCH "${PROJECT_VERSION_PATCH}")
    else()
        set(app_VERSION_PATCH 0)
    endif()
    
    # Define the tweak version.
    if (NOT PROJECT_VERSION_TWEAK STREQUAL "")
        set(app_VERSION_TWEAK "${PROJECT_VERSION_TWEAK}")
    else()
        set(app_VERSION_TWEAK 0)
    endif()
    
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
    ${CMAKE_CURRENT_BINARY_DIR} # needed so files can include verion_defines.h
)

target_link_libraries(acquisition PUBLIC
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

target_link_libraries(QsLog PUBLIC
    Qt::Core
)

# Add dependency: darkstyle
add_library(qdarkstyle STATIC
    deps/qdarkstyle/dark/darkstyle.qrc
    deps/qdarkstyle/light/lightstyle.qrc
)

if(MSVC)
	
    # Visual Studio 2022 needs this to work with boost.
    target_compile_options(acquisition PRIVATE /EHsc /W4)

    # Visual Studio 2022 needs for debugging to work.
    target_compile_options(acquisition PRIVATE "$<$<CONFIG:DEBUG>:/Zi>")

endif()

if(APPLE)

    # Add the macOS application icon
    set(MACOSX_BUNDLE_ICON_FILE icon.icns)
    set(macos_app_icon "${CMAKE_CURRENT_SOURCE_DIR}/assets/icon.icns")
    set_source_files_properties(${macos_app_icon} PROPERTIES MACOSX_PACKAGE_LOCATION "Resources")
    target_sources(acquisition PUBLIC "${macos_app_icon}")

    # On macOS we need to enable unary functions for version of boost we are using.
    target_compile_definitions(acquisition PUBLIC _LIBCPP_ENABLE_CXX17_REMOVED_UNARY_BINARY_FUNCTION)

endif()

set_target_properties(acquisition PROPERTIES
    ${BUNDLE_ID_OPTION}
    MACOSX_BUNDLE_BUNDLE_VERSION ${PROJECT_VERSION}
    MACOSX_BUNDLE_SHORT_VERSION_STRING ${PROJECT_VERSION_MAJOR}.${PROJECT_VERSION_MINOR}
    MACOSX_BUNDLE TRUE
    WIN32_EXECUTABLE TRUE
)
