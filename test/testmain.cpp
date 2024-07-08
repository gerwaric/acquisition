#include "testmain.h"

#include <QLocale>
#include <QSettings>
#include <QTest>

#include <clocale>
#include <memory>

#include "testitem.h"
#include "testitemsmanager.h"
#include "testshop.h"
#include "testutil.h"

#define TEST(Class) QTest::qExec(std::make_unique<Class>().get())

int test_main() {

    QLocale::setDefault(QLocale::C);
    std::setlocale(LC_ALL, "C");

    int result = 0;
    result |= TEST(TestItem);
    result |= TEST(TestShop);
    result |= TEST(TestUtil);
    result |= TEST(TestItemsManager);

    return (result != 0) ? -1 : 0;
}
