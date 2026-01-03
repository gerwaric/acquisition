# SPDX-License-Identifier: GPL-3.0-or-later
# SPDX-FileCopyrightText: 2024 Tom Holz

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
    VERSION 0.16.0
    DESCRIPTION "Stash and forum shop management for Path of Exile (TM)"
    HOMEPAGE_URL "https://github.com/gerwaric/acquisition"
    LANGUAGES CXX
)

# Use this to define pre-releases
set(version_postfix "-alpha.1")

# Variables used to generate version_defines.h and installer.iss
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
set(CMAKE_CXX_EXTENSIONS OFF)

# When building with Qt Creator in Windows, use target-scoped flags so
# dependencies don't inherit MSVC options unintentionally.

if(WIN32)

    # Hard-code the MSCV runtime required, because knowing when the runtime really
    # needs to be updated is tricky
    set(msvc_runtime_minimum_version "14.42")

endif()

include(FetchContent)

find_package(Qt6 6.10 REQUIRED COMPONENTS Core Gui Network NetworkAuth Sql Widgets)

if(NOT TARGET sentry-native)
    FetchContent_Declare(
        sentry-native
        GIT_REPOSITORY https://github.com/getsentry/sentry-native
        GIT_TAG        0.12.2
        GIT_SHALLOW    TRUE
    )
    FetchContent_MakeAvailable(sentry-native)
endif()

if(NOT TARGET glaze)
    FetchContent_Declare(
        glaze
        GIT_REPOSITORY https://github.com/stephenberry/glaze.git
        GIT_TAG        v6.4.0
        GIT_SHALLOW    TRUE
    )
    FetchContent_MakeAvailable(glaze)
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

#------------------------------------------------------
# Acquisition Sources
#------------------------------------------------------

set(ACQ_CORE
    src/application.cpp
    src/application.h
    src/bucket.cpp
    src/bucket.h
    src/buyout.cpp
    src/buyout.h
    src/buyoutmanager.cpp
    src/buyoutmanager.h
    src/column.cpp
    src/column.h
    src/currency.cpp
    src/currency.h
    src/currencymanager.cpp
    src/currencymanager.h
    src/filters.cpp
    src/filters.h
    src/imagecache.cpp
    src/imagecache.h
    src/influence.cpp
    src/influence.h
    src/item.cpp
    src/item.h
    src/itemcategories.cpp
    src/itemcategories.h
    src/itemconstants.cpp
    src/itemconstants.h
    src/itemlocation.cpp
    src/itemlocation.h
    src/items_model.cpp
    src/items_model.h
    src/itemsmanager.cpp
    src/itemsmanager.h
    src/itemsmanagerworker.cpp
    src/itemsmanagerworker.h
    src/main.cpp
    src/modlist.cpp
    src/modlist.h
    src/modsfilter.cpp
    src/modsfilter.h
    src/network_info.h
    src/pseudomods.cpp
    src/pseudomods.h
    src/replytimeout.h
    src/search.cpp
    src/search.h
    src/shop.cpp
    src/shop.h
)

set(ACQ_DATASTORE
    src/datastore/characterrepo.cpp
    src/datastore/characterrepo.h
    src/datastore/datastore.cpp
    src/datastore/datastore.h
    src/datastore/datastore_utils.cpp
    src/datastore/datastore_utils.h
    src/datastore/memorydatastore.cpp
    src/datastore/memorydatastore.h
    src/datastore/sqlitedatastore.cpp
    src/datastore/sqlitedatastore.h
    src/datastore/stashrepo.cpp
    src/datastore/stashrepo.h
    src/datastore/userstore.cpp
    src/datastore/userstore.h
)

set(ACQ_LEGACY
    src/legacy/legacybuyout.h
    src/legacy/legacycharacter.h
    src/legacy/legacycurrency.h
    src/legacy/legacydatastore.cpp
    src/legacy/legacydatastore.h
    src/legacy/legacyitem.cpp
    src/legacy/legacyitem.h
    src/legacy/legacyitemlocation.h
    src/legacy/legacystash.h
)

set(ACQ_RATELIMIT
    src/ratelimit/ratelimit.cpp
    src/ratelimit/ratelimit.h
    src/ratelimit/ratelimitdialog.cpp
    src/ratelimit/ratelimitdialog.h
    src/ratelimit/ratelimitedreply.h
    src/ratelimit/ratelimitedrequest.cpp
    src/ratelimit/ratelimitedrequest.h
    src/ratelimit/ratelimiter.cpp
    src/ratelimit/ratelimiter.h
    src/ratelimit/ratelimitmanager.cpp
    src/ratelimit/ratelimitmanager.h
    src/ratelimit/ratelimitpolicy.cpp
    src/ratelimit/ratelimitpolicy.h
)

