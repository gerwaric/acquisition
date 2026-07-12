#include <QtTest/QtTest>

#include "testfixtures.h"

class BuyoutManagerTest : public QObject
{
    Q_OBJECT

private slots:
    void stringToBuyout();
    void itemBuyoutRoundTrip();
    void tabBuyoutRoundTrip();
    void tabBuyoutPropagation();
    void clearingItemBuyoutRemovesInMemoryValue();
    void clearingTabBuyoutRemovesInMemoryValue();
};

void BuyoutManagerTest::stringToBuyout()
{
    BuyoutManagerFixture fixture;

    const Buyout chaos = fixture.manager->StringToBuyout("~b/o 5 chaos");
    QCOMPARE(chaos.type, Buyout::BUYOUT_TYPE_BUYOUT);
    QCOMPARE(chaos.value, 5.0);
    QCOMPARE(chaos.currency, Currency::CURRENCY_CHAOS_ORB);
    QCOMPARE(chaos.source, Buyout::BUYOUT_SOURCE_GAME);
    QVERIFY(chaos.IsActive());

    const Buyout exalted = fixture.manager->StringToBuyout("~price 1.5 exalted");
    QCOMPARE(exalted.type, Buyout::BUYOUT_TYPE_FIXED);
    QCOMPARE(exalted.value, 1.5);
    QCOMPARE(exalted.currency, Currency::CURRENCY_EXALTED_ORB);

    const Buyout fusing = fixture.manager->StringToBuyout("~gb/o 20 fusing");
    QCOMPARE(fusing.type, Buyout::BUYOUT_TYPE_BUYOUT);
    QCOMPARE(fusing.value, 20.0);
    QCOMPARE(fusing.currency, Currency::CURRENCY_ORB_OF_FUSING);

    const Buyout currentOffer = fixture.manager->StringToBuyout("~c/o 3 chaos");
    QCOMPARE(currentOffer.type, Buyout::BUYOUT_TYPE_CURRENT_OFFER);
    QCOMPARE(currentOffer.value, 3.0);
    QCOMPARE(currentOffer.currency, Currency::CURRENCY_CHAOS_ORB);

    QVERIFY(fixture.manager->StringToBuyout("not a buyout").IsNull());

    const Buyout tolerant = fixture.manager->StringToBuyout("prefix ~b/o 7 chaos suffix");
    QCOMPARE(tolerant.type, Buyout::BUYOUT_TYPE_BUYOUT);
    QCOMPARE(tolerant.value, 7.0);
    QCOMPARE(tolerant.currency, Currency::CURRENCY_CHAOS_ORB);
}

void BuyoutManagerTest::itemBuyoutRoundTrip()
{
    BuyoutManagerFixture fixture;
    const Item item = makeTestItem("item-round-trip");
    const Buyout buyout = makeChaosBuyout(9.0);

    fixture.manager->Set(item, buyout);

    QCOMPARE(fixture.manager->Get(item), buyout);
}

void BuyoutManagerTest::tabBuyoutRoundTrip()
{
    BuyoutManagerFixture fixture;
    const ItemLocation location = makeTestStashLocation();
    const Buyout buyout = makeChaosBuyout(11.0);

    fixture.manager->SetTab(location, buyout);

    QCOMPARE(fixture.manager->GetTab(location), buyout);
}

static void propagateTabBuyout(BuyoutManager &manager, const Item &item)
{
    const auto itemBuyout = manager.Get(item);
    auto tabBuyout = manager.GetTab(item.location());

    if (itemBuyout.IsInherited()) {
        if (tabBuyout.IsActive()) {
            tabBuyout.inherited = true;
            tabBuyout.last_update = QDateTime::currentDateTime();
            manager.Set(item, tabBuyout);
        } else {
            manager.Set(item, Buyout());
        }
    }
}

void BuyoutManagerTest::tabBuyoutPropagation()
{
    BuyoutManagerFixture activeFixture;
    const Item activeItem = makeTestItem("propagated-item");
    activeFixture.manager->SetTab(activeItem.location(), makeChaosBuyout(12.0));

    propagateTabBuyout(*activeFixture.manager, activeItem);

    const Buyout propagated = activeFixture.manager->Get(activeItem);
    QVERIFY(propagated.IsActive());
    QVERIFY(propagated.IsInherited());
    QCOMPARE(propagated.value, 12.0);
    QCOMPARE(propagated.currency, Currency::CURRENCY_CHAOS_ORB);

    BuyoutManagerFixture inactiveFixture;
    const Item inactiveItem = makeTestItem("cleared-inherited-item");

    propagateTabBuyout(*inactiveFixture.manager, inactiveItem);

    QVERIFY(inactiveFixture.manager->Get(inactiveItem).IsNull());
}

void BuyoutManagerTest::clearingItemBuyoutRemovesInMemoryValue()
{
    BuyoutManagerFixture fixture;
    const Item item = makeTestItem("f14-item");

    fixture.manager->Set(item, makeChaosBuyout(14.0));
    QVERIFY(fixture.manager->Get(item).IsActive());

    fixture.manager->Set(item, Buyout());

    QEXPECT_FAIL("", "F14: stale in-memory buyout after clear", Continue);
    QVERIFY(fixture.manager->Get(item).IsNull());
}

void BuyoutManagerTest::clearingTabBuyoutRemovesInMemoryValue()
{
    BuyoutManagerFixture fixture;
    const ItemLocation location = makeTestStashLocation("f14-tab");

    fixture.manager->SetTab(location, makeChaosBuyout(14.0));
    QVERIFY(fixture.manager->GetTab(location).IsActive());

    fixture.manager->SetTab(location, Buyout());

    QEXPECT_FAIL("", "F14: stale in-memory buyout after clear", Continue);
    QVERIFY(fixture.manager->GetTab(location).IsNull());
}

QTEST_GUILESS_MAIN(BuyoutManagerTest)

#include "tst_buyoutmanager.moc"
