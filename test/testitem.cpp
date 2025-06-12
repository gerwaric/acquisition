/*
    Copyright (C) 2014-2025 Acquisition Contributors

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

#include "testitem.h"

#include <QtTest/QTest>

#include <util/spdlog_qt.h>
#include <rapidjson/document.h>
#include <rapidjson/error/en.h>

#include "testdata.h"

void TestItem::testBasicParsing() {

    const Item item = parseItem(kItem1);

    // no need to check everything, just some basic properties
	QCOMPARE(item.name(), "Chimeric Crest");
    QCOMPARE(item.typeLine(), "Vaal Mask");
    QCOMPARE(item.frameType(), 2);

    const auto& sockets = item.sockets();
    QCOMPARE(sockets.b, 2);
    QCOMPARE(sockets.g, 2);
    QCOMPARE(sockets.r, 0);
    QCOMPARE(sockets.w, 0);

    const auto& requirements = item.requirements();
    const auto it = requirements.find(QStringLiteral("Level"));
    if (it == requirements.end()) {
        QVERIFY(false);
    } else {
        const int required_level = it->second;
        QCOMPARE(required_level, 69);
    };
    
    // the hash should be the same between different versions of Acquisition and OSes
    QCOMPARE(item.hash(), "d7341b85cb8115efee9896dda9b3f60e");

    // This needs to match so that item hash migration is successful
    QCOMPARE(item.old_hash(), "fb915d79d2659e9175afae12612da584");
}

void TestItem::testDivCardCategory() {
    const Item item = parseItem(kCategoriesItemCard);
    const QString category = item.category();
    QCOMPARE(category, "divination cards");
}

void TestItem::testBeltCategory() {
    const Item item = parseItem(kCategoriesItemBelt);
    const QString category = item.category();
    QCOMPARE(category, "belts");
}

void TestItem::testEssenceCategory() {
    const Item item = parseItem(kCategoriesItemEssence);
    const QString category = item.category();
    QCOMPARE(category, "stackable currency");
}

void TestItem::testVaalGemCategory() {
    const Item item = parseItem(kCategoriesItemVaalGem);
    const QString category = item.category();
    QCOMPARE(category, "gems.vaal");
}

void TestItem::testSupportGemCategory() {
    const Item item = parseItem(kCategoriesItemSupportGem);
    const QString category = item.category();
    QCOMPARE(category, "gems.support");
}

void TestItem::testBowCategory() {
    const Item item = parseItem(kCategoriesItemBow);
    const QString category = item.category();
    QCOMPARE(category, "bows");
}

void TestItem::testClawCategory() {
    const Item item = parseItem(kCategoriesItemClaw);
    const QString category = item.category();
    QCOMPARE(category, "claws");
}

void TestItem::testFragmentCategory() {
    const Item item = parseItem(kCategoriesItemFragment);
    const QString category = item.category();
    QCOMPARE(category, "maps.atziri fragments");
}

void TestItem::testMapCategory() {
    const Item item = parseItem(kCategoriesItemWarMap);
    const QString category = item.category();
    QCOMPARE(category, "maps.3.1");
}

void TestItem::testUniqueMapCategory() {
    const Item item = parseItem(kCategoriesItemUniqueMap);
    const QString category = item.category();
    QCOMPARE(category, "maps.older uniques");
}

void TestItem::testBreachstoneCategory() {
    const Item item = parseItem(kCategoriesItemBreachstone);
    const QString category = item.category();
    QCOMPARE(category, "currency.breach");
}

void TestItem::testBeltPOB() {
    const Item item = parseItem(kCategoriesItemBelt);
    const std::string pob = item.POBformat().toStdString();
    QCOMPARE(pob.c_str(), kItemBeltPOB);
}

void TestItem::testBowPOB() {
    const Item item = parseItem(kCategoriesItemBow);
    const std::string pob = item.POBformat().toStdString();
    QCOMPARE(pob.c_str(), kItemBowPOB);
}

void TestItem::testClawPOB() {
    const Item item = parseItem(kCategoriesItemClaw);
    const std::string pob = item.POBformat().toStdString();
    QCOMPARE(pob.c_str(), kItemClawPOB);
}

Item TestItem::parseItem(const char* json) {
    rapidjson::Document doc;
    doc.Parse(json);
    if (doc.HasParseError()) {
        const auto code = doc.GetParseError();
        const auto error = rapidjson::GetParseError_En(code);
        spdlog::error("Error parsing test item: {}", error);
        spdlog::error("Item is: {}", json);
        return Item("", ItemLocation());
    } else {
        return Item(doc, ItemLocation());
    };
}

QString TestItem::getCategory(const char* json) {
    const Item item = parseItem(json);
    const QString category = item.category();
    return category;
}

QString TestItem::getPOB(const char* json) {
    const Item item = parseItem(json);
    const QString pob = item.POBformat();
    return pob;
}
