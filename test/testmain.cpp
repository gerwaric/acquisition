#include "testmain.h"

#include <QLocale>
#include <QSettings>
#include <QTest>

#include <clocale>
#include <memory>

#include "application.h"
#include "datastore.h"
#include "itemsmanager.h"
#include "memorydatastore.h"
#include "network_info.h"
#include "shop.h"
#include "testitem.h"
#include "testitemsmanager.h"
#include "testsettings.h"
#include "testshop.h"
#include "testutil.h"

#define TEST(Class, ...) QTest::qExec(std::make_unique<Class>(__VA_ARGS__).get())

int test_main() {

    QLocale::setDefault(QLocale::C);
    std::setlocale(LC_ALL, "C");

    Application app(true);

    DataStore& data = app.data();
    ItemsManager& items_manager = app.items_manager();
    BuyoutManager& buyout_manager = app.buyout_manager();
    Shop& shop = app.shop();

    int result = 0;
    result |= TEST(TestItem);
    result |= TEST(TestShop, items_manager, buyout_manager, shop);
    result |= TEST(TestUtil);
    result |= TEST(TestItemsManager, data, items_manager, buyout_manager);

    return (result != 0) ? -1 : 0;
}
