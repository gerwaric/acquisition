#include <QtTest/QtTest>

#include <QComboBox>
#include <QLineEdit>
#include <QPushButton>
#include <QStringListModel>
#include <QVBoxLayout>
#include <QWidget>

#include "filters.h"
#include "filters/filtermatchers.h"
#include "filters/filterspec.h"
#include "itemcategories.h"
#include "modlist.h"
#include "modsfilter.h"
#include "testfixtures.h"

class FiltersTest : public QObject
{
    Q_OBJECT

private slots:
    void tabFilter();
    void nameFilter();
    void categoryFilter();
    void categoryAnySentinel();
    void minMaxFilter();
    void minMaxGarbageTextIsActiveZero();
    void defaultAndRequiredFilters();
    void socketColorFilters();
    void colorsGarbageTextIsActiveZero();
    void booleanFilter();
    void booleanPredicates();
    void rarityFilter();
    void modsFilter();
    void colorsToFormKeepsStaleText();
    void modsFormSyncQuirks();
};

struct FilterHarness
{
    QWidget host;
    QVBoxLayout layout{&host};
    QStringListModel rarityModel{RaritySearchFilter::RARITY_LIST};
    QObject receiver;
    FilterCallbacks callbacks{
        &receiver,
        [] {},
        [] {},
    };
};

static std::shared_ptr<Item> makeFilterItem(
    const QString &id,
    const QString &extraJson,
    int frameType = 2,
    const QString &frameTypeId = "Rare",
    const QString &icon = "https://web.poecdn.com/image/test.png",
    const QString &baseType = "Test Item",
    bool identified = true)
{
    const QByteArray json = QString(R"json({
        "baseType": "%6",
        "frameType": %3,
        "frameTypeId": "%4",
        "h": 1,
        "icon": "%5",
        "id": "%1",
        "identified": %7,
        "ilvl": 1,
        "name": "Alpha Bite",
        "typeLine": "Test Item",
        "verified": false,
        "w": 1,
        "x": 0,
        "y": 0%2
    })json")
                                .arg(id,
                                     extraJson,
                                     QString::number(frameType),
                                     frameTypeId,
                                     icon,
                                     baseType,
                                     identified ? "true" : "false")
                                .toUtf8();
    return std::make_shared<Item>(makeTestItem(json.constData(), makeTestStashLocation()));
}

static const FilterSpec *findFilterSpec(const FilterCatalog &catalog, const QString &caption)
{
    for (const auto &spec : catalog) {
        if (spec.caption == caption) {
            return &spec;
        }
    }
    return nullptr;
}

static void verifyBooleanPredicate(const FilterSpec &spec,
                                   const std::shared_ptr<Item> &matching,
                                   const std::shared_ptr<Item> &notMatching)
{
    BoolState state{true};
    QVERIFY(MatchesFilter(*matching, spec, state));
    QVERIFY(!MatchesFilter(*notMatching, spec, state));

    state.checked = false;
    QVERIFY(MatchesFilter(*notMatching, spec, state));
}

void FiltersTest::tabFilter()
{
    FilterHarness harness;
    TabSearchFilter filter(&harness.layout, harness.callbacks);
    FilterData data(&filter);
    data.text_query = "test";

    QVERIFY(filter.Matches(makeFilterItem("tab-match", ""), &data));
    data.text_query = "missing";
    QVERIFY(!filter.Matches(makeFilterItem("tab-miss", ""), &data));
}

void FiltersTest::nameFilter()
{
    FilterHarness harness;
    NameSearchFilter filter(&harness.layout, harness.callbacks);
    FilterData data(&filter);
    data.text_query = "alpha";

    QVERIFY(filter.Matches(makeFilterItem("name-match", ""), &data));

    data.text_query = "ALPHA";
    QVERIFY(filter.Matches(makeFilterItem("name-match-case", ""), &data));

    data.text_query = "missing";
    QVERIFY(!filter.Matches(makeFilterItem("name-miss", ""), &data));
}

