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
#include <QtDebug>
#include <QtLogging>

#include <clocale>
#include <memory>
#include <vector>

#include <QsLog/QsLog.h>

#include "application.h"
#include "datastore/datastore.h"
#include "itemsmanager.h"
#include "datastore/memorydatastore.h"
#include "network_info.h"
#include "util/oauthmanager.h"
#include "ratelimit/ratelimiter.h"
#include "util/repoe.h"
#include "shop.h"
#include "testitem.h"
#include "testitemsmanager.h"
#include "testshop.h"
#include "testutil.h"

int test_main(const QString& data_dir) {

    QNetworkAccessManager network_manager;
    TestHelper helper;
    RePoE repoe(network_manager);

    QEventLoop loop;
    QObject::connect(&repoe, &RePoE::finished, &helper, [&]() { helper.run(network_manager, repoe); });
    QObject::connect(&helper, &TestHelper::finished, &loop, &QEventLoop::exit);
    repoe.Init(data_dir);
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
    Shop shop(settings, network_manager, rate_limiter, *datastore, items_manager, buyout_manager);

    const QString verbosity = "-v2";

	int overall_result = 0;
    {
		TestItem item_test;
		const int result = QTest::qExec(&item_test, { verbosity, "-o", "acquisition-test-items.log" });
		QLOG_INFO() << "TestItem result is" << result;
		overall_result |= result;
	};
    {
		TestShop shop_test(items_manager, buyout_manager, shop);
		const int result = QTest::qExec(&shop_test, { verbosity, "-o", "acquisition-test-shop.log" });
		QLOG_INFO() << "TestShop result is" << result;
		overall_result |= result;
	};
    {
		TestUtil util_test;
		const int result = QTest::qExec(&util_test, { verbosity, "-o", "acquisition-test-utils.log" });
		QLOG_INFO() << "TestUtil result is" << result;
		overall_result |= result;
	};
    {
		TestItemsManager items_manager_test(*datastore, items_manager, buyout_manager);
		const int result = QTest::qExec(&items_manager_test, { verbosity, "-o", "acquisition-test-item-manager.log" });
		QLOG_INFO() << "TestItemsManager result is" << result;
		overall_result |= result;
	};
	int status = (overall_result == 0) ? 0 : -1;
    emit finished(status);
    return status;
}
