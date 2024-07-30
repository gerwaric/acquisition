#include "testmain.h"

#include <QEventLoop>
#include <QLocale>
#include <QObject>
#include <QSettings>
#include <QTest>

#include <clocale>
#include <memory>

#include "application.h"
#include "datastore.h"
#include "itemsmanager.h"
#include "memorydatastore.h"
#include "network_info.h"
#include "repoe.h"
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
    TestHelper helper;

    if (app.repoe().IsInitialized()) {
        return helper.run(app);
    } else {
        QEventLoop loop;
        QObject::connect(&app.repoe(), &RePoE::finished, &helper, [&]() { helper.run(app); });
        QObject::connect(&helper, &TestHelper::finished, &loop, &QEventLoop::exit);
        return loop.exec();
    };
}

int TestHelper::run(Application& app) {

    DataStore& data = app.data();
    ItemsManager& items_manager = app.items_manager();
    BuyoutManager& buyout_manager = app.buyout_manager();
    Shop& shop = app.shop();

    int result = 0;
    result |= TEST(TestItem);
    result |= TEST(TestShop, items_manager, buyout_manager, shop);
    result |= TEST(TestUtil);
    result |= TEST(TestItemsManager, data, items_manager, buyout_manager);

    int status = (result != 0) ? -1 : 0;
    emit finished(status);
    return status;
}
