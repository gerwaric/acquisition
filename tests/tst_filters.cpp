#include <QtTest/QtTest>

#include <QComboBox>
#include <QLineEdit>
#include <QPushButton>
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
    void minMaxFilter();
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

static const FilterSpec *findMinMaxFilterSpec(const FilterCatalog &catalog, const QString &caption)
{
    for (const auto &spec : catalog) {
        if ((spec.caption == caption) && std::holds_alternative<MinMaxPayload>(spec.payload)) {
            return &spec;
        }
    }
    return nullptr;
}

static const FilterSpec *findTextFilterSpec(const FilterCatalog &catalog, const QString &caption)
{
    for (const auto &spec : catalog) {
        if ((spec.caption == caption) && std::holds_alternative<TextPayload>(spec.payload)) {
            return &spec;
        }
    }
    return nullptr;
}

static const FilterSpec *findComboFilterSpec(const FilterCatalog &catalog, const QString &caption)
{
    for (const auto &spec : catalog) {
        if ((spec.caption == caption) && std::holds_alternative<ComboPayload>(spec.payload)) {
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
    BuyoutManagerFixture buyoutFixture;
    const FilterCatalog catalog = BuildFilterCatalog(*buyoutFixture.manager);
    const auto *tab = findTextFilterSpec(catalog, "Tab");
    QVERIFY(tab);
    QCOMPARE(tab->refreshMode, RefreshMode::Debounced);
    const FilterState defaultState = MakeDefaultState(*tab);
    QVERIFY(std::holds_alternative<TextState>(defaultState));
    QVERIFY(!IsActive(defaultState));

    TextState state{"test"};
    QVERIFY(MatchesFilter(*makeFilterItem("tab-match", ""), *tab, state));
    state.query = "missing";
    QVERIFY(!MatchesFilter(*makeFilterItem("tab-miss", ""), *tab, state));
}

void FiltersTest::nameFilter()
{
    BuyoutManagerFixture buyoutFixture;
    const FilterCatalog catalog = BuildFilterCatalog(*buyoutFixture.manager);
    const auto *name = findTextFilterSpec(catalog, "Name");
    QVERIFY(name);
    QCOMPARE(name->refreshMode, RefreshMode::Debounced);

    TextState state{"alpha"};
    QVERIFY(MatchesFilter(*makeFilterItem("name-match", ""), *name, state));

    state.query = "ALPHA";
    QVERIFY(MatchesFilter(*makeFilterItem("name-match-case", ""), *name, state));

    state.query = "missing";
    QVERIFY(!MatchesFilter(*makeFilterItem("name-miss", ""), *name, state));
}

void FiltersTest::categoryFilter()
{
    InitItemClasses(R"json({"TestClass":{"name":"Weapons"}})json");
    InitItemBaseTypes(
        R"json({"Metadata/Items/TestSword":{"item_class":"TestClass","name":"Test Sword","release_state":"released"}})json");

    BuyoutManagerFixture buyoutFixture;
    const FilterCatalog catalog = BuildFilterCatalog(*buyoutFixture.manager);
    const auto *category = findComboFilterSpec(catalog, "Category");
    QVERIFY(category);
    QCOMPARE(category->refreshMode, RefreshMode::Debounced);
    const auto *payload = std::get_if<ComboPayload>(&category->payload);
    QVERIFY(payload);
    QCOMPARE(payload->matchKind, ComboMatchKind::CategoryContains);
    QCOMPARE(payload->anySentinel, kAnyFilterChoice);

    ComboState state{"weapons"};

    const auto weapon = makeFilterItem("category-match", "", 2, "Rare", {}, "Test Sword");
    const auto unknown = makeFilterItem("category-miss", "");
    QVERIFY(MatchesFilter(*weapon, *category, state));
    QVERIFY(!MatchesFilter(*unknown, *category, state));
}

void FiltersTest::minMaxFilter()
{
    BuyoutManagerFixture buyoutFixture;
    const FilterCatalog catalog = BuildFilterCatalog(*buyoutFixture.manager);

    for (const QString &caption :
         {"Crit.",   "DPS",    "pDPS",    "eDPS",    "cDPS",     "APS",      "Armour",
          "Evasion", "Shield", "Block",   "Sockets", "Links",    "R. Level", "R. Str",
          "R. Dex",  "R. Int", "Quality", "Level",   "Map Tier", "ilvl"}) {
        const auto *spec = findMinMaxFilterSpec(catalog, caption);
        QVERIFY(spec);
        QCOMPARE(spec->refreshMode, RefreshMode::Debounced);
        const FilterState defaultState = MakeDefaultState(*spec);
        QVERIFY(std::holds_alternative<MinMaxState>(defaultState));
        QVERIFY(!IsActive(defaultState));
    }

    const auto *crit = findMinMaxFilterSpec(catalog, "Crit.");
    QVERIFY(crit);
    const auto withCrit = makeFilterItem("crit",
                                         R"json(,
        "properties": [
            {
                "displayMode": 0,
                "name": "Critical Strike Chance",
                "type": 6,
                "values": [["20", 1]]
            }
        ])json");
    const auto withoutCrit = makeFilterItem("missing-crit", "");

    MinMaxState state;
    state.min = 10.0;
    QVERIFY(MatchesFilter(*withCrit, *crit, state));

    state.min = 25.0;
    QVERIFY(!MatchesFilter(*withCrit, *crit, state));

    state.min.reset();
    state.max = 20.0;
    QVERIFY(MatchesFilter(*withCrit, *crit, state));

    state.max = 19.0;
    QVERIFY(!MatchesFilter(*withCrit, *crit, state));

    state.min = 1.0;
    state.max.reset();
    QVERIFY(!MatchesFilter(*withoutCrit, *crit, state));

    state.min.reset();
    QVERIFY(MatchesFilter(*withoutCrit, *crit, state));
}

void FiltersTest::defaultAndRequiredFilters()
{
    BuyoutManagerFixture buyoutFixture;
    const FilterCatalog catalog = BuildFilterCatalog(*buyoutFixture.manager);
    const auto withoutQuality = makeFilterItem("default-quality", "");
    const auto *quality = findMinMaxFilterSpec(catalog, "Quality");
    QVERIFY(quality);
    MinMaxState qualityState;
    qualityState.min = 0.0;
    QVERIFY(MatchesFilter(*withoutQuality, *quality, qualityState));
    qualityState.min = 1.0;
    QVERIFY(!MatchesFilter(*withoutQuality, *quality, qualityState));

    const auto withRequirement = makeFilterItem("requirement",
                                                R"json(,
        "requirements": [
            {"displayMode": 0, "name": "Level", "values": [["12", 0]]}
        ])json");
    const auto withoutRequirement = makeFilterItem("no-requirement", "");
    const auto *required = findMinMaxFilterSpec(catalog, "R. Level");
    QVERIFY(required);
    MinMaxState requiredState;
    requiredState.min = 12.0;
    QVERIFY(MatchesFilter(*withRequirement, *required, requiredState));
    QVERIFY(!MatchesFilter(*withoutRequirement, *required, requiredState));
    requiredState.min.reset();
    requiredState.max = 0.0;
    QVERIFY(MatchesFilter(*withoutRequirement, *required, requiredState));
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
    BuyoutManagerFixture buyoutFixture;
    const FilterCatalog catalog = BuildFilterCatalog(*buyoutFixture.manager);
    const auto *rarity = findComboFilterSpec(catalog, "Rarity");
    QVERIFY(rarity);
    QCOMPARE(rarity->refreshMode, RefreshMode::Debounced);
    const auto *payload = std::get_if<ComboPayload>(&rarity->payload);
    QVERIFY(payload);
    QCOMPARE(payload->matchKind, ComboMatchKind::Rarity);
    QCOMPARE(payload->anySentinel, kAnyFilterChoice);

    const auto normal = makeFilterItem("normal", "", 0, "Normal");
    const auto rare = makeFilterItem("rare", "", 2, "Rare");
    const auto unique = makeFilterItem("unique", "", 3, "Unique");
    const auto foil = makeFilterItem("foil", "", 9, "Foil");

    ComboState state{"Any Non-Unique"};
    QVERIFY(MatchesFilter(*normal, *rarity, state));
    QVERIFY(MatchesFilter(*rare, *rarity, state));
    QVERIFY(!MatchesFilter(*unique, *rarity, state));

    state.value = "Unique";
    QVERIFY(MatchesFilter(*unique, *rarity, state));
    QVERIFY(!MatchesFilter(*foil, *rarity, state));

    state.value = "Unique (Foil)";
    QVERIFY(MatchesFilter(*foil, *rarity, state));
    QVERIFY(!MatchesFilter(*unique, *rarity, state));
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
