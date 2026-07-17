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
    void clearingItemBuyoutRemovesRepoRow();
    void clearingTabBuyoutRemovesRepoRow();
    void clearingAbsentBuyoutSkipsRepo();
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

    QVERIFY(fixture.manager->Get(item).IsNull());
}

void BuyoutManagerTest::clearingTabBuyoutRemovesInMemoryValue()
{
    BuyoutManagerFixture fixture;
    const ItemLocation location = makeTestStashLocation("f14-tab");

    fixture.manager->SetTab(location, makeChaosBuyout(14.0));
    QVERIFY(fixture.manager->GetTab(location).IsActive());

    fixture.manager->SetTab(location, Buyout());

    QVERIFY(fixture.manager->GetTab(location).IsNull());
}

// The F52 guard must not weaken the F14 behavior: clearing a buyout that
// exists still removes the database row, not just the in-memory entry.
void BuyoutManagerTest::clearingItemBuyoutRemovesRepoRow()
{
    BuyoutManagerFixture fixture;
    const Item item = makeTestItem("f52-item");

    fixture.manager->Set(item, makeChaosBuyout(52.0));
    QVERIFY(fixture.repo->getItemBuyouts().contains(item.id()));

    fixture.manager->Set(item, Buyout());

    QVERIFY(!fixture.repo->getItemBuyouts().contains(item.id()));
}

void BuyoutManagerTest::clearingTabBuyoutRemovesRepoRow()
{
    BuyoutManagerFixture fixture;
    const ItemLocation location = makeTestStashLocation("f52-tab");

    fixture.manager->SetTab(location, makeChaosBuyout(52.0));
    QVERIFY(fixture.repo->getLocationBuyouts().contains(location.id()));

    fixture.manager->SetTab(location, Buyout());

    QVERIFY(!fixture.repo->getLocationBuyouts().contains(location.id()));
}

// Clearing a buyout the manager does not hold must not touch the repo (F52:
// PropagateTabBuyouts clears nearly every item on every refresh, which used
// to issue one no-op DELETE per item). Pinned by seeding a row behind the
// manager's back: with the guard, the row survives the clear. This is the
// accepted tradeoff — a desynced row lasts until the next Load() picks it
// up — in exchange for not running ~item-count DELETEs per refresh.
void BuyoutManagerTest::clearingAbsentBuyoutSkipsRepo()
{
    BuyoutManagerFixture fixture;
    const Item item = makeTestItem("f52-desynced-item");

    fixture.repo->saveItemBuyout(makeChaosBuyout(52.0), item);
    QVERIFY(fixture.manager->Get(item).IsNull());

    fixture.manager->Set(item, Buyout());

    QVERIFY(fixture.repo->getItemBuyouts().contains(item.id()));
}

QTEST_GUILESS_MAIN(BuyoutManagerTest)

#include "tst_buyoutmanager.moc"
