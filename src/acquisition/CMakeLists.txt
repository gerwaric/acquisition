qt_add_executable(acquisition WIN32 MACOSX_BUNDLE
    # Source
    acquisition/application.cpp
    acquisition/bucket.cpp
    acquisition/buyout.cpp
    acquisition/buyoutmanager.cpp
    acquisition/column.cpp
    acquisition/crashpad.cpp
    acquisition/currency.cpp
    acquisition/currencymanager.cpp
    acquisition/datastore.cpp
    acquisition/fatalerror.cpp
    acquisition/filters.cpp
    acquisition/flowlayout.cpp
    acquisition/imagecache.cpp
    acquisition/influence.cpp
    acquisition/item.cpp
    acquisition/itemcategories.cpp
    acquisition/itemconstants.cpp
    acquisition/itemlocation.cpp
    acquisition/items_model.cpp
    acquisition/itemsmanager.cpp
    acquisition/itemsmanagerworker.cpp
    acquisition/itemtooltip.cpp
    acquisition/logindialog.cpp
    acquisition/logpanel.cpp
    acquisition/main.cpp
    acquisition/mainwindow.cpp
    acquisition/memorydatastore.cpp
    acquisition/modlist.cpp
    acquisition/modsfilter.cpp
    acquisition/networkmanager.cpp
    acquisition/oauthmanager.cpp
    acquisition/oauthtoken.cpp
    acquisition/ratelimit.cpp
    acquisition/ratelimitdialog.cpp
    acquisition/ratelimiter.cpp
    acquisition/ratelimitmanager.cpp
    acquisition/repoe.cpp
    acquisition/search.cpp
    acquisition/searchcombobox.cpp
    acquisition/shop.cpp
    acquisition/sqlitedatastore.cpp
    acquisition/updatechecker.cpp
    acquisition/util.cpp
    acquisition/verticalscrollarea.cpp
    test/testdata.cpp
    test/testitem.cpp
    test/testitemsmanager.cpp
    test/testmain.cpp
    test/testshop.cpp
    test/testutil.cpp
    # Headers
    acquisition/application.h
    acquisition/bucket.h
    acquisition/buyout.h
    acquisition/buyoutmanager.h
    acquisition/column.h
    acquisition/crashpad.h
    acquisition/currency.h
    acquisition/currencymanager.h
    acquisition/datastore.h
    acquisition/fatalerror.h
    acquisition/filters.h
    acquisition/flowlayout.h
    acquisition/imagecache.h
    acquisition/influence.h
    acquisition/item.h
    acquisition/itemcategories.h
    acquisition/itemconstants.h
    acquisition/itemlocation.h
    acquisition/items_model.h
    acquisition/itemsmanager.h
    acquisition/itemsmanagerworker.h
    acquisition/itemtooltip.h
    acquisition/logindialog.h
    acquisition/logpanel.h
    acquisition/mainwindow.h
    acquisition/memorydatastore.h
    acquisition/modlist.h
    acquisition/modsfilter.h
    acquisition/network_info.h
    acquisition/networkmanager.h
    acquisition/oauthmanager.h
    acquisition/oauthtoken.h
    acquisition/rapidjson_util.h
    acquisition/ratelimit.h
    acquisition/ratelimitdialog.h
    acquisition/ratelimiter.h
    acquisition/ratelimitmanager.h
    acquisition/replytimeout.h
    acquisition/repoe.h
    acquisition/search.h
    acquisition/searchcombobox.h
    acquisition/shop.h
    acquisition/sqlitedatastore.h
    acquisition/updatechecker.h
    acquisition/util.h
    acquisition/verticalscrollarea.h
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
    # Generated version header
    "${CMAKE_BINARY_DIR}/version_defines.h"
)

target_sources(acquisition PRIVATE
    "${PROJECT_SOURCE_DIR}/resources/resources.qrc"
)

# Setup include directories.
target_include_directories(acquisition PRIVATE
    "${CMAKE_CURRENT_SOURCE_DIR}"
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
    boost-headers-only
    Crashpad
    qdarkstyle
    QsLog
    rapidjson
    semver
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
        acquisition/checkmsvc.cpp
        acquisition/checkmsvc.h
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
configure_file("acquisition/version_defines.h.in" "version_defines.h" @ONLY)

#----------------------------------
# Platform-specific section
#----------------------------------

if(APPLE)
    
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
