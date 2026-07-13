#include <QtTest/QtTest>

#include <memory>
#include <variant>

#include "filters/filterspec.h"
#include "search.h"
#include "testfixtures.h"

class SearchTest : public QObject
{
    Q_OBJECT

private slots:
    void bucketConstruction();
    void nameFilterMembership();
    void backgroundRefilterUsesOwnState();
    void backgroundBooleanRefilterUsesOwnState();
    void backgroundMinMaxRefilterUsesOwnState();
    void tabChangeSkipsRefilterWhenStateIsUnchanged();
    void tabChangeRefiltersAfterStateChange();
};

template<typename Payload>
static qsizetype findFilterIndex(const FilterCatalog &catalog, const QString &caption)
{
    for (qsizetype index = 0; index < catalog.size(); ++index) {
        const auto &spec = catalog[index];
        if ((spec.caption == caption) && std::holds_alternative<Payload>(spec.payload)) {
            return index;
        }
    }
    return -1;
}

static std::shared_ptr<Item> makeSearchItem(const QString &id,
                                            const QString &name,
                                            const QString &typeLine,
                                            const ItemLocation &location,
                                            const QString &extraJson = {})
{
    const QByteArray json = QString(R"json({
        "baseType": "%3",
        "frameType": 2,
        "frameTypeId": "Rare",
        "h": 1,
        "icon": "https://web.poecdn.com/image/test.png",
        "id": "%1",
        "identified": true,
        "ilvl": 1,
        "name": "%2",
        "typeLine": "%3",
        "verified": false,
        "w": 1,
        "x": 0,
        "y": 0%4
    })json")
                                .arg(id, name, typeLine, extraJson)
                                .toUtf8();
    return std::make_shared<Item>(makeTestItem(json.constData(), location));
}

void SearchTest::bucketConstruction()
{
    BuyoutManagerFixture buyoutFixture;
    const ItemLocation firstTab = makeTestStashLocation("stash-a", "Alpha Tab", 0);
    const ItemLocation secondTab = makeTestStashLocation("stash-b", "Beta Tab", 1);
    const ItemLocation emptyTab = makeTestStashLocation("stash-empty", "Empty Tab", 2);
    buyoutFixture.manager->SetStashTabLocations({firstTab, secondTab, emptyTab});

    Items items;
    items.push_back(makeSearchItem("alpha-item", "Alpha Bite", "Vaal Axe", firstTab));
    items.push_back(makeSearchItem("beta-item", "Beta Guard", "Copper Shield", secondTab));

    const FilterCatalog catalog = BuildFilterCatalog(*buyoutFixture.manager);
    Search search(*buyoutFixture.manager, "All", catalog);
    search.FilterItems(items);

    QCOMPARE(search.GetCaption(), "All [2]");
    QCOMPARE(search.buckets().size(), 3);
    QCOMPARE(search.buckets()[0].location().id(), firstTab.id());
    QCOMPARE(search.buckets()[0].items().size(), 1);
    QCOMPARE(search.buckets()[1].location().id(), secondTab.id());
    QCOMPARE(search.buckets()[1].items().size(), 1);
    QCOMPARE(search.buckets()[2].location().id(), emptyTab.id());
    QCOMPARE(search.buckets()[2].items().size(), 0);

    search.SetViewMode(Search::ViewMode::ByItem);
    QCOMPARE(search.buckets().size(), 1);
    QCOMPARE(search.buckets()[0].items().size(), 2);
}

void SearchTest::nameFilterMembership()
{
    BuyoutManagerFixture buyoutFixture;
    const ItemLocation firstTab = makeTestStashLocation("stash-a", "Alpha Tab", 0);
    const ItemLocation secondTab = makeTestStashLocation("stash-b", "Beta Tab", 1);
    const ItemLocation emptyTab = makeTestStashLocation("stash-empty", "Empty Tab", 2);
    buyoutFixture.manager->SetStashTabLocations({firstTab, secondTab, emptyTab});

    const FilterCatalog catalog = BuildFilterCatalog(*buyoutFixture.manager);
    const qsizetype nameIndex = findFilterIndex<TextPayload>(catalog, "Name");
    QVERIFY(nameIndex >= 0);
    Search search(*buyoutFixture.manager, "Filtered", catalog);
    search.setFilterState(nameIndex, TextState{"alpha"});

    Items items;
    items.push_back(makeSearchItem("alpha-item", "Alpha Bite", "Vaal Axe", firstTab));
    items.push_back(makeSearchItem("beta-item", "Beta Guard", "Copper Shield", secondTab));

    search.FilterItems(items);

    QCOMPARE(search.GetCaption(), "Filtered [1]");
    QCOMPARE(search.buckets().size(), 1);
    QCOMPARE(search.buckets()[0].location().id(), firstTab.id());
    QCOMPARE(search.buckets()[0].items().size(), 1);
    QCOMPARE(search.buckets()[0].items()[0]->id(), "alpha-item");

    search.SetViewMode(Search::ViewMode::ByItem);
    QCOMPARE(search.buckets().size(), 1);
    QCOMPARE(search.buckets()[0].items().size(), 1);
    QCOMPARE(search.buckets()[0].items()[0]->id(), "alpha-item");
}

