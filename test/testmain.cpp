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
#include "oauthmanager.h"
#include "ratelimiter.h"
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

    QNetworkAccessManager network_manager;
    TestHelper helper;
    RePoE repoe(nullptr, network_manager);

    QEventLoop loop;
    QObject::connect(&repoe, &RePoE::finished, &helper, [&]() { helper.run(network_manager, repoe); });
    QObject::connect(&helper, &TestHelper::finished, &loop, &QEventLoop::exit);
    repoe.Init();
    return loop.exec();
}

int TestHelper::run(QNetworkAccessManager& network_manager, RePoE& repoe) {

    std::unique_ptr<QSettings> settings = TestSettings::NewInstance();
    std::unique_ptr<DataStore> datastore = std::make_unique<MemoryDataStore>();

    OAuthManager oauth_manager(nullptr, network_manager, *datastore);
    RateLimiter rate_limiter(nullptr, network_manager, oauth_manager, POE_API::LEGACY);
    BuyoutManager buyout_manager(*datastore);
    ItemsManager items_manager(nullptr, *settings, network_manager, repoe, buyout_manager, *datastore, rate_limiter);
    Shop shop(nullptr, *settings, network_manager, *datastore, items_manager, buyout_manager);

    int result = 0;
    result |= TEST(TestItem);
    result |= TEST(TestShop, items_manager, buyout_manager, shop);
    result |= TEST(TestUtil);
    result |= TEST(TestItemsManager, *datastore, items_manager, buyout_manager);

    int status = (result != 0) ? -1 : 0;
    emit finished(status);
    return status;
}
