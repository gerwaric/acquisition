#include <clocale>
#include <QLocale>
#include <QTest>
#include <memory>

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

	TEST(TestItem);
	TEST(TestShop);
	TEST(TestUtil);
	TEST(TestItemsManager);
	return (result != 0) ? -1 : 0;
}