void FiltersTest::categoryFilter()
{
    InitItemClasses(R"json({"TestClass":{"name":"Weapons"}})json");
    InitItemBaseTypes(
        R"json({"Metadata/Items/TestSword":{"item_class":"TestClass","name":"Test Sword","release_state":"released"}})json");

    FilterHarness harness;
    QStringListModel model;
    CategorySearchFilter filter(&harness.layout, &model, harness.callbacks);
    FilterData data(&filter);
    data.text_query = "weapons";

    const auto weapon = makeFilterItem("category-match", "", 2, "Rare", {}, "Test Sword");
    const auto unknown = makeFilterItem("category-miss", "");
    QVERIFY(filter.Matches(weapon, &data));
    QVERIFY(!filter.Matches(unknown, &data));
}

void FiltersTest::categoryAnySentinel()
{
    FilterHarness harness;
    QStringListModel model{{CategorySearchFilter::k_Default, "Weapons"}};
    CategorySearchFilter filter(&harness.layout, &model, harness.callbacks);
    FilterData data(&filter);

    auto *combo = harness.host.findChild<QComboBox *>();
    QVERIFY(combo);
    combo->setCurrentText(CategorySearchFilter::k_Default);
    data.FromForm();

    QVERIFY(data.text_query.isEmpty());
    QVERIFY(!filter.IsActive());
}

void FiltersTest::minMaxFilter()
{
    FilterHarness harness;
    SimplePropertyFilter filter(&harness.layout, "Quality", harness.callbacks);
    const auto withQuality = makeFilterItem("quality",
                                            R"json(,
        "properties": [
            {
                "displayMode": 0,
                "name": "Quality",
                "type": 6,
                "values": [["+20%", 1]]
            }
        ])json");
    const auto withoutQuality = makeFilterItem("missing-quality", "");

    FilterData data(&filter);
    data.min_filled = true;
    data.min = 10.0;
    QVERIFY(filter.Matches(withQuality, &data));

    data.min = 25.0;
    QVERIFY(!filter.Matches(withQuality, &data));

    data.min_filled = false;
    data.max_filled = true;
    data.max = 20.0;
    QVERIFY(filter.Matches(withQuality, &data));

    data.max = 19.0;
    QVERIFY(!filter.Matches(withQuality, &data));

    data.min_filled = true;
    data.min = 1.0;
    data.max_filled = false;
    QVERIFY(!filter.Matches(withoutQuality, &data));

    data.min_filled = false;
    QVERIFY(filter.Matches(withoutQuality, &data));
}

void FiltersTest::minMaxGarbageTextIsActiveZero()
{
    FilterHarness harness;
    SimplePropertyFilter filter(&harness.layout, "Quality", harness.callbacks);
    FilterData data(&filter);
    const auto edits = harness.host.findChildren<QLineEdit *>();
    QCOMPARE(edits.size(), 2);

    edits[0]->setText("garbage");
    data.FromForm();

    QVERIFY(data.min_filled);
    QCOMPARE(data.min, 0.0);
    QVERIFY(!data.max_filled);
    QVERIFY(filter.IsActive());
}

void FiltersTest::defaultAndRequiredFilters()
{
    FilterHarness harness;
    const auto withoutQuality = makeFilterItem("default-quality", "");
    DefaultPropertyFilter quality(&harness.layout, "Quality", 0.0, harness.callbacks);
    FilterData qualityData(&quality);
    qualityData.min_filled = true;
    qualityData.min = 0.0;
    QVERIFY(quality.Matches(withoutQuality, &qualityData));
    qualityData.min = 1.0;
    QVERIFY(!quality.Matches(withoutQuality, &qualityData));

    const auto withRequirement = makeFilterItem("requirement",
                                                R"json(,
        "requirements": [
            {"displayMode": 0, "name": "Level", "values": [["12", 0]]}
        ])json");
    const auto withoutRequirement = makeFilterItem("no-requirement", "");
    RequiredStatFilter required(&harness.layout, "Level", harness.callbacks);
    FilterData requiredData(&required);
    requiredData.min_filled = true;
    requiredData.min = 12.0;
    QVERIFY(required.Matches(withRequirement, &requiredData));
    QVERIFY(!required.Matches(withoutRequirement, &requiredData));
    requiredData.min_filled = false;
    requiredData.max_filled = true;
    requiredData.max = 0.0;
    QVERIFY(required.Matches(withoutRequirement, &requiredData));
}

