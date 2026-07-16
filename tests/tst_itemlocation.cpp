#include <QtTest/QtTest>

#include "itemlocation.h"
#include "poe/types/character.h"
#include "poe/types/stashtab.h"

// Pins the fetch-source id contract (items-pipeline M1): the fetch id keys
// the per-reply atomic item replacement in ItemsManagerWorker, defaults to
// the location's own id, can be redirected to a child stash, and must never
// influence equality, ordering, or buyout hashing.
class ItemLocationTest : public QObject
{
    Q_OBJECT

private slots:
    void stashFetchIdDefaultsToOwnId();
    void characterFetchIdDefaultsToOwnId();
    void childFetchIdLeavesDisplayLocationAlone();
    void fetchIdDoesNotAffectIdentity();
};

static poe::StashTab makeStash(const QString &id, const QString &name, unsigned index = 0)
{
    poe::StashTab stash;
    stash.id = id;
    stash.name = name;
    stash.type = "PremiumStash";
    stash.index = index;
    return stash;
}

void ItemLocationTest::stashFetchIdDefaultsToOwnId()
{
    const ItemLocation location(makeStash("stash00001", "Test Tab"));
    QCOMPARE(location.fetch_id(), QString("stash00001"));
    QCOMPARE(location.fetch_id(), location.id());
}

void ItemLocationTest::characterFetchIdDefaultsToOwnId()
{
    poe::Character character;
    character.id = "abcdef0123456789";
    character.name = "TestCharacter";
    const ItemLocation location(character, 3);
    QCOMPARE(location.fetch_id(), QString("abcdef0123456789"));
    QCOMPARE(location.fetch_id(), location.id());
}

void ItemLocationTest::childFetchIdLeavesDisplayLocationAlone()
{
    const ItemLocation parent(makeStash("parent0001", "Map Stash"));
    ItemLocation child_fetch = parent;
    child_fetch.setFetchId("child00001");

    QCOMPARE(child_fetch.fetch_id(), QString("child00001"));
    // The display location is still the parent tab.
    QCOMPARE(child_fetch.id(), parent.id());
    QCOMPARE(child_fetch.tab_label(), parent.tab_label());
    QCOMPARE(child_fetch.GetHeader(), parent.GetHeader());
}

void ItemLocationTest::fetchIdDoesNotAffectIdentity()
{
    const ItemLocation a(makeStash("stash00001", "Test Tab", 4));
    ItemLocation b = a;
    b.setFetchId("child00001");

    // Equality, ordering, and the buyout hash key must ignore the fetch id;
    // buyouts and sort order would silently change otherwise.
    QVERIFY(a == b);
    QVERIFY(!(a < b));
    QVERIFY(!(b < a));
    QCOMPARE(a.GetLegacyHash(), b.GetLegacyHash());
}

QTEST_GUILESS_MAIN(ItemLocationTest)

#include "tst_itemlocation.moc"
