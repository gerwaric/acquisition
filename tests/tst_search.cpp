#include <QtTest/QtTest>

#include <QCheckBox>
#include <QComboBox>
#include <QLabel>
#include <QLineEdit>
#include <QVBoxLayout>
#include <QWidget>

#include <functional>

#include "itemcategories.h"
#include "search.h"
#include "testfixtures.h"
#include "ui/searchform.h"

class SearchTest : public QObject
{
    Q_OBJECT

private slots:
    void bucketConstruction();
    void nameFilterMembership();
    void backgroundRefilterUsesOwnState();
    void backgroundBooleanRefilterUsesOwnState();
    void backgroundMinMaxRefilterUsesOwnState();
    void filterStateRestoresAcrossTabSwitch();
    void booleanFormAdapterRoundTrip();
    void minMaxFormAdapterRoundTrip();
    void textAndComboFormAdapterRoundTrip();
};

static qsizetype findTextFilterIndex(const FilterCatalog &catalog, const QString &caption)
{
    for (qsizetype index = 0; index < catalog.size(); ++index) {
        const auto &spec = catalog[index];
        if ((spec.caption == caption) && std::holds_alternative<TextPayload>(spec.payload)) {
            return index;
        }
    }
    return -1;
}

static qsizetype findComboFilterIndex(const FilterCatalog &catalog, const QString &caption)
{
    for (qsizetype index = 0; index < catalog.size(); ++index) {
        const auto &spec = catalog[index];
        if ((spec.caption == caption) && std::holds_alternative<ComboPayload>(spec.payload)) {
            return index;
        }
    }
    return -1;
}

static qsizetype findMinMaxFilterIndex(const FilterCatalog &catalog, const QString &caption)
{
    for (qsizetype index = 0; index < catalog.size(); ++index) {
        const auto &spec = catalog[index];
        if ((spec.caption == caption) && std::holds_alternative<MinMaxPayload>(spec.payload)) {
            return index;
        }
    }
    return -1;
}

struct SearchHarness
{
    BuyoutManagerFixture buyoutFixture;
    QWidget host;
    QVBoxLayout layout{&host};
    QObject receiver;
    int immediateChanges = 0;
    int delayedChanges = 0;
    FilterCallbacks callbacks{
        &receiver,
        [this] { ++immediateChanges; },
        [this] { ++delayedChanges; },
    };
    FilterCatalog catalog{BuildFilterCatalog(*buyoutFixture.manager)};
    SearchForm form{layout, catalog, callbacks};

    template<typename Widget>
    Widget *findByLabel(const QString &labelText,
                        const std::function<bool(const Widget *)> &matches = {})
    {
        for (auto *label : host.findChildren<QLabel *>()) {
            if (label->text() != labelText) {
                continue;
            }
            for (auto *widget : label->parentWidget()->findChildren<Widget *>()) {
                if (!matches || matches(widget)) {
                    return widget;
                }
            }
        }
        return nullptr;
    }
};

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
    SearchHarness harness;
    const ItemLocation firstTab = makeTestStashLocation("stash-a", "Alpha Tab", 0);
    const ItemLocation secondTab = makeTestStashLocation("stash-b", "Beta Tab", 1);
    const ItemLocation emptyTab = makeTestStashLocation("stash-empty", "Empty Tab", 2);
    harness.buyoutFixture.manager->SetStashTabLocations({firstTab, secondTab, emptyTab});

    Items items;
    items.push_back(makeSearchItem("alpha-item", "Alpha Bite", "Vaal Axe", firstTab));
    items.push_back(makeSearchItem("beta-item", "Beta Guard", "Copper Shield", secondTab));

    Search search(*harness.buyoutFixture.manager,
                  "All",
                  harness.catalog,
                  harness.form.legacyFilters());
    harness.form.saveTo(search);
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
    auto *name = harness.findByLabel<QLineEdit>("Name");
    QVERIFY(name);
    name->setText("alpha");

    Items items;
    items.push_back(makeSearchItem("alpha-item", "Alpha Bite", "Vaal Axe", firstTab));
    items.push_back(makeSearchItem("beta-item", "Beta Guard", "Copper Shield", secondTab));

    Search search(*harness.buyoutFixture.manager,
                  "Filtered",
                  harness.catalog,
                  harness.form.legacyFilters());
    harness.form.saveTo(search);
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
    SearchHarness harness;
    const ItemLocation firstTab = makeTestStashLocation("stash-a", "Alpha Tab", 0);
    const ItemLocation secondTab = makeTestStashLocation("stash-b", "Beta Tab", 1);
    harness.buyoutFixture.manager->SetStashTabLocations({firstTab, secondTab});

    Items items;
    items.push_back(makeSearchItem("alpha-item", "Alpha Bite", "Vaal Axe", firstTab));
    items.push_back(makeSearchItem("beta-item", "Beta Guard", "Copper Shield", secondTab));

    auto *name = harness.findByLabel<QLineEdit>("Name");
    QVERIFY(name);
    name->setText("alpha");
    Search background(*harness.buyoutFixture.manager,
                      "Background",
                      harness.catalog,
                      harness.form.legacyFilters());
    harness.form.saveTo(background);

    name->setText("");
    Search current(*harness.buyoutFixture.manager,
                   "Current",
                   harness.catalog,
                   harness.form.legacyFilters());
    harness.form.saveTo(current);
    background.FilterItems(items);

    // F33 is fixed for text states: a background search uses its own saved
    // activity and query rather than the current search form's empty state.
    QCOMPARE(background.GetCaption(), "Background [1]");
    QCOMPARE(background.items().size(), 1);
    QCOMPARE(background.items().front()->id(), "alpha-item");

    harness.form.loadFrom(background);
    QCOMPARE(name->text(), "alpha");
}