void FiltersTest::socketColorFilters()
{
    FilterHarness harness;
    const auto item = makeFilterItem("sockets",
                                     R"json(,
        "sockets": [
            {"group": 0, "attr": "S", "sColour": "R"},
            {"group": 0, "attr": "G", "sColour": "W"},
            {"group": 1, "attr": "I", "sColour": "B"}
        ])json");

    SocketsColorsFilter socketsFilter(&harness.layout, harness.callbacks);
    FilterData socketsData(&socketsFilter);
    socketsData.r_filled = true;
    socketsData.r = 1;
    socketsData.b_filled = true;
    socketsData.b = 1;
    QVERIFY(socketsFilter.Matches(item, &socketsData));

    socketsData.b = 3;
    QVERIFY(!socketsFilter.Matches(item, &socketsData));

    LinksColorsFilter linksFilter(&harness.layout, harness.callbacks);
    FilterData linksData(&linksFilter);
    linksData.r_filled = true;
    linksData.r = 1;
    linksData.b_filled = true;
    linksData.b = 1;
    QVERIFY(linksFilter.Matches(item, &linksData));

    linksData.b = 2;
    QVERIFY(!linksFilter.Matches(item, &linksData));
}

void FiltersTest::colorsGarbageTextIsActiveZero()
{
    FilterHarness harness;
    SocketsColorsFilter filter(&harness.layout, harness.callbacks);
    FilterData data(&filter);
    const auto edits = harness.host.findChildren<QLineEdit *>();
    QCOMPARE(edits.size(), 3);

    edits[0]->setText("garbage");
    data.FromForm();

    QVERIFY(data.r_filled);
    QCOMPARE(data.r, 0);
    QVERIFY(filter.IsActive());
}

void FiltersTest::booleanFilter()
{
    BuyoutManagerFixture buyoutFixture;
    const FilterCatalog catalog = BuildFilterCatalog(*buyoutFixture.manager);
    const auto *corrupted = findFilterSpec(catalog, "Corrupted");
    QVERIFY(corrupted);
    QVERIFY(std::holds_alternative<BoolPayload>(corrupted->payload));

    const FilterState defaultState = MakeDefaultState(*corrupted);
    QVERIFY(std::holds_alternative<BoolState>(defaultState));
    QVERIFY(!IsActive(defaultState));

    BoolState state{true};

    QVERIFY(MatchesFilter(*makeFilterItem("corrupted", R"json(, "corrupted": true)json"),
                          *corrupted,
                          state));
    QVERIFY(!MatchesFilter(*makeFilterItem("not-corrupted", ""), *corrupted, state));

    state.checked = false;
    QVERIFY(MatchesFilter(*makeFilterItem("unchecked", ""), *corrupted, state));
}

