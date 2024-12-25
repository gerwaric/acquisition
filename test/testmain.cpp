/*
    Copyright (C) 2014-2024 Acquisition Contributors

    This file is part of Acquisition.

    Acquisition is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Acquisition is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Acquisition.  If not, see <http://www.gnu.org/licenses/>.
*/

#include "testmain.h"

#include <QEventLoop>
#include <QLocale>
#include <QObject>
#include <QSettings>
#include <QTemporaryFile>
#include <QTest>

#include <clocale>
#include <memory>

#include "application.h"
#include "datastore/datastore.h"
#include "itemsmanager.h"
#include "datastore/memorydatastore.h"
#include "network_info.h"
#include "oauthmanager.h"
#include "ratelimit/ratelimiter.h"
#include "repoe.h"
#include "shop.h"
#include "testitem.h"
#include "testitemsmanager.h"
#include "testshop.h"
#include "testutil.h"

#define TEST(Class, ...) QTest::qExec(std::make_unique<Class>(__VA_ARGS__).get())

int test_main() {

    QLocale::setDefault(QLocale::C);
    std::setlocale(LC_ALL, "C");

    QNetworkAccessManager network_manager;
    TestHelper helper;
    RePoE repoe(network_manager);

    QEventLoop loop;
    QObject::connect(&repoe, &RePoE::finished, &helper, [&]() { helper.run(network_manager, repoe); });
    QObject::connect(&helper, &TestHelper::finished, &loop, &QEventLoop::exit);
    repoe.Init();
    return loop.exec();
}

int TestHelper::run(QNetworkAccessManager& network_manager, RePoE& repoe) {

    // Create a temporary settings file.
    auto tmp = std::make_unique<QTemporaryFile>();
    tmp->open();

    std::unique_ptr<DataStore> datastore = std::make_unique<MemoryDataStore>();

    QSettings settings(tmp->fileName(), QSettings::IniFormat);
    OAuthManager oauth_manager(network_manager, *datastore);
    RateLimiter rate_limiter(network_manager, oauth_manager, POE_API::LEGACY);
    BuyoutManager buyout_manager(*datastore);
    ItemsManager items_manager(settings, network_manager, repoe, buyout_manager, *datastore, rate_limiter);
    Shop shop(settings, network_manager, *datastore, items_manager, buyout_manager);

    int result = 0;
    result |= TEST(TestItem);
    result |= TEST(TestShop, items_manager, buyout_manager, shop);
    result |= TEST(TestUtil);
    result |= TEST(TestItemsManager, *datastore, items_manager, buyout_manager);

    int status = (result != 0) ? -1 : 0;
    emit finished(status);
    return status;
}