void SearchTest::backgroundBooleanRefilterUsesOwnState()
{
    SearchHarness harness;
    const ItemLocation firstTab = makeTestStashLocation("stash-a", "Alpha Tab", 0);
    const ItemLocation secondTab = makeTestStashLocation("stash-b", "Beta Tab", 1);
    harness.buyoutFixture.manager->SetStashTabLocations({firstTab, secondTab});

    Items items;
    items.push_back(makeSearchItem("corrupted-item",
                                   "Corrupted Bite",
                                   "Vaal Axe",
                                   firstTab,
                                   R"json(, "corrupted": true)json"));
    items.push_back(makeSearchItem("ordinary-item", "Ordinary Guard", "Copper Shield", secondTab));

    auto *corrupted = harness.findByLabel<QCheckBox>("Corrupted");
    QVERIFY(corrupted);
    corrupted->setChecked(true);
    Search background(*harness.buyoutFixture.manager,
                      "Background",
                      harness.catalog,
                      harness.form.legacyFilters());
    harness.form.saveTo(background);

    corrupted->setChecked(false);
    Search current(*harness.buyoutFixture.manager,
                   "Current",
                   harness.catalog,
                   harness.form.legacyFilters());
    harness.form.saveTo(current);
    background.FilterItems(items);

    QCOMPARE(background.GetCaption(), "Background [1]");
    QCOMPARE(background.items().size(), 1);
    QCOMPARE(background.items().front()->id(), "corrupted-item");
}

void SearchTest::backgroundMinMaxRefilterUsesOwnState()
{
    SearchHarness harness;
    const ItemLocation firstTab = makeTestStashLocation("stash-a", "Alpha Tab", 0);
    const ItemLocation secondTab = makeTestStashLocation("stash-b", "Beta Tab", 1);
    harness.buyoutFixture.manager->SetStashTabLocations({firstTab, secondTab});

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

    auto *critMin = harness.findByLabel<QLineEdit>("Crit.", [](const QLineEdit *edit) {
        return edit->placeholderText() == "min";
    });
    QVERIFY(critMin);
    critMin->setText("5");
    Search background(*harness.buyoutFixture.manager,
                      "Background",
                      harness.catalog,
                      harness.form.legacyFilters());
    harness.form.saveTo(background);

    critMin->setText("");
    Search current(*harness.buyoutFixture.manager,
                   "Current",
                   harness.catalog,
                   harness.form.legacyFilters());
    harness.form.saveTo(current);
    background.FilterItems(items);

    QCOMPARE(background.GetCaption(), "Background [1]");
    QCOMPARE(background.items().size(), 1);
    QCOMPARE(background.items().front()->id(), "critical-item");
}

