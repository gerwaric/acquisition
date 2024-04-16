cmake_minimum_required(VERSION 3.19)

project(acquisition
    VERSION 0.10.4
    DESCRIPTION "Stash and shop management for Path of Exile"
    LANGUAGES CXX)

set(CMAKE_AUTOUIC ON)
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)

set(CMAKE_AUTOUIC_SEARCH_PATHS forms/)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(QT NAMES Qt6 REQUIRED COMPONENTS Widgets)
find_package(Qt6 REQUIRED COMPONENTS
    Gui
    Httpserver
    Network
    Sql
    Test
    Websockets
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
    src/shop.cpp
    src/sqlitedatastore.cpp
    src/updatechecker.cpp
    src/util.cpp
    src/verticalscrollarea.cpp
    # Test code
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
    src/selfdestructingreply.h
    src/shop.h
    src/sqlitedatastore.h
    src/updatechecker.h
    src/util.h
    src/version_defines.h
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
    # Resource
    resources.qrc
    deps/qdarkstyle/dark/darkstyle.qrc
    deps/qdarkstyle/light/lightstyle.qrc
    # QsLog (This should be in another file)
    deps/QsLog/QsLog.cpp
    deps/QsLog/QsLogDest.cpp
    deps/QsLog/QsLogDestConsole.cpp
    deps/QsLog/QsLogDestFile.cpp
    deps/QsLog/QsLogDestFunctor.cpp
    deps/QsLog/QsLog.h
    deps/QsLog/QsLogDest.h
    deps/QsLog/QsLogDestConsole.h
    deps/QsLog/QsLogDestFile.h
    deps/QsLog/QsLogDestFunctor.h
    deps/QsLog/QsLogDisableForThisFile.h
    deps/QsLog/QsLogLevel.h
)

target_include_directories(acquisition PRIVATE
    deps/
    deps/boost-headers-only/
    deps/QsLog/
    src/
    test/
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
)

target_compile_definitions(acquisition PRIVATE
    MY_TARGET=\"${CMAKE_PROJECT_NAME}\"
    MY_VERSION=\"${CMAKE_PROJECT_VERSION}\"
)

# Visual Studio 2022 needs this to work with boost somehow.
if (MSVC)
	target_compile_options(acquisition PRIVATE "/EHsc")
endif()

# Qt for iOS sets MACOSX_BUNDLE_GUI_IDENTIFIER automatically since Qt 6.1.
# If you are developing for iOS or macOS you should consider setting an
# explicit, fixed bundle identifier manually though.
if (${QT_VERSION} VERSION_LESS 6.1.0)
  set(BUNDLE_ID_OPTION MACOSX_BUNDLE_GUI_IDENTIFIER com.example.acquisition)
endif()

set_target_properties(acquisition PROPERTIES
    ${BUNDLE_ID_OPTION}
    MACOSX_BUNDLE_BUNDLE_VERSION ${PROJECT_VERSION}
    MACOSX_BUNDLE_SHORT_VERSION_STRING ${PROJECT_VERSION_MAJOR}.${PROJECT_VERSION_MINOR}
    MACOSX_BUNDLE TRUE
    WIN32_EXECUTABLE TRUE
)
