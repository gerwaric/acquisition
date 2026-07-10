#include <QtTest/QtTest>

#include <QLineEdit>
#include <QStringListModel>
#include <QTreeView>
#include <QVBoxLayout>
#include <QWidget>

#include "filters.h"
#include "itemcategories.h"
#include "modsfilter.h"
#include "search.h"
#include "testfixtures.h"
#include "ui/flowlayout.h"

class SearchTest : public QObject
{
    Q_OBJECT

private slots:
    void bucketConstruction();
    void nameFilterMembership();
};

struct SearchHarness
{
    QWidget host;
    QVBoxLayout layout{&host};
    QStringListModel categoryModel{GetItemCategories()};
    QStringListModel rarityModel{RaritySearchFilter::RARITY_LIST};
    std::vector<std::unique_ptr<Filter>> filters;
    QTreeView view;
    QObject receiver;
    FilterCallbacks callbacks{
        &receiver,
        [] {},
        [] {},
    };

    SearchHarness()
    {
        auto tab_search = std::make_unique<TabSearchFilter>(&layout, callbacks);
        auto name_search = std::make_unique<NameSearchFilter>(&layout, callbacks);
        auto category_search = std::make_unique<CategorySearchFilter>(&layout,
                                                                      &categoryModel,
                                                                      callbacks);
        auto rarity_search = std::make_unique<RaritySearchFilter>(&layout, &rarityModel, callbacks);
        auto *offense_layout = new FlowLayout;
        auto *defense_layout = new FlowLayout;
        auto *sockets_layout = new FlowLayout;
        auto *requirements_layout = new FlowLayout;
        auto *misc_layout = new FlowLayout;
        auto *misc_flags_layout = new FlowLayout;
        auto *misc_flags2_layout = new FlowLayout;
        auto *mods_layout = new QVBoxLayout;

        addSearchGroup(offense_layout);
        addSearchGroup(defense_layout);
        addSearchGroup(sockets_layout);
        addSearchGroup(requirements_layout);
        addSearchGroup(misc_layout);
        addSearchGroup(misc_flags_layout);
        addSearchGroup(misc_flags2_layout);
        addSearchGroup(mods_layout);

        using move_only = std::unique_ptr<Filter>;
        // clang-format off
        move_only init[] = {
            std::move(tab_search),
            std::move(name_search),
            std::move(category_search),
            std::move(rarity_search),
            std::make_unique<SimplePropertyFilter>(offense_layout, "Critical Strike Chance", "Crit.", callbacks),
            std::make_unique<ItemMethodFilter>(offense_layout, [](Item *item) { return item->DPS(); }, "DPS", callbacks),
            std::make_unique<ItemMethodFilter>(offense_layout, [](Item *item) { return item->pDPS(); }, "pDPS", callbacks),
            std::make_unique<ItemMethodFilter>(offense_layout, [](Item *item) { return item->eDPS(); }, "eDPS", callbacks),
            std::make_unique<ItemMethodFilter>(offense_layout, [](Item *item) { return item->cDPS(); }, "cDPS", callbacks),
            std::make_unique<SimplePropertyFilter>(offense_layout, "Attacks per Second", "APS", callbacks),
            std::make_unique<SimplePropertyFilter>(defense_layout, "Armour", callbacks),
            std::make_unique<SimplePropertyFilter>(defense_layout, "Evasion Rating", "Evasion", callbacks),
            std::make_unique<SimplePropertyFilter>(defense_layout, "Energy Shield", "Shield", callbacks),
            std::make_unique<SimplePropertyFilter>(defense_layout, "Chance to Block", "Block", callbacks),
            std::make_unique<SocketsFilter>(sockets_layout, "Sockets", callbacks),
            std::make_unique<LinksFilter>(sockets_layout, "Links", callbacks),
            std::make_unique<SocketsColorsFilter>(sockets_layout, callbacks),
            std::make_unique<LinksColorsFilter>(sockets_layout, callbacks),
            std::make_unique<RequiredStatFilter>(requirements_layout, "Level", "R. Level", callbacks),
            std::make_unique<RequiredStatFilter>(requirements_layout, "Str", "R. Str", callbacks),
            std::make_unique<RequiredStatFilter>(requirements_layout, "Dex", "R. Dex", callbacks),
            std::make_unique<RequiredStatFilter>(requirements_layout, "Int", "R. Int", callbacks),
            std::make_unique<DefaultPropertyFilter>(misc_layout, "Quality", 0, callbacks),
            std::make_unique<SimplePropertyFilter>(misc_layout, "Level", callbacks),
            std::make_unique<SimplePropertyFilter>(misc_layout, "Map Tier", callbacks),
            std::make_unique<ItemlevelFilter>(misc_layout, "ilvl", callbacks),
            std::make_unique<AltartFilter>(misc_flags_layout, "", "Alt. art", callbacks),
            std::make_unique<PricedFilter>(misc_flags_layout, "", "Priced", callbacks, *buyoutFixture.manager),
            std::make_unique<UnidentifiedFilter>(misc_flags2_layout, "", "Unidentified", callbacks),
            std::make_unique<InfluencedFilter>(misc_flags2_layout, "", "Influenced", callbacks),
            std::make_unique<CraftedFilter>(misc_flags2_layout, "", "Crafted", callbacks),
            std::make_unique<EnchantedFilter>(misc_flags2_layout, "", "Enchanted", callbacks),
            std::make_unique<CorruptedFilter>(misc_flags2_layout, "", "Corrupted", callbacks),
            std::make_unique<FracturedFilter>(misc_flags2_layout, "", "Fractured", callbacks),
            std::make_unique<SplitFilter>(misc_flags2_layout, "", "Split", callbacks),
            std::make_unique<SynthesizedFilter>(misc_flags2_layout, "", "Synthesized", callbacks),
            std::make_unique<MutatedFilter>(misc_flags2_layout, "", "Mutated", callbacks),
            std::make_unique<ModsFilter>(mods_layout, callbacks),
        };
        // clang-format on
        filters = std::vector<move_only>(std::make_move_iterator(std::begin(init)),
                                         std::make_move_iterator(std::end(init)));
    }