void SearchTest::filterStateRestoresAcrossTabSwitch()
{
    SearchHarness harness;
    auto *tab = harness.findByLabel<QLineEdit>("Tab");
    auto *name = harness.findByLabel<QLineEdit>("Name");
    auto *critMin = harness.findByLabel<QLineEdit>("Crit.", [](const QLineEdit *edit) {
        return edit->placeholderText() == "min";
    });
    auto *critMax = harness.findByLabel<QLineEdit>("Crit.", [](const QLineEdit *edit) {
        return edit->placeholderText() == "max";
    });
    auto *colorR = harness.findByLabel<QLineEdit>("Colors", [](const QLineEdit *edit) {
        return edit->placeholderText() == "R";
    });
    auto *colorG = harness.findByLabel<QLineEdit>("Colors", [](const QLineEdit *edit) {
        return edit->placeholderText() == "G";
    });
    auto *rarity = harness.findByLabel<QComboBox>("Rarity");
    auto *corrupted = harness.findByLabel<QCheckBox>("Corrupted");
    QVERIFY(tab);
    QVERIFY(name);
    QVERIFY(critMin);
    QVERIFY(critMax);
    QVERIFY(colorR);
    QVERIFY(colorG);
    QVERIFY(rarity);
    QVERIFY(corrupted);

    tab->setText("Alpha Tab");
    name->setText("alpha");
    critMin->setText("5");
    critMax->setText("10");
    colorR->setText("2");
    colorG->setText("1");
    rarity->setCurrentText("Rare");
    corrupted->setChecked(true);

    Search searchA(*harness.buyoutFixture.manager,
                   "A",
                   harness.catalog,
                   harness.form.legacyFilters());
    harness.form.saveTo(searchA);

    Search searchB(*harness.buyoutFixture.manager,
                   "B",
                   harness.catalog,
                   harness.form.legacyFilters());
    harness.form.reset();
    harness.form.saveTo(searchB);
    harness.form.loadFrom(searchA);

    QCOMPARE(tab->text(), "Alpha Tab");
    QCOMPARE(name->text(), "alpha");
    QCOMPARE(critMin->text(), "5");
    QCOMPARE(critMax->text(), "10");
    QCOMPARE(colorR->text(), "2");
    QCOMPARE(colorG->text(), "1");
    QCOMPARE(rarity->currentText(), "Rare");
    QVERIFY(corrupted->isChecked());
}

void SearchTest::booleanFormAdapterRoundTrip()
{
    SearchHarness harness;
    auto *corrupted = harness.findByLabel<QCheckBox>("Corrupted");
    QVERIFY(corrupted);

    QCOMPARE(harness.immediateChanges, 0);
    QCOMPARE(harness.delayedChanges, 0);
    corrupted->click();
    QCOMPARE(harness.immediateChanges, 1);
    QCOMPARE(harness.delayedChanges, 0);

    Search searchA(*harness.buyoutFixture.manager,
                   "A",
                   harness.catalog,
                   harness.form.legacyFilters());
    harness.form.saveTo(searchA);

    harness.form.reset();
    Search searchB(*harness.buyoutFixture.manager,
                   "B",
                   harness.catalog,
                   harness.form.legacyFilters());
    harness.form.saveTo(searchB);
    harness.form.loadFrom(searchA);

    QVERIFY(corrupted->isChecked());
    QCOMPARE(harness.immediateChanges, 1);
    QCOMPARE(harness.delayedChanges, 0);
}

