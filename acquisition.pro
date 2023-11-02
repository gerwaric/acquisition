TARGET = acquisition
TEMPLATE = app

QT += core gui network testlib widgets httpserver websockets

win32 {
	QT.testlib.CONFIG -= console
}

macx {
	# Needed to use boost-headers-only 1.75 with Xcode 15.0.1
	DEFINES += _LIBCPP_ENABLE_CXX17_REMOVED_UNARY_BINARY_FUNCTION
}

unix:!macx {
        # This may need troubleshooting
	LIBS += -ldl -L"/usr/local/ssl/lib" -l:libcrypto.a -l:libssl.a
}

include(deps/QsLog/QsLog.pri)

INCLUDEPATH += src deps deps/boost-headers-only

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
	src/network_info.cpp \
	src/oauth.cpp \
	src/porting.cpp \
	src/ratelimit.cpp \
	src/ratelimitpanel.cpp \
	src/replytimeout.cpp \
	src/search.cpp \
	src/shop.cpp \
	src/updatechecker.cpp \
	src/util.cpp \
	src/version.cpp \
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
	src/version.h \
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

DEPENDPATH *= $${INCLUDEPATH}

RESOURCES += resources.qrc \
	deps/qdarkstyle/dark/darkstyle.qrc \
	deps/qdarkstyle/light/lightstyle.qrc

RC_FILE = resources.rc
