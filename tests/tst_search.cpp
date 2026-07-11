#include <QtTest/QtTest>

#include <QCheckBox>
#include <QComboBox>
#include <QLabel>
#include <QLineEdit>
#include <QVBoxLayout>
#include <QWidget>

#include <functional>

#include "search.h"
#include "testfixtures.h"
#include "ui/searchform.h"

class SearchTest : public QObject
{
    Q_OBJECT

private slots:
    void bucketConstruction();
    void nameFilterMembership();
    void backgroundRefilterUsesCurrentSearchActivity();
    void filterStateRestoresAcrossTabSwitch();
};

struct SearchHarness
{
    BuyoutManagerFixture buyoutFixture;
    QWidget host;
    QVBoxLayout layout{&host};
    QObject receiver;
    FilterCallbacks callbacks{
        &receiver,
        [] {},
        [] {},
    };
    FilterCatalog catalog{BuildFilterCatalog(*buyoutFixture.manager)};
    SearchForm form{layout, catalog, *buyoutFixture.manager, callbacks};

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

    Search search(*harness.buyoutFixture.manager,
                  "All",
                  harness.catalog,
                  harness.form.legacyFilters());
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

void SearchTest::backgroundRefilterUsesCurrentSearchActivity()
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
    background.FromForm();

    name->setText("");
    Search current(*harness.buyoutFixture.manager,
                   "Current",
                   harness.catalog,
                   harness.form.legacyFilters());
    current.FromForm();
    background.FilterItems(items);

    // Characterizes F33: the current search's inactive flag causes the
    // background search to skip its saved name query.
    QCOMPARE(background.GetCaption(), "Background [2]");
    QCOMPARE(background.items().size(), 2);

    background.ToForm();
    QCOMPARE(name->text(), "alpha");
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
    searchA.FromForm();

    Search searchB(*harness.buyoutFixture.manager,
                   "B",
                   harness.catalog,
                   harness.form.legacyFilters());
    searchB.ResetForm();
    searchB.FromForm();
    searchA.ToForm();

    QCOMPARE(tab->text(), "Alpha Tab");
    QCOMPARE(name->text(), "alpha");
    QCOMPARE(critMin->text(), "5");
    QCOMPARE(critMax->text(), "10");
    QCOMPARE(colorR->text(), "2");
    QCOMPARE(colorG->text(), "1");
    QCOMPARE(rarity->currentText(), "Rare");
    QVERIFY(corrupted->isChecked());
}

QTEST_MAIN(SearchTest)

#include "tst_search.moc"