void SearchTest::backgroundRefilterUsesOwnState()
{
    BuyoutManagerFixture buyoutFixture;
    const ItemLocation firstTab = makeTestStashLocation("stash-a", "Alpha Tab", 0);
    const ItemLocation secondTab = makeTestStashLocation("stash-b", "Beta Tab", 1);
    buyoutFixture.manager->SetStashTabLocations({firstTab, secondTab});

    Items items;
    items.push_back(makeSearchItem("alpha-item", "Alpha Bite", "Vaal Axe", firstTab));
    items.push_back(makeSearchItem("beta-item", "Beta Guard", "Copper Shield", secondTab));

    const FilterCatalog catalog = BuildFilterCatalog(*buyoutFixture.manager);
    const qsizetype nameIndex = findFilterIndex<TextPayload>(catalog, "Name");
    QVERIFY(nameIndex >= 0);
    Search background(*buyoutFixture.manager, "Background", catalog);
    background.setFilterState(nameIndex, TextState{"alpha"});
    // The current search leaves its name empty: under the old shared-activity
    // design that made the name filter inactive for every search, so the
    // background search's own query was skipped and it kept both items.
    Search current(*buyoutFixture.manager, "Current", catalog);
    background.FilterItems(items);

    // F33: a background search uses its own saved activity and query.
    QCOMPARE(background.GetCaption(), "Background [1]");
    QCOMPARE(background.items().size(), 1);
    QCOMPARE(background.items().front()->id(), "alpha-item");
}

void SearchTest::backgroundBooleanRefilterUsesOwnState()
{
    BuyoutManagerFixture buyoutFixture;
    const ItemLocation firstTab = makeTestStashLocation("stash-a", "Alpha Tab", 0);
    const ItemLocation secondTab = makeTestStashLocation("stash-b", "Beta Tab", 1);
    buyoutFixture.manager->SetStashTabLocations({firstTab, secondTab});

    Items items;
    items.push_back(makeSearchItem("corrupted-item",
                                   "Corrupted Bite",
                                   "Vaal Axe",
                                   firstTab,
                                   R"json(, "corrupted": true)json"));
    items.push_back(makeSearchItem("ordinary-item", "Ordinary Guard", "Copper Shield", secondTab));

    const FilterCatalog catalog = BuildFilterCatalog(*buyoutFixture.manager);
    const qsizetype corruptedIndex = findFilterIndex<BoolPayload>(catalog, "Corrupted");
    QVERIFY(corruptedIndex >= 0);
    Search background(*buyoutFixture.manager, "Background", catalog);
    background.setFilterState(corruptedIndex, BoolState{true});
    Search current(*buyoutFixture.manager, "Current", catalog);
    current.setFilterState(corruptedIndex, BoolState{false});
    background.FilterItems(items);

    QCOMPARE(background.GetCaption(), "Background [1]");
    QCOMPARE(background.items().size(), 1);
    QCOMPARE(background.items().front()->id(), "corrupted-item");
}