void FiltersTest::booleanPredicates()
{
    BuyoutManagerFixture buyoutFixture;
    const FilterCatalog catalog = BuildFilterCatalog(*buyoutFixture.manager);
    const auto ordinary = makeFilterItem("ordinary", "");

    for (const QString &caption : {"Alt. art",
                                   "Priced",
                                   "Unidentified",
                                   "Influenced",
                                   "Crafted",
                                   "Enchanted",
                                   "Corrupted",
                                   "Fractured",
                                   "Split",
                                   "Synthesized",
                                   "Mutated"}) {
        const auto *spec = findFilterSpec(catalog, caption);
        QVERIFY(spec);
        QVERIFY(std::holds_alternative<BoolPayload>(spec->payload));
        QCOMPARE(spec->refreshMode, RefreshMode::Immediate);
        const FilterState defaultState = MakeDefaultState(*spec);
        QVERIFY(std::holds_alternative<BoolState>(defaultState));
        QVERIFY(!IsActive(defaultState));
    }

    const auto *altart = findFilterSpec(catalog, "Alt. art");
    QVERIFY(altart);
    QVERIFY(std::holds_alternative<BoolPayload>(altart->payload));
    for (const QString &needle : {"RedBeak2.png", "dGlsbGF0ZUFsdCI7czoy", "WinterHeart.png"}) {
        const auto item = makeFilterItem("alt-" + needle,
                                         "",
                                         2,
                                         "Rare",
                                         "https://web.poecdn.com/image/" + needle);
        QVERIFY(MatchesFilter(*item, *altart, BoolState{true}));
    }
    verifyBooleanPredicate(*altart,
                           makeFilterItem("alt-match",
                                          "",
                                          2,
                                          "Rare",
                                          "https://web.poecdn.com/image/RedBeak2.png"),
                           ordinary);

    const auto pricedItem = makeFilterItem("priced", "");
    buyoutFixture.manager->Set(*pricedItem, makeChaosBuyout(5.0));
    const auto *priced = findFilterSpec(catalog, "Priced");
    QVERIFY(priced);
    QVERIFY(std::holds_alternative<BoolPayload>(priced->payload));
    verifyBooleanPredicate(*priced, pricedItem, ordinary);

    const auto flags = makeFilterItem("flags",
                                      R"json(,
        "influences": {"shaper": true},
        "craftedMods": ["Crafted modifier"],
        "enchantMods": ["Enchanted modifier"],
        "fractured": true,
        "split": true,
        "synthesised": true,
        "mutated": true
    )json",
                                      2,
                                      "Rare",
                                      "https://web.poecdn.com/image/test.png",
                                      "Test Item",
                                      false);
    for (const QString &caption : {"Unidentified",
                                   "Influenced",
                                   "Crafted",
                                   "Enchanted",
                                   "Fractured",
                                   "Split",
                                   "Synthesized",
                                   "Mutated"}) {
        const auto *spec = findFilterSpec(catalog, caption);
        QVERIFY(spec);
        QVERIFY(std::holds_alternative<BoolPayload>(spec->payload));
        verifyBooleanPredicate(*spec, flags, ordinary);
    }
}

void FiltersTest::rarityFilter()
{
    FilterHarness harness;
    RaritySearchFilter filter(&harness.layout, &harness.rarityModel, harness.callbacks);
    FilterData data(&filter);

    const auto normal = makeFilterItem("normal", "", 0, "Normal");
    const auto rare = makeFilterItem("rare", "", 2, "Rare");
    const auto unique = makeFilterItem("unique", "", 3, "Unique");
    const auto foil = makeFilterItem("foil", "", 9, "Foil");

    data.text_query = "Any Non-Unique";
    QVERIFY(filter.Matches(normal, &data));
    QVERIFY(filter.Matches(rare, &data));
    QVERIFY(!filter.Matches(unique, &data));

    data.text_query = "Unique";
    QVERIFY(filter.Matches(unique, &data));
    QVERIFY(!filter.Matches(foil, &data));

    data.text_query = "Unique (Foil)";
    QVERIFY(filter.Matches(foil, &data));
    QVERIFY(!filter.Matches(unique, &data));
}