void SearchTest::minMaxFormAdapterRoundTrip()
{
    SearchHarness harness;
    const auto &legacyFilters = harness.form.legacyFilters();
    QCOMPARE(static_cast<qsizetype>(legacyFilters.size()), harness.catalog.size());
    for (qsizetype index = 0; index < harness.catalog.size(); ++index) {
        QCOMPARE(legacyFilters.at(static_cast<size_t>(index)) != nullptr,
                 std::holds_alternative<LegacyPayload>(harness.catalog[index].payload));
    }

    auto *critMin = harness.findByLabel<QLineEdit>("Crit.", [](const QLineEdit *edit) {
        return edit->placeholderText() == "min";
    });
    auto *critMax = harness.findByLabel<QLineEdit>("Crit.", [](const QLineEdit *edit) {
        return edit->placeholderText() == "max";
    });
    QVERIFY(critMin);
    QVERIFY(critMax);
    const qsizetype critIndex = findMinMaxFilterIndex(harness.catalog, "Crit.");
    QVERIFY(critIndex >= 0);

    QCOMPARE(harness.immediateChanges, 0);
    QCOMPARE(harness.delayedChanges, 0);
    critMin->setFocus();
    QTest::keyClick(critMin, Qt::Key_1);
    QCOMPARE(harness.immediateChanges, 0);
    QCOMPARE(harness.delayedChanges, 1);

    critMin->setText("garbage");
    critMax->setText("12.5");
    Search searchA(*harness.buyoutFixture.manager,
                   "A",
                   harness.catalog,
                   harness.form.legacyFilters());
    harness.form.saveTo(searchA);

    const auto &state = std::get<MinMaxState>(searchA.filterStateAt(critIndex));
    QVERIFY(state.min.has_value());
    QCOMPARE(*state.min, 0.0);
    QVERIFY(state.max.has_value());
    QCOMPARE(*state.max, 12.5);
    QVERIFY(state.isActive());

    harness.form.reset();
    Search searchB(*harness.buyoutFixture.manager,
                   "B",
                   harness.catalog,
                   harness.form.legacyFilters());
    harness.form.saveTo(searchB);
    harness.form.loadFrom(searchB);
    QCOMPARE(critMin->text(), "");
    QCOMPARE(critMax->text(), "");
    harness.form.loadFrom(searchA);

    QCOMPARE(critMin->text(), "0");
    QCOMPARE(critMax->text(), "12.5");
    QCOMPARE(harness.immediateChanges, 0);
    QCOMPARE(harness.delayedChanges, 1);
}