    void setNameFilter(const QString &text)
    {
        const auto edits = host.findChildren<QLineEdit *>();
        QVERIFY(edits.size() >= 2);
        edits[1]->setText(text);
    }

    BuyoutManagerFixture buyoutFixture;

private:
    void addSearchGroup(QLayout *child)
    {
        child->setContentsMargins(0, 0, 0, 0);
        auto *layout_container = new QWidget;
        layout_container->setLayout(child);
        layout.addWidget(layout_container);
    }
};

static std::shared_ptr<Item> makeSearchItem(const QString &id,
                                            const QString &name,
                                            const QString &typeLine,
                                            const ItemLocation &location)
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
        "y": 0
    })json")
                                .arg(id, name, typeLine)
                                .toUtf8();
    return std::make_shared<Item>(makeTestItem(json.constData(), location));
}

void SearchTest::bucketConstruction()
{
    SearchHarness harness;
    const ItemLocation firstTab = makeTestStashLocation("stash-a", "Alpha Tab", 0);
    const ItemLocation secondTab = makeTestStashLocation("stash-b", "Beta Tab", 1);
    const ItemLocation emptyTab = makeTestStashLocation("stash-empty", "Empty Tab", 2);
    harness.buyoutFixture.manager->SetStashTabLocations({firstTab, secondTab, emptyTab});

    Items items;
    items.push_back(makeSearchItem("alpha-item", "Alpha Bite", "Vaal Axe", firstTab));
    items.push_back(makeSearchItem("beta-item", "Beta Guard", "Copper Shield", secondTab));

    Search search(*harness.buyoutFixture.manager, "All", harness.filters);
    search.FromForm();
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
    SearchHarness harness;
    const ItemLocation firstTab = makeTestStashLocation("stash-a", "Alpha Tab", 0);
    const ItemLocation secondTab = makeTestStashLocation("stash-b", "Beta Tab", 1);
    const ItemLocation emptyTab = makeTestStashLocation("stash-empty", "Empty Tab", 2);
    harness.buyoutFixture.manager->SetStashTabLocations({firstTab, secondTab, emptyTab});
    harness.setNameFilter("alpha");

    Items items;
    items.push_back(makeSearchItem("alpha-item", "Alpha Bite", "Vaal Axe", firstTab));
    items.push_back(makeSearchItem("beta-item", "Beta Guard", "Copper Shield", secondTab));

    Search search(*harness.buyoutFixture.manager, "Filtered", harness.filters);
    search.FromForm();
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

QTEST_MAIN(SearchTest)

#include "tst_search.moc"