void FiltersTest::modsFilter()
{
    InitStatTranslations();
    AddStatTranslations(R"json([
        {"English":[{"format":["+#"],"string":"{0} to maximum Life"}]}
    ])json");
    InitModList();

    FilterHarness harness;
    ModsFilter filter(&harness.layout, harness.callbacks);
    const auto item = makeFilterItem("mods",
                                     R"json(,
        "explicitMods": ["+42 to maximum Life"]
    )json");
    FilterData data(&filter);

    data.mod_data.emplace_back("", 100.0, 1.0, true, true);
    QVERIFY(filter.Matches(item, &data));

    data.mod_data.emplace_back("+# to maximum Life", 40.0, 45.0, true, true);
    QVERIFY(filter.Matches(item, &data));
    data.mod_data.back().min = 43.0;
    QVERIFY(!filter.Matches(item, &data));
    data.mod_data.back().min = 40.0;
    data.mod_data.back().max = 41.0;
    QVERIFY(!filter.Matches(item, &data));
    data.mod_data.back().max = 45.0;
    data.mod_data.emplace_back("+# to missing stat", 0.0, 0.0, false, false);
    QVERIFY(!filter.Matches(item, &data));
}

void FiltersTest::colorsToFormKeepsStaleText()
{
    FilterHarness harness;
    SocketsColorsFilter filter(&harness.layout, harness.callbacks);
    FilterData withColors(&filter);
    withColors.r_filled = true;
    withColors.r = 2;
    withColors.g_filled = true;
    withColors.g = 1;
    withColors.b_filled = true;
    withColors.b = 3;
    withColors.ToForm();

    FilterData withoutColors(&filter);
    withoutColors.ToForm();

    const auto edits = harness.host.findChildren<QLineEdit *>();
    QCOMPARE(edits.size(), 3);
    QCOMPARE(edits[0]->text(), "2");
    QCOMPARE(edits[1]->text(), "1");
    QCOMPARE(edits[2]->text(), "3");
}

void FiltersTest::modsFormSyncQuirks()
{
    FilterHarness harness;
    mod_list_model().setStringList({"Default mod", "Saved mod"});
    ModsFilter filter(&harness.layout, harness.callbacks);
    auto *addButton = harness.host.findChild<QPushButton *>();
    QVERIFY(addButton);
    QCOMPARE(addButton->text(), "Add mod");

    FilterData searchA(&filter);
    searchA.FromForm();
    addButton->click();
    QCOMPARE(harness.host.findChildren<QComboBox *>().size(), 1);

    FilterData searchB(&filter);
    searchB.ToForm();
    searchA.ToForm();
    QCOMPARE(harness.host.findChildren<QComboBox *>().size(), 0);

    searchA.mod_data.emplace_back("Saved mod", 1.0, 2.0, true, true);
    searchA.ToForm();
    const auto combos = harness.host.findChildren<QComboBox *>();
    QCOMPARE(combos.size(), 1);
    QCOMPARE(searchA.mod_data.front().mod, "Saved mod");
    QCOMPARE(combos.front()->currentText(), "Default mod");

    FilterHarness visibilityHarness;
    ModsFilter visibilityFilter(&visibilityHarness.layout, visibilityHarness.callbacks);
    auto *rowContainer = visibilityHarness.layout.itemAt(0)->widget();
    auto *visibilityAddButton = visibilityHarness.host.findChild<QPushButton *>();
    QVERIFY(rowContainer);
    QVERIFY(visibilityAddButton);

    FilterData withRow(&visibilityFilter);
    withRow.mod_data.emplace_back("Saved mod", 0.0, 0.0, false, false);
    withRow.ToForm();
    QVERIFY(rowContainer->isHidden());

    visibilityAddButton->click();
    QVERIFY(!rowContainer->isHidden());
    FilterData empty(&visibilityFilter);
    empty.ToForm();
    QCOMPARE(visibilityHarness.host.findChildren<QComboBox *>().size(), 0);
    QVERIFY(!rowContainer->isHidden());
}

QTEST_MAIN(FiltersTest)

#include "tst_filters.moc"
