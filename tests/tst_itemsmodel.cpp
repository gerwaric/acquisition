#include <QtTest/QtTest>

#include <QAbstractItemModelTester>
#include <QSignalSpy>
#include <QTreeView>

#include <memory>
#include <vector>

#include "filters.h"
#include "filters/filterspec.h"
#include "items_model.h"
#include "search.h"
#include "testfixtures.h"

class ItemsModelTest : public QObject
{
    Q_OBJECT

private slots:
    void testerSurvivesRebuildModeSwitchAndSort();
    void selectionSurvivesSort();
};

static std::shared_ptr<Item> makeModelItem(const QString &id,
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

void ItemsModelTest::testerSurvivesRebuildModeSwitchAndSort()
{
    BuyoutManagerFixture buyoutFixture;
    const ItemLocation firstTab = makeTestStashLocation("stash-a", "Alpha Tab", 0);
    const ItemLocation secondTab = makeTestStashLocation("stash-b", "Beta Tab", 1);
    buyoutFixture.manager->SetStashTabLocations({firstTab, secondTab});

    Items items;
    items.push_back(makeModelItem("alpha-2", "Zulu Bite", "Vaal Axe", firstTab));
    items.push_back(makeModelItem("alpha-1", "Alpha Bite", "Copper Sword", firstTab));
    items.push_back(makeModelItem("beta-1", "Beta Guard", "Copper Shield", secondTab));

    QTreeView view;
    FilterCatalog catalog({});
    std::vector<Filter *> filters;
    Search search(*buyoutFixture.manager, "Model", catalog, filters);
    search.FilterItems(items);
    view.setModel(&search.model());

    auto *model = &search.model();
    QAbstractItemModelTester tester(model,
                                    QAbstractItemModelTester::FailureReportingMode::Fatal,
                                    this);

    QSignalSpy aboutToReset(model, &QAbstractItemModel::modelAboutToBeReset);
    QSignalSpy reset(model, &QAbstractItemModel::modelReset);
    search.SetRefreshReason(RefreshReason::SearchFormChanged);
    search.FilterItems(items);
    QCOMPARE(aboutToReset.count(), 1);
    QCOMPARE(reset.count(), 1);

    QSignalSpy dataChanged(model, &QAbstractItemModel::dataChanged);
    for (const auto &bucket : search.buckets()) {
        buyoutFixture.manager->SetRefreshChecked(bucket.location(), true);
    }
    model->refreshCheckStates();
    QCOMPARE(dataChanged.count(), 1);
    const auto dataChangedArguments = dataChanged.takeFirst();
    const auto roles = qvariant_cast<QList<int>>(dataChangedArguments.at(2));
    QCOMPARE(roles, QList<int>{Qt::CheckStateRole});

    aboutToReset.clear();
    reset.clear();
    search.SetViewMode(Search::ViewMode::ByItem);
    QCOMPARE(aboutToReset.count(), 1);
    QCOMPARE(reset.count(), 1);

    QStringList layoutEvents;
    QObject::connect(model,
                     &QAbstractItemModel::layoutAboutToBeChanged,
                     this,
                     [&](const QList<QPersistentModelIndex> &,
                         QAbstractItemModel::LayoutChangeHint) { layoutEvents.push_back("about"); });
    QObject::connect(model,
                     &QAbstractItemModel::layoutChanged,
                     this,
                     [&](const QList<QPersistentModelIndex> &,
                         QAbstractItemModel::LayoutChangeHint) {
                         layoutEvents.push_back("changed");
                     });
    QSignalSpy layoutAboutToChange(model, &QAbstractItemModel::layoutAboutToBeChanged);
    QSignalSpy layoutChanged(model, &QAbstractItemModel::layoutChanged);

    model->sort(0, Qt::AscendingOrder);
    QCOMPARE(layoutAboutToChange.count(), 1);
    QCOMPARE(layoutChanged.count(), 1);
    QCOMPARE(layoutEvents, QStringList({"about", "changed"}));
}

void ItemsModelTest::selectionSurvivesSort()
{
    BuyoutManagerFixture buyoutFixture;
    const ItemLocation firstTab = makeTestStashLocation("stash-a", "Alpha Tab", 0);
    buyoutFixture.manager->SetStashTabLocations({firstTab});

    Items items;
    items.push_back(makeModelItem("alpha-2", "Zulu Bite", "Vaal Axe", firstTab));
    items.push_back(makeModelItem("alpha-1", "Alpha Bite", "Copper Sword", firstTab));
    items.push_back(makeModelItem("alpha-3", "Mid Bite", "Iron Hammer", firstTab));
    items.push_back(makeModelItem("alpha-4", "Omega Bite", "Steel Dagger", firstTab));
    items.push_back(makeModelItem("alpha-5", "Beta Bite", "Bronze Mace", firstTab));

    QTreeView view;
    FilterCatalog catalog({});
    std::vector<Filter *> filters;
    Search search(*buyoutFixture.manager, "Model", catalog, filters);
    search.FilterItems(items);
    view.setModel(&search.model());

    auto *model = &search.model();

    const QModelIndex bucket0 = model->index(0, 0);
    QCOMPARE(model->rowCount(bucket0), 5);

    // Sort one way first so the opposite re-sort below is guaranteed to
    // reorder rows regardless of fixture insertion order; a no-op sort would
    // make the remap assertion vacuous. (Note Bucket::Sort maps the Qt order
    // enums to arrangements invertedly — F34 — so assert on the arrangement
    // changing, not on a specific direction.)
    model->sort(0, Qt::DescendingOrder);
    const QString first_before = model->index(0, 0, bucket0).data().toString();

    // Select rows 0-2 as one range, like a user shift-clicking three items.
    // The selection model tracks multi-row ranges with persistent indexes it
    // creates during layoutAboutToBeChanged; the model must remap them.
    const QItemSelection range(model->index(0, 0, bucket0), model->index(2, 0, bucket0));
    view.selectionModel()->select(range,
                                  QItemSelectionModel::ClearAndSelect | QItemSelectionModel::Rows);
    QStringList selected_before;
    for (const QModelIndex &idx : view.selectionModel()->selectedRows()) {
        selected_before << model->data(idx).toString();
    }
    selected_before.sort();
    QCOMPARE(selected_before.size(), 3);

    // Re-sort in the opposite order; the selection should follow the items.
    model->sort(0, Qt::AscendingOrder);
    QVERIFY(model->index(0, 0, bucket0).data().toString() != first_before);

    QStringList selected_after;
    for (const QModelIndex &idx : view.selectionModel()->selectedRows()) {
        selected_after << model->data(idx).toString();
    }
    selected_after.sort();
    QCOMPARE(selected_after, selected_before);
}

QTEST_MAIN(ItemsModelTest)

#include "tst_itemsmodel.moc"