set(ACQ_REPOE
    src/repoe/repoe.cpp
    src/repoe/repoe.h
    src/repoe/baseitem.h
    src/repoe/itemclass.h
    src/repoe/stattranslation.h
)

set(ACQ_POE
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
)

set(ACQ_UI
    src/ui/flowlayout.cpp
    src/ui/flowlayout.h
    src/ui/itemtooltip.cpp
    src/ui/itemtooltip.h
    src/ui/logindialog.cpp
    src/ui/logindialog.h
    src/ui/logpanel.cpp
    src/ui/logpanel.h
    src/ui/mainwindow.cpp
    src/ui/mainwindow.h
    src/ui/searchcombobox.cpp
    src/ui/searchcombobox.h
    src/ui/verticalscrollarea.cpp
    src/ui/verticalscrollarea.h
    # Forms
    src/ui/logindialog.ui
    src/ui/mainwindow.ui
)

set(ACQ_UTIL
    src/util/checkmsvc.cpp
    src/util/checkmsvc.h
    src/util/fatalerror.cpp
    src/util/fatalerror.h
    src/util/glaze_qt.h
    src/util/json_readers.cpp
    src/util/json_readers.h
    src/util/json_writers.cpp
    src/util/json_writers.h
    src/util/logging.cpp
    src/util/logging.h
    src/util/networkmanager.cpp
    src/util/networkmanager.h
    src/util/oauthmanager.cpp
    src/util/oauthmanager.h
    src/util/oauthtoken.cpp
    src/util/oauthtoken.h
    src/util/spdlog_qt.h
    src/util/updatechecker.cpp
    src/util/updatechecker.h
    src/util/util.cpp
    src/util/util.h
)

set(ACQ_RESOURCES
    resources.qrc
)

qt_standard_project_setup(REQUIRES 6.10.0)

qt_add_executable(acquisition WIN32 MACOSX_BUNDLE
    ${ACQ_CORE}
    ${ACQ_DATASTORE}
    ${ACQ_LEGACY}
    ${ACQ_RATELIMIT}
    ${ACQ_REPOE}
    ${ACQ_POE}
    ${ACQ_UI}
    ${ACQ_UTIL}
    ${ACQ_RESOURCES}
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
    # Extenal libraries
    glaze::glaze
    qdarkstyle
    sentry
    semver
    spdlog::spdlog_header_only
)

target_compile_features(acquisition PRIVATE cxx_std_23)

#----------------------------------
# Platform-specific section
#----------------------------------

if(WIN32)

    # Needed to check the MSVC runtime version
    target_link_libraries(acquisition PRIVATE Version)

    # Enable more warnings by default (MSVC only).
    target_compile_options(acquisition PRIVATE
        $<$<CXX_COMPILER_ID:MSVC>:/W4>
        $<$<CXX_COMPILER_ID:MSVC>:/EHsc>
        $<$<CXX_COMPILER_ID:MSVC>:/D_DISABLE_CONSTEXPR_MUTEX_CONSTRUCTOR>
        $<$<AND:$<CXX_COMPILER_ID:MSVC>,$<NOT:$<CONFIG:Debug>>>:/Zi>
    )
    target_link_options(acquisition PRIVATE
        $<$<AND:$<CXX_COMPILER_ID:MSVC>,$<NOT:$<CONFIG:Debug>>>:/DEBUG>
        $<$<AND:$<CXX_COMPILER_ID:MSVC>,$<NOT:$<CONFIG:Debug>>>:/OPT:REF>
        $<$<AND:$<CXX_COMPILER_ID:MSVC>,$<NOT:$<CONFIG:Debug>>>:/OPT:ICF>
    )

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
    target_link_libraries(acquisition PRIVATE
        OpenSSL::SSL
        OpenSSL::Crypto)

    # Generate debug symbols.
    target_compile_options(acquisition PRIVATE -g)
    if(CMAKE_OBJCOPY AND CMAKE_STRIP)
        add_custom_command(TARGET acquisition
            POST_BUILD
            COMMAND ${CMAKE_OBJCOPY} --only-keep-debug acquisition acquisition.debug
            COMMAND ${CMAKE_STRIP} --strip-debug --strip-unneeded acquisition
            COMMAND ${CMAKE_OBJCOPY} --add-gnu-debuglink=acquisition.debug acquisition
            COMMAND chmod -x acquisition.debug
        )
    endif()
    
endif()