void SearchTest::textAndComboFormAdapterRoundTrip()
{
    InitItemClasses(R"json({"TestClass":{"name":"Weapons"}})json");

    SearchHarness harness;
    const qsizetype tabIndex = findTextFilterIndex(harness.catalog, "Tab");
    const qsizetype nameIndex = findTextFilterIndex(harness.catalog, "Name");
    const qsizetype categoryIndex = findComboFilterIndex(harness.catalog, "Category");
    const qsizetype rarityIndex = findComboFilterIndex(harness.catalog, "Rarity");
    QVERIFY(tabIndex >= 0);
    QVERIFY(nameIndex >= 0);
    QVERIFY(categoryIndex >= 0);
    QVERIFY(rarityIndex >= 0);

    const auto &legacyFilters = harness.form.legacyFilters();
    QCOMPARE(static_cast<qsizetype>(legacyFilters.size()), harness.catalog.size());
    QVERIFY(legacyFilters.at(static_cast<size_t>(tabIndex)) == nullptr);
    QVERIFY(legacyFilters.at(static_cast<size_t>(nameIndex)) == nullptr);
    QVERIFY(legacyFilters.at(static_cast<size_t>(categoryIndex)) == nullptr);
    QVERIFY(legacyFilters.at(static_cast<size_t>(rarityIndex)) == nullptr);

    auto *tab = harness.findByLabel<QLineEdit>("Tab");
    auto *name = harness.findByLabel<QLineEdit>("Name");
    auto *category = harness.findByLabel<QComboBox>("Type");
    auto *rarity = harness.findByLabel<QComboBox>("Rarity");
    QVERIFY(tab);
    QVERIFY(name);
    QVERIFY(category);
    QVERIFY(rarity);

    const auto *categoryPayload = std::get_if<ComboPayload>(&harness.catalog[categoryIndex].payload);
    const auto *rarityPayload = std::get_if<ComboPayload>(&harness.catalog[rarityIndex].payload);
    QVERIFY(categoryPayload);
    QVERIFY(rarityPayload);
    const QStringList categoryChoices = categoryPayload->choices();
    const QStringList rarityChoices = rarityPayload->choices();
    QCOMPARE(category->count(), categoryChoices.size());
    QCOMPARE(rarity->count(), rarityChoices.size());
    for (qsizetype index = 0; index < categoryChoices.size(); ++index) {
        QCOMPARE(category->itemText(static_cast<int>(index)), categoryChoices.at(index));
    }
    for (qsizetype index = 0; index < rarityChoices.size(); ++index) {
        QCOMPARE(rarity->itemText(static_cast<int>(index)), rarityChoices.at(index));
    }

    const int weaponsIndex = category->findText("Weapons", Qt::MatchFixedString);
    QVERIFY(weaponsIndex >= 0);
    const int rareIndex = rarity->findText("Rare", Qt::MatchFixedString);
    QVERIFY(rareIndex >= 0);

    harness.immediateChanges = 0;
    harness.delayedChanges = 0;
    tab->setFocus();
    QTest::keyClick(tab, Qt::Key_T);
    QCOMPARE(harness.immediateChanges, 0);
    QCOMPARE(harness.delayedChanges, 1);
    name->setFocus();
    QTest::keyClick(name, Qt::Key_N);
    QCOMPARE(harness.immediateChanges, 0);
    QCOMPARE(harness.delayedChanges, 2);

    tab->setText("Alpha Tab");
    name->setText("Alpha");
    QCOMPARE(harness.immediateChanges, 0);
    QCOMPARE(harness.delayedChanges, 2);

    QVERIFY(category->currentIndex() != weaponsIndex);
    category->setCurrentIndex(weaponsIndex);
    QCOMPARE(harness.immediateChanges, 0);
    QCOMPARE(harness.delayedChanges, 3);
    QVERIFY(rarity->currentIndex() != rareIndex);
    rarity->setCurrentIndex(rareIndex);
    QCOMPARE(harness.immediateChanges, 0);
    QCOMPARE(harness.delayedChanges, 4);

    Search searchA(*harness.buyoutFixture.manager,
                   "A",
                   harness.catalog,
                   harness.form.legacyFilters());
    harness.form.saveTo(searchA);
    QCOMPARE(std::get<TextState>(searchA.filterStateAt(tabIndex)).query, "Alpha Tab");
    QCOMPARE(std::get<TextState>(searchA.filterStateAt(nameIndex)).query, "Alpha");
    QCOMPARE(std::get<ComboState>(searchA.filterStateAt(categoryIndex)).value, "weapons");
    QCOMPARE(std::get<ComboState>(searchA.filterStateAt(rarityIndex)).value, "Rare");

    harness.form.reset();
    Search searchB(*harness.buyoutFixture.manager,
                   "B",
                   harness.catalog,
                   harness.form.legacyFilters());
    harness.form.saveTo(searchB);
    QCOMPARE(std::get<ComboState>(searchB.filterStateAt(categoryIndex)).value, "");
    QCOMPARE(std::get<ComboState>(searchB.filterStateAt(rarityIndex)).value, "");

    category->setCurrentText("");
    harness.form.saveTo(searchB);
    QCOMPARE(std::get<ComboState>(searchB.filterStateAt(categoryIndex)).value, "");

    auto &rarityState = std::get<ComboState>(searchB.filterStateAt(rarityIndex));
    rarityState.value = "Rare";
    harness.immediateChanges = 0;
    harness.delayedChanges = 0;
    harness.form.loadFrom(searchB);
    QCOMPARE(rarity->currentText(), "Rare");
    QCOMPARE(harness.immediateChanges, 0);
    QCOMPARE(harness.delayedChanges, 2);

    rarityState.value = "Missing rarity";
    harness.immediateChanges = 0;
    harness.delayedChanges = 0;
    harness.form.loadFrom(searchB);
    QCOMPARE(rarity->currentIndex(), 0);
    QCOMPARE(rarity->currentText(), rarityPayload->anySentinel);
    QCOMPARE(harness.immediateChanges, 0);
    QCOMPARE(harness.delayedChanges, 1);

    category->setCurrentIndex(weaponsIndex);
    auto &categoryState = std::get<ComboState>(searchB.filterStateAt(categoryIndex));
    categoryState.value = "Missing category";
    harness.form.loadFrom(searchB);
    QCOMPARE(category->currentIndex(), 0);
    QCOMPARE(category->currentText(), categoryPayload->anySentinel);

    harness.form.loadFrom(searchA);
    QCOMPARE(tab->text(), "Alpha Tab");
    QCOMPARE(name->text(), "Alpha");
    QCOMPARE(category->currentText(), "Weapons");
    QCOMPARE(rarity->currentText(), "Rare");
}

QTEST_MAIN(SearchTest)

#include "tst_search.moc"
