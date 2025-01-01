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

#include "testitem.h"

#include <QtTest>

#include "rapidjson/document.h"

#include "item.h"
#include "testdata.h"

void TestItem::Parse() {

    rapidjson::Document doc;
    doc.Parse(kItem1);

    Item item(doc, ItemLocation());

    // no need to check everything, just some basic properties
    QCOMPARE(item.name(), "Demon Ward");
    QCOMPARE(item.sockets().b, 0);
    QCOMPARE(item.sockets().g, 1);
    QCOMPARE(item.sockets().r, 0);
    QCOMPARE(item.sockets().w, 0);

    // the hash should be the same between different versions of Acquisition and OSes
    QCOMPARE(item.hash(), "605d9f566bc4305f4fd425efbbbed6a6");
    // This needs to match so that item hash migration is successful
    QCOMPARE(item.old_hash(), "36f0097563123e5296dc2eed54e9d6f3");

}

void TestItem::ParseCategories() {

    rapidjson::Document doc;
    doc.Parse(kCategoriesItemCard);
    Item item(doc, ItemLocation());
    QCOMPARE(item.category(), "divination cards");

    doc.Parse(kCategoriesItemBelt);
    item = Item(doc, ItemLocation());
    QCOMPARE(item.category(), "belts");

    doc.Parse(kCategoriesItemEssence);
    item = Item(doc, ItemLocation());
    QCOMPARE(item.category(), "currency.essence");

    doc.Parse(kCategoriesItemVaalGem);
    item = Item(doc, ItemLocation());
    QCOMPARE(item.category(), "gems.vaal");

    doc.Parse(kCategoriesItemSupportGem);
    item = Item(doc, ItemLocation());
    QCOMPARE(item.category(), "gems.support");

    doc.Parse(kCategoriesItemBow);
    item = Item(doc, ItemLocation());
    QCOMPARE(item.category(), "weapons.2hand.bows");

    doc.Parse(kCategoriesItemClaw);
    item = Item(doc, ItemLocation());
    QCOMPARE(item.category(), "weapons.1hand.claws");

    doc.Parse(kCategoriesItemFragment);
    item = Item(doc, ItemLocation());
    QCOMPARE(item.category(), "maps.atziri fragments");

    doc.Parse(kCategoriesItemWarMap);
    item = Item(doc, ItemLocation());
    QCOMPARE(item.category(), "maps.3.1");

    doc.Parse(kCategoriesItemUniqueMap);
    item = Item(doc, ItemLocation());
    QCOMPARE(item.category(), "maps.older uniques");

    doc.Parse(kCategoriesItemBreachstone);
    item = Item(doc, ItemLocation());
    QCOMPARE(item.category(), "currency.breach");

}

void TestItem::POBformat() {

    rapidjson::Document doc;
    doc.Parse(kCategoriesItemBelt);
    Item item(doc, ItemLocation());
    QCOMPARE(item.POBformat(), kItemBeltPOB);

    doc.Parse(kCategoriesItemBow);
    item = Item(doc, ItemLocation());
    QCOMPARE(item.POBformat(), kItemBowPOB);

    doc.Parse(kCategoriesItemClaw);
    item = Item(doc, ItemLocation());
    QCOMPARE(item.POBformat(), kItemClawPOB);

}
