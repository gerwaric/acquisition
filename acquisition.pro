# First check the version of Qt.
lessThan(QT_MAJOR_VERSION, 6):error("Qt 6.5 or newer is required.")
!versionAtLeast(QT_VERSION, 6.5):error("Qt 6.5 or newer is required.");

TARGET = acquisition
TEMPLATE = app
VERSION = 0.10.0

# These defines are checked against the contents of version_defines.h at build time.
DEFINES += "MY_TARGET=\\\"$${TARGET}\\\""
DEFINES += "MY_VERSION=\\\"$${VERSION}\\\""

QT += core gui network testlib widgets httpserver websockets

win32 {
        QT.testlib.CONFIG -= console
	# Define information for the window version resource
	QMAKE_TARGET_PRODUCT = Acquisition
	QMAKE_TARGET_DESCRIPTION = "Stash management for Path of Exile"
	QMAKE_TARGET_COPYRIGHT = "Copyright © 2014 Ilya Zhuravlev and 2023 Gerwaric"
	RC_ICONS = assets/icon.ico
}

macx {
	# Needed to use boost-headers-only 1.75 with Xcode 15.0.1
	DEFINES += _LIBCPP_ENABLE_CXX17_REMOVED_UNARY_BINARY_FUNCTION
	ICON = assets/icon.icns
}

unix:!macx {
        # This may need troubleshooting
	LIBS += -ldl -L"/usr/local/ssl/lib" -l:libcrypto.a -l:libssl.a
}

include(deps/QsLog/QsLog.pri)

INCLUDEPATH += src deps deps/boost-headers-only test

SOURCES += \
	deps/sqlite/sqlite3.c \
	src/application.cpp \
	src/bucket.cpp \
	src/buyoutmanager.cpp \
	src/column.cpp \
	src/currencymanager.cpp \
	src/itemcategories.cpp \
	src/sqlitedatastore.cpp \
	src/filesystem.cpp \
	src/filters.cpp \
	src/flowlayout.cpp \
	src/imagecache.cpp \
	src/influence.cpp \
	src/item.cpp \
	src/itemlocation.cpp \
	src/items_model.cpp \
	src/itemsmanager.cpp \
	src/itemsmanagerworker.cpp \
	src/itemtooltip.cpp \
	src/logindialog.cpp \
	src/logpanel.cpp \
	src/main.cpp \
	src/mainwindow.cpp \
	src/memorydatastore.cpp \
	src/modlist.cpp \
	src/modsfilter.cpp \
	src/oauth.cpp \
	src/porting.cpp \
	src/ratelimit.cpp \
	src/ratelimitpanel.cpp \
	src/search.cpp \
	src/shop.cpp \
	src/updatechecker.cpp \
	src/util.cpp \
	src/verticalscrollarea.cpp \
	test/testdata.cpp \
	test/testitem.cpp \
	test/testitemsmanager.cpp \
	test/testmain.cpp \
	test/testshop.cpp \
	test/testutil.cpp

HEADERS += \
	deps/sqlite/sqlite3.h \
	src/application.h \
	src/bucket.h \
	src/buyoutmanager.h \
	src/column.h \
	src/currencymanager.h \
	src/datastore.h \
	src/itemcategories.h \
	src/sqlitedatastore.h \
	src/filesystem.h \
	src/filters.h \
	src/flowlayout.h \
	src/imagecache.h \
	src/influence.h \
	src/item.h \
	src/itemconstants.h \
	src/itemlocation.h \
	src/items_model.h \
	src/itemsmanager.h \
	src/itemsmanagerworker.h \
	src/itemtooltip.h \
	src/logindialog.h \
	src/logpanel.h \
	src/mainwindow.h \
	src/memorydatastore.h \
	src/modlist.h \
	src/modsfilter.h \
	src/network_info.h \
	src/oauth.h \
	src/porting.h \
	src/ratelimit.h \
	src/ratelimitpanel.h \
	src/rapidjson_util.h \
	src/replytimeout.h \
	src/search.h \
	src/selfdestructingreply.h \
	src/shop.h \
	src/updatechecker.h \
	src/util.h \
	src/version_defines.h \
	src/verticalscrollarea.h \
	test/testdata.h \
	test/testitem.h \
	test/testitemsmanager.h \
	test/testmain.h \
	test/testshop.h \
	test/testutil.h

FORMS += \
	forms/mainwindow.ui \
	forms/logindialog.ui

UI_DIR = ui

DEPENDPATH *= $${INCLUDEPATH}

RESOURCES += resources.qrc \
	deps/qdarkstyle/dark/darkstyle.qrc \
	deps/qdarkstyle/light/lightstyle.qrc

