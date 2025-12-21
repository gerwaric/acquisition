#[[
	Copyright (C) 2014-2025 Acquisition Contributors

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

cmake_minimum_required(VERSION 3.30)

# NOTE: For automatic update detection to work, the 'app_version' variable
# defined below should match a tag in github. Historically, the github tags
# have been identical to the 'app_version' as defined below, except the 
# tags on github are prefixed with a 'v'.
#
# The version code that used to be used for update detection is no longer
# needed, except that some users with version of acquisition prior to
# version 0.10.5 will still need it, so we can't outright get rid of it.

project(acquisition
    VERSION 0.14.3
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
set(app_copyright       "Copyright (C) 2014-2025 Ilya Zhuravlev and other Acquisition Contributors")
set(app_trademark       "Path of Exile is a trademark of Grinding Gear Games, Ltd.")

set(CMAKE_AUTOUIC ON)
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# When building with Qt Creator in Windows, the compiler and linker options
# need to be set before qt_add_executable(), otherwise these options won't be used.
# (This is apparently not an issue with Visual Studio 2022).

if(WIN32)

    # Hard-code the MSCV runtime required, because knowing when the runtime really
    # needs to be updated is tricky
    set(msvc_runtime_minimum_version "14.42")

    # Enable more warnings by default
    add_compile_options(/W4)
    add_compile_options(/EHsc)

    # There was a change to mutex constructors at some point
    # that was causing random crashes, which can be avoided
    # with this preprocessor flag.
    add_compile_options(/D_DISABLE_CONSTEXPR_MUTEX_CONSTRUCTOR)

    # Create .pdb files for debugging
    add_compile_options("$<$<NOT:$<CONFIG:Debug>>:/Zi>")
    add_link_options("$<$<NOT:$<CONFIG:Debug>>:/DEBUG>")
    add_link_options("$<$<NOT:$<CONFIG:Debug>>:/OPT:REF>")
    add_link_options("$<$<NOT:$<CONFIG:Debug>>:/OPT:ICF>")

endif()

include(FetchContent)

find_package(Qt6 REQUIRED COMPONENTS Core Gui Network NetworkAuth Sql Test Widgets)

if(NOT TARGET glaze)
    FetchContent_Declare(
        glaze
        GIT_REPOSITORY https://github.com/stephenberry/glaze.git
        GIT_TAG        v6.0.1
        GIT_SHALLOW    TRUE
    )
    FetchContent_MakeAvailable(glaze)
endif()

if(NOT TARGET rapidjson)
    # This is a headers-only library, so we use an invalid SOURCE_DIR to prevent
    # add_subdirectory() from called and define an interface target manually.
    FetchContent_Declare(
        rapidjson
        GIT_REPOSITORY https://github.com/tencent/rapidjson.git
        GIT_TAG        24b5e7a8b27f42fa16b96fc70aade9106cf7102f # There hasn't been a tagged release since 2016
        GIT_SHALLOW    TRUE
        SOURCE_SUBDIR  /dev/null
    )
    FetchContent_MakeAvailable(rapidjson)
    add_library(rapidjson INTERFACE)
    target_include_directories(rapidjson INTERFACE "${rapidjson_SOURCE_DIR}/include")
endif()

if(NOT TARGET semver)
    # This is a headers-only library, so we use an invalid SOURCE_DIR to prevent
    # add_subdirectory() from called and define an interface target manually.
    FetchContent_Declare(
        semver
        GIT_REPOSITORY https://github.com/z4kn4fein/cpp-semver
        GIT_TAG        v0.4.0
        GIT_SHALLOW    TRUE
        SOURCE_SUBDIR  /dev/null
    )
    FetchContent_MakeAvailable(semver)
    add_library(semver INTERFACE)
    target_include_directories(semver INTERFACE "${semver_SOURCE_DIR}/include")
endif()

if(NOT TARGET spdlog)
    FetchContent_Declare(
        spdlog
        GIT_REPOSITORY https://github.com/gabime/spdlog.git
        GIT_TAG        v1.16.0
        GIT_SHALLOW    TRUE
    )
    FetchContent_MakeAvailable(spdlog)
endif()

qt_standard_project_setup(REQUIRES 6.10.0)

qt_add_executable(acquisition WIN32 MACOSX_BUNDLE
    # Source
    src/application.cpp
    src/bucket.cpp
    src/buyout.cpp
    src/buyoutmanager.cpp
    src/column.cpp
    src/currency.cpp
    src/currencymanager.cpp
    src/datastore/datastore.cpp
    src/datastore/memorydatastore.cpp
    src/datastore/sqlitedatastore.cpp
    src/filters.cpp
    src/imagecache.cpp
    src/influence.cpp
    src/item.cpp
    src/itemcategories.cpp
    src/itemconstants.cpp
    src/itemlocation.cpp
    src/items_model.cpp
    src/itemsmanager.cpp
    src/itemsmanagerworker.cpp
    # src/legacy/legacybuyoutvalidator.cpp DISABLED as of v0.12.4
    src/legacy/legacydatastore.cpp
    src/legacy/legacyitem.cpp
    src/main.cpp
    src/modlist.cpp
    src/modsfilter.cpp
    src/ratelimit/ratelimit.cpp
    src/ratelimit/ratelimitdialog.cpp
    src/ratelimit/ratelimitedrequest.cpp
    src/ratelimit/ratelimiter.cpp
    src/ratelimit/ratelimitmanager.cpp
    src/ratelimit/ratelimitpolicy.cpp
    src/search.cpp
    src/shop.cpp
    src/ui/flowlayout.cpp
    src/ui/itemtooltip.cpp
    src/ui/logindialog.cpp
    src/ui/logpanel.cpp
    src/ui/mainwindow.cpp
    src/ui/searchcombobox.cpp
    src/ui/verticalscrollarea.cpp
    src/util/checkmsvc.cpp
    src/util/crashpad.cpp
    src/util/fatalerror.cpp
    src/util/logging.cpp
    src/util/networkmanager.cpp
    src/util/oauthmanager.cpp
    src/util/oauthtoken.cpp
    src/util/repoe.cpp
    src/util/updatechecker.cpp
    src/util/util.cpp
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
    src/datastore/datastore.h
    src/datastore/memorydatastore.h
    src/datastore/sqlitedatastore.h
    src/filters.h
    src/imagecache.h
    src/influence.h
    src/item.h
    src/itemcategories.h
    src/itemconstants.h
    src/itemlocation.h
    src/items_model.h
    src/itemsmanager.h
    src/itemsmanagerworker.h
    src/legacy/legacybuyout.h
    # src/legacy/legacybuyoutvalidator.h DISABLED as of v0.12.4
    src/legacy/legacycharacter.h
    src/legacy/legacycurrency.h
    src/legacy/legacydatastore.h
    src/legacy/legacyitem.h
    src/legacy/legacystash.h
    src/modlist.h
    src/modsfilter.h
    src/network_info.h
    src/poe/endpoints/accountcharacter.h
    src/poe/endpoints/accountcharacters.h
    src/poe/endpoints/accountleagues.h
    src/poe/endpoints/accountprofile.h
    src/poe/endpoints/accountstash.h
    src/poe/endpoints/accountstashes.h
    src/poe/endpoints/website/webcharacteritems.h
    src/poe/endpoints/website/webcharacters.h
    src/poe/endpoints/website/webleagues.h
    src/poe/endpoints/website/webstashitems.h
    src/poe/types/account.h
    src/poe/types/character.h
    src/poe/types/cruciblenode.h
    src/poe/types/displaymode.h
    src/poe/types/eventladderentry.h
    src/poe/types/frametype.h
    src/poe/types/gempage.h
    src/poe/types/gemtab.h
    src/poe/types/guild.h
    src/poe/types/item.h
    src/poe/types/itemfilter.h
    src/poe/types/itemjeweldata.h
    src/poe/types/itemproperty.h
    src/poe/types/itemsocket.h
    src/poe/types/ladderentry.h
    src/poe/types/league.h
    src/poe/types/leagueaccount.h
    src/poe/types/leaguerule.h
    src/poe/types/passivegroup.h
    src/poe/types/passivenode.h
    src/poe/types/publicstashchange.h
    src/poe/types/pvpladderteamentry.h
    src/poe/types/pvpladderteammember.h
    src/poe/types/pvpmatch.h
    src/poe/types/stashtab.h
    src/poe/types/website/webcharacter.h
    src/poe/types/website/webstashtab.h
    src/ratelimit/ratelimit.h
    src/ratelimit/ratelimitdialog.h
    src/ratelimit/ratelimitedreply.h
    src/ratelimit/ratelimitedrequest.h
    src/ratelimit/ratelimiter.h
    src/ratelimit/ratelimitmanager.h
    src/ratelimit/ratelimitpolicy.h
    src/replytimeout.h
    src/search.h
    src/shop.h
    src/ui/flowlayout.h
    src/ui/itemtooltip.h
    src/ui/logindialog.h
    src/ui/logpanel.h
    src/ui/mainwindow.h
    src/ui/searchcombobox.h
    src/ui/verticalscrollarea.h
    src/util/checkmsvc.h
    src/util/crashpad.h
    src/util/fatalerror.h
    src/util/glaze_qt.h
    # src/util/json_struct_qt.h
    src/util/logging.h
    src/util/networkmanager.h
    src/util/oauthmanager.h
    src/util/oauthtoken.h
    src/util/rapidjson_util.h
    src/util/repoe.h
    src/util/spdlog_qt.h
    src/util/updatechecker.h
    src/util/util.h
    test/testdata.h
    test/testitem.h
    test/testitemsmanager.h
    test/testmain.h
    test/testshop.h
    test/testutil.h
    # Forms
    src/ui/logindialog.ui
    src/ui/mainwindow.ui
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
    string(REGEX REPLACE "^${msvc_compiler_ver}" "${msvc_runtime_ver}" msvc_runtime_build_version ${msvc_version})
    if (NOT msvc_runtime_build_version)
        message(FATAL_ERROR "Could not determine MSVC runtime version from cl.exe output")
    endif()
    message(STATUS "MSVC Runtime Build Version: ${msvc_runtime_build_version}")
    message(STATUS "MSVC Runtime Minimum Version: ${msvc_runtime_minimum_version}")

endif()

# Automatically populate and include the versions header file.
configure_file("src/version_defines.h.in" "version_defines.h" @ONLY)

# Setup include directories.
target_include_directories(acquisition PRIVATE
    src/
    test/
    "${CMAKE_BINARY_DIR}"
)

add_subdirectory(deps/Crashpad)
add_subdirectory(deps/qdarkstyle)

# Link required libraries.
target_link_libraries(acquisition PRIVATE
    # Qt libraries
    Qt::Core
    Qt::Gui
    Qt::Network
    Qt::NetworkAuth
    Qt::Widgets
    Qt::Sql
    Qt::Test
    # Extenal libraries
    Crashpad
    glaze::glaze
    qdarkstyle
    rapidjson
    semver
    spdlog::spdlog_header_only
)

#----------------------------------
# Platform-specific section
#----------------------------------

if(WIN32)

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
        MACOSX_BUNDLE TRUE
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

    find_package(OpenSSL REQUIRED)

    # Make sure OpenSSL was found
    if (OpenSSL_FOUND)
        message(STATUS "Found OpenSSL: ${OPENSSL_VERSION}")
        message(STATUS "OpenSSL Include Dir: ${OPENSSL_INCLUDE_DIR}")
        message(STATUS "OpenSSL Libraries: ${OPENSSL_LIBRARIES}")
    else()
        message(FATAL_ERROR "OpenSSL not found on this system!")
    endif()

    # Link OpenSSL libraries to acquisition
    target_include_directories(acquisition PUBLIC "${OPENSSL_INCLUDE_DIR}")
    target_link_libraries(acquisition PUBLIC
        OpenSSL::SSL
        OpenSSL::Crypto)

    # Generate debug symbols.
    target_compile_options(acquisition PRIVATE -g)
    add_custom_command(TARGET acquisition
        POST_BUILD
        COMMAND ${CMAKE_OBJCOPY} --only-keep-debug acquisition acquisition.debug
        COMMAND ${CMAKE_STRIP} --strip-debug --strip-unneeded acquisition
        COMMAND ${CMAKE_OBJCOPY} --add-gnu-debuglink=acquisition.debug acquisition
        COMMAND chmod -x acquisition.debug
    )
    
endif()
