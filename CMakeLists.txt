cmake_minimum_required(VERSION 3.19)

project(acquisition
    VERSION 0.10.4
    DESCRIPTION "Stash and forum shop management for Path of Exile"
    HOMEPAGE_URL "https://github.com/gerwaric/acquisition"
    LANGUAGES CXX
)

# Use this to define pre-releases
set(version_postfix "-alpha.6")

# Variables used to generate version_defines.h and installer.iss
set(version_code        58)
set(app_name            "${CMAKE_PROJECT_NAME}")
set(app_version         "${CMAKE_PROJECT_VERSION}")
set(app_version_string  "${CMAKE_PROJECT_VERSION}${version_postfix}")
set(app_url             "${CMAKE_PROJECT_HOMEPAGE_URL}")
set(app_publisher       "GERWARIC")
set(app_publisher_email "gerwaric@gmail.com")

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
    src/buyoutmanager.cpp
    src/column.cpp
    src/currencymanager.cpp
    src/datastore.cpp
    src/filesystem.cpp
    src/filters.cpp
    src/flowlayout.cpp
    src/imagecache.cpp
    src/influence.cpp
    src/item.cpp
    src/itemcategories.cpp
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
    src/oauth.cpp
    src/porting.cpp
    src/ratelimit.cpp
    src/ratelimitpanel.cpp
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
    src/buyoutmanager.h
    src/column.h
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
    src/oauth.h
    src/porting.h
    src/rapidjson_util.h
    src/ratelimit.h
    src/ratelimitpanel.h
    src/replytimeout.h
    src/search.h
    src/searchcombobox.h
    src/selfdestructingreply.h
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
    set(ver_FILEVERSION        "${PROJECT_VERSION_MAJOR},${PROJECT_VERSION_MINOR},${PROJECT_VERSION_PATCH}")
    set(ver_FILEVERSION_STR    "${app_version_string}")
    set(ver_PRODUCTVERSION     "${PROJECT_VERSION_MAJOR},${PROJECT_VERSION_MINOR},${PROJECT_VERSION_PATCH}")
    set(ver_PRODUCTVERSION_STR "${app_version_string}")
    set(ver_COMPANYNAME_STR    "")
    set(ver_PRODUCTNAME_STR    "${PROJECT_NAME}")
    set(ver_LEGALCOPYRIGHT_STR "Copyright \\251 2014 Ilya Zhuravlev and 2023 Gerwaric")
    set(ver_FILENAME_STR       "${PROJECT_NAME}.exe")
    set(ver_NAME_STR           "${PROJECT_NAME}")
    set(ver_FILEDESC_STR       "${PROJECT_DESCRIPTION}")
    configure_file(version_info.rc.in version_info.rc @ONLY)
    target_sources(acquisition PUBLIC "${CMAKE_CURRENT_BINARY_DIR}/version_info.rc")
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
if(UNIX)
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
    target_compile_options(acquisition PRIVATE "/EHsc")

    # Visual Studio 2022 needs for debugging to work.
    target_compile_options(acquisition PRIVATE "$<$<CONFIG:DEBUG>:/ZI>")

endif()

set_target_properties(acquisition PROPERTIES
    ${BUNDLE_ID_OPTION}
    MACOSX_BUNDLE_BUNDLE_VERSION ${PROJECT_VERSION}
    MACOSX_BUNDLE_SHORT_VERSION_STRING ${PROJECT_VERSION_MAJOR}.${PROJECT_VERSION_MINOR}
    MACOSX_BUNDLE TRUE
    WIN32_EXECUTABLE TRUE
)
