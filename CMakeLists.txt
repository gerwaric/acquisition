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
    VERSION 0.12.0
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

add_subdirectory(external/boost-headers-only)
add_subdirectory(external/Crashpad)
add_subdirectory(external/qdarkstyle)
add_subdirectory(external/QsLog)
add_subdirectory(external/rapidjson)
add_subdirectory(external/semver)

add_subdirectory(src/acquisition)


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
    target_sources(acquisition PRIVATE "${PROJECT_SOURCE_DIR}/resources/icons/icon.ico")

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
    set(app_ICON                "${PROJECT_SOURCE_DIR}/resources/icons/icon.ico")

    # Create the windows VERSIONINFO resource file.
    configure_file(version_info.rc.in version_info.rc @ONLY)
    target_sources(acquisition PUBLIC "${CMAKE_BINARY_DIR}/version_info.rc")

endif()
