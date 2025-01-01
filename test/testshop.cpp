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

#include "testshop.h"

#include <QTest>

#include <memory>

#include "rapidjson/document.h"

#include "buyoutmanager.h"
#include "itemsmanager.h"
#include "shop.h"
#include "testdata.h"

TestShop::TestShop(ItemsManager& items_manager, BuyoutManager& buyout_manager, Shop& shop)
    :
    items_manager_(items_manager),
    buyout_manager_(buyout_manager),
    shop_(shop)
{}

void TestShop::SocketedGemsNotLinked() {

    rapidjson::Document doc;
    doc.Parse(kSocketedItem);

    Items items = { std::make_shared<Item>(doc, ItemLocation()) };
    items_manager_.OnItemsRefreshed(items, {}, true);

    Buyout bo;
    bo.type = BUYOUT_TYPE_FIXED;
    bo.value = 10;
    bo.currency = CURRENCY_CHAOS_ORB;
    buyout_manager_.Set(*items[0], bo);

    shop_.Update();
    QStringList shop = shop_.shop_data();
    QVERIFY(shop.size() == 0);

}

void TestShop::TemplatedShopGeneration() {

    rapidjson::Document doc;
    doc.Parse(kItem1);

    Items items = { std::make_shared<Item>(doc, ItemLocation()) };
    items_manager_.OnItemsRefreshed(items, {}, true);

    Buyout bo;
    bo.type = BUYOUT_TYPE_FIXED;
    bo.value = 10;
    bo.currency = CURRENCY_CHAOS_ORB;
    buyout_manager_.Set(*items[0], bo);

    shop_.SetShopTemplate("My awesome shop [items]");
    shop_.Update();

    QStringList shop = shop_.shop_data();
    QVERIFY(shop.size() == 1);
    QVERIFY(shop[0].contains("~price"));
    QVERIFY(shop[0].contains("My awesome shop"));

}
