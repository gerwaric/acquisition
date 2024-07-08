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
    std::vector<std::string> shop = shop_.shop_data();
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

    std::vector<std::string> shop = shop_.shop_data();
    QVERIFY(shop.size() == 1);
    QVERIFY(shop[0].find("~price") != std::string::npos);
    QVERIFY(shop[0].find("My awesome shop") != std::string::npos);

}