void SearchTest::backgroundMinMaxRefilterUsesOwnState()
{
    BuyoutManagerFixture buyoutFixture;
    const ItemLocation firstTab = makeTestStashLocation("stash-a", "Alpha Tab", 0);
    const ItemLocation secondTab = makeTestStashLocation("stash-b", "Beta Tab", 1);
    buyoutFixture.manager->SetStashTabLocations({firstTab, secondTab});

    Items items;
    items.push_back(makeSearchItem("critical-item",
                                   "Critical Bite",
                                   "Vaal Axe",
                                   firstTab,
                                   R"json(,
        "properties": [
            {
                "displayMode": 0,
                "name": "Critical Strike Chance",
                "type": 6,
                "values": [["6", 1]]
            }
        ])json"));
    items.push_back(makeSearchItem("ordinary-item", "Ordinary Guard", "Copper Shield", secondTab));

    const FilterCatalog catalog = BuildFilterCatalog(*buyoutFixture.manager);
    const qsizetype critIndex = findFilterIndex<MinMaxPayload>(catalog, "Crit.");
    QVERIFY(critIndex >= 0);
    Search background(*buyoutFixture.manager, "Background", catalog);
    background.setFilterState(critIndex, MinMaxState{5.0, std::nullopt});
    // As above: the current search leaves both bounds empty, which is what made
    // the old shared activity flag drop the background search's own bounds.
    Search current(*buyoutFixture.manager, "Current", catalog);
    background.FilterItems(items);

    QCOMPARE(background.GetCaption(), "Background [1]");
    QCOMPARE(background.items().size(), 1);
    QCOMPARE(background.items().front()->id(), "critical-item");
}

// The TabChanged short-circuit is an optimization: switching tabs alone cannot
// change what a search matches, so it keeps its buckets.
void SearchTest::tabChangeSkipsRefilterWhenStateIsUnchanged()
{
    BuyoutManagerFixture buyoutFixture;
    const ItemLocation firstTab = makeTestStashLocation("stash-a", "Alpha Tab", 0);
    const ItemLocation secondTab = makeTestStashLocation("stash-b", "Beta Tab", 1);
    buyoutFixture.manager->SetStashTabLocations({firstTab, secondTab});

    Items items;
    items.push_back(makeSearchItem("alpha-item", "Alpha Bite", "Vaal Axe", firstTab));
    items.push_back(makeSearchItem("beta-item", "Beta Guard", "Copper Shield", secondTab));

    const FilterCatalog catalog = BuildFilterCatalog(*buyoutFixture.manager);
    Search search(*buyoutFixture.manager, "Search", catalog);
    search.FilterItems(items);
    QCOMPARE(search.items().size(), 2);

    // Nothing changed, so a tab change must not re-run the filters: the new
    // item list is ignored entirely.
    Items moreItems = items;
    moreItems.push_back(makeSearchItem("gamma-item", "Gamma Blade", "Vaal Axe", firstTab));
    search.SetRefreshReason(RefreshReason::TabChanged);
    search.FilterItems(moreItems);
    QCOMPARE(search.items().size(), 2);
}

// ...but a filter state edited while this search was in the background (the
// mods form writes through to the bound search) must force the refilter that
// the debounced refresh gave to whichever search was current when it fired.
void SearchTest::tabChangeRefiltersAfterStateChange()
{
    BuyoutManagerFixture buyoutFixture;
    const ItemLocation firstTab = makeTestStashLocation("stash-a", "Alpha Tab", 0);
    const ItemLocation secondTab = makeTestStashLocation("stash-b", "Beta Tab", 1);
    buyoutFixture.manager->SetStashTabLocations({firstTab, secondTab});

    Items items;
    items.push_back(makeSearchItem("alpha-item", "Alpha Bite", "Vaal Axe", firstTab));
    items.push_back(makeSearchItem("beta-item", "Beta Guard", "Copper Shield", secondTab));

    const FilterCatalog catalog = BuildFilterCatalog(*buyoutFixture.manager);
    const qsizetype nameIndex = findFilterIndex<TextPayload>(catalog, "Name");
    QVERIFY(nameIndex >= 0);
    Search search(*buyoutFixture.manager, "Search", catalog);
    search.FilterItems(items);
    QCOMPARE(search.items().size(), 2);

    search.setFilterState(nameIndex, TextState{"alpha"});
    search.SetRefreshReason(RefreshReason::TabChanged);
    search.FilterItems(items);

    QCOMPARE(search.GetCaption(), "Search [1]");
    QCOMPARE(search.items().size(), 1);
    QCOMPARE(search.items().front()->id(), "alpha-item");

    // The refilter clears the dirty flag: the short-circuit is back in force.
    Items moreItems = items;
    moreItems.push_back(makeSearchItem("gamma-item", "Alpha Blade", "Vaal Axe", firstTab));
    search.SetRefreshReason(RefreshReason::TabChanged);
    search.FilterItems(moreItems);
    QCOMPARE(search.items().size(), 1);
}

QTEST_MAIN(SearchTest)

#include "tst_search.moc"
