#include <clocale>
#include <QLocale>
#include <QTest>
#include <memory>

#include "QsLog.h"

#include "porting.h"
#include "testitem.h"
#include "testitemsmanager.h"
#include "testshop.h"
#include "testutil.h"

#define TEST(Class) result |= QTest::qExec(std::make_unique<Class>().get())

int test_main() {
	int result = 0;

	QLocale::setDefault(QLocale::C);
	std::setlocale(LC_ALL, "C");

	QLOG_INFO() << "------------------------------------------------------------------------------";
	QLOG_INFO() << "TestItem starting";
	TEST(TestItem);
	QLOG_INFO() << "TestItem returned" << result;

	QLOG_INFO() << "------------------------------------------------------------------------------"; 
	QLOG_INFO() << "TestShop starting";
	TEST(TestShop);
	QLOG_INFO() << "TestShop returned" << result;

	QLOG_INFO() << "------------------------------------------------------------------------------";
	QLOG_INFO() << "TestUtil starting";
	TEST(TestUtil);
	QLOG_INFO() << "TestUtil returned" << result;

	QLOG_INFO() << "------------------------------------------------------------------------------";
	QLOG_INFO() << "TestUtil starting";
	TEST(TestItemsManager);
	QLOG_INFO() << "TestUtil returned" << result;

	result = (result != 0) ? -1 : 0;
	QLOG_INFO() << "------------------------------------------------------------------------------";
	QLOG_INFO() << "Final result is" << result;
	return result;
}
