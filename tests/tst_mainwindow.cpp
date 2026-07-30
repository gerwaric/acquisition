// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#include <QtTest/QtTest>

#include <QAbstractItemModel>
#include <QComboBox>
#include <QHeaderView>
#include <QItemSelectionModel>
#include <QLabel>
#include <QLineEdit>
#include <QScrollBar>
#include <QStandardPaths>
#include <QTabBar>
#include <QTreeView>

#include <memory>

#include <spdlog/logger.h>
#include <spdlog/sinks/dist_sink.h>
#include <spdlog/spdlog.h>

#include "buyout.h"
#include "currency.h"
#include "mainwindowfixture.h"

class MainWindowTest : public QObject
{
    Q_OBJECT

private slots:
    void initTestCase();
    void cleanupTestCase();
    void fixtureConstructsOffline();
    void tabChangeActivatesSelectedSearch();
    void itemsRefreshRefiltersBackgroundSearches();
    void pendingEditFollowsOutgoingSearch();
    void deleteTabDance();
    void currentViewStatePins();

    // Items-pipeline M2, stage 3: D6 stable-identity bucketing and the D9
    // five-rule streamed-delta consumer.
    void bucketsKeyOnStableIdDuringRefresh();
    void emptyDeltaMetadataLandsAtNextRefilter();
    void backgroundDeltaLeavesModelUntouched();
    void removalOnlyDeltaIntersects();
    void throttleDoesNotRearm();
    void tabSwitchBeforeTickPreservesDirty();
    void searchDeleteCancelsPendingTimer();
    void finalSnapshotCancelsPendingTick();
    void successfulRefilterCancelsPendingTick();
    void pendingTickSurvivesTerminalFailure();
    void childReconciliationIntersectsVisibleGhosts();

    // Items-pipeline M2, stage 5: the R6-3 restore-fidelity contract on the
    // throttled reset path.
    void expansionSurvivesRenameByStableKey();
    void selectionSurvivesReplacementByStableIdentity();
    void scrollAndCaptureSurviveThrottledReset();
    void reselectionSurvivesCrossTabMove();

private:
    std::shared_ptr<spdlog::logger> m_main_logger;
    std::shared_ptr<spdlog::sinks::dist_sink_mt> m_sink_hub;
};

static std::shared_ptr<Item> makeMainWindowItem(const QString &id,
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

static QTabBar *findSearchTabs(MainWindow &window)
{
    for (auto *tabs : window.findChildren<QTabBar *>()) {
        if ((tabs->count() > 0) && (tabs->tabText(tabs->count() - 1) == "+")) {
            return tabs;
        }
    }
    return nullptr;
}

static QLineEdit *findNameFilter(MainWindow &window)
{
    for (auto *label : window.findChildren<QLabel *>()) {
        if (label->text() != "Name") {
            continue;
        }
        auto *group = label->parentWidget();
        if (!group) {
            continue;
        }
        const auto edits = group->findChildren<QLineEdit *>();
        if (!edits.isEmpty()) {
            return edits.front();
        }
    }
    return nullptr;
}

static QStringList visibleItemNames(const QTreeView &view)
{
    const QAbstractItemModel *model = view.model();
    if (!model) {
        qFatal("visibleItemNames: the tree view has no model");
    }

    QStringList names;
    for (int bucketRow = 0; bucketRow < model->rowCount(); ++bucketRow) {
        const QModelIndex bucket = model->index(bucketRow, 0);
        for (int itemRow = 0; itemRow < model->rowCount(bucket); ++itemRow) {
            names.append(model->index(itemRow, 0, bucket).data().toString());
        }
    }
    return names;
}

static QModelIndex findBucket(const QAbstractItemModel &model, const QString &header)
{
    for (int row = 0; row < model.rowCount(); ++row) {
        const QModelIndex bucket = model.index(row, 0);
        if (bucket.data().toString().startsWith(header)) {
            return bucket;
        }
    }
    return QModelIndex();
}

void MainWindowTest::initTestCase()
{
    QStandardPaths::setTestModeEnabled(true);
    // LogPanel attaches its sinks through the dist-sink hub that
    // logging::init installs on the main logger (F42), so the test logger
    // needs one too.
    m_main_logger = std::make_shared<spdlog::logger>("main");
    m_sink_hub = std::make_shared<spdlog::sinks::dist_sink_mt>();
    m_main_logger->sinks().push_back(m_sink_hub);
    spdlog::register_logger(m_main_logger);
}

void MainWindowTest::cleanupTestCase()
{
    QCOMPARE(static_cast<int>(m_sink_hub->sinks().size()), 0);
    spdlog::drop("main");
    m_sink_hub.reset();
    m_main_logger.reset();
}

void MainWindowTest::fixtureConstructsOffline()
{
    QCOMPARE(static_cast<int>(m_sink_hub->sinks().size()), 0);
    {
        MainWindowFixture fixture;
        QVERIFY(fixture.window);
        QCOMPARE(static_cast<int>(m_sink_hub->sinks().size()), 2);
    }
    QCOMPARE(static_cast<int>(m_sink_hub->sinks().size()), 0);
}

void MainWindowTest::tabChangeActivatesSelectedSearch()
{
    MainWindowFixture fixture;
    auto *tabs = findSearchTabs(*fixture.window);
    auto *tree = fixture.window->findChild<QTreeView *>("treeView");
    auto *name = findNameFilter(*fixture.window);
    QVERIFY(tabs);
    QVERIFY(tree);
    QVERIFY(name);
    QCOMPARE(tabs->currentIndex(), 0);
    QSignalSpy nameEdited(name, &QLineEdit::textEdited);

    const ItemLocation alphaTab = makeTestStashLocation("stash-alpha", "Alpha Tab", 0);
    const ItemLocation betaTab = makeTestStashLocation("stash-beta", "Beta Tab", 1);
    Items items;
    items.push_back(makeMainWindowItem("alpha-item", "Alpha", "Sword", alphaTab));
    items.push_back(makeMainWindowItem("beta-item", "Beta", "Shield", betaTab));
    fixture.itemsManager->OnItemsRefreshed(items, {alphaTab, betaTab}, false);

    name->setFocus();
    QTest::keyClicks(name, "alpha");
    QCOMPARE(name->text(), "alpha");
    QCOMPARE(nameEdited.count(), 5);
    // The tab switch deterministically flushes the debounced edit before the
    // form is rebound to Search 2, and the flushed caption lands on the
    // outgoing search's own tab (F41).
    tabs->setCurrentIndex(1);
    QCOMPARE(tabs->tabText(0), "Search 1 [1]");
    QCOMPARE(tabs->tabText(1), "Search 2 [2]");

    QTest::keyClicks(name, "beta");
    tabs->setCurrentIndex(0);
    QCOMPARE(tabs->tabText(0), "Search 1 [1]");
    QCOMPARE(name->text(), "alpha");
    QCOMPARE(visibleItemNames(*tree), QStringList({"Alpha Sword"}));

    tabs->setCurrentIndex(1);
    QCOMPARE(tabs->tabText(1), "Search 2 [1]");
    QCOMPARE(name->text(), "beta");
    QCOMPARE(visibleItemNames(*tree), QStringList({"Beta Shield"}));
}

void MainWindowTest::itemsRefreshRefiltersBackgroundSearches()
{
    MainWindowFixture fixture;
    auto *tabs = findSearchTabs(*fixture.window);
    auto *name = findNameFilter(*fixture.window);
    QVERIFY(tabs);
    QVERIFY(name);

    const ItemLocation alphaTab = makeTestStashLocation("stash-alpha", "Alpha Tab", 0);
    const ItemLocation betaTab = makeTestStashLocation("stash-beta", "Beta Tab", 1);
    Items initialItems;
    initialItems.push_back(makeMainWindowItem("alpha-one", "Alpha One", "Sword", alphaTab));
    initialItems.push_back(makeMainWindowItem("beta-one", "Beta One", "Shield", betaTab));
    fixture.itemsManager->OnItemsRefreshed(initialItems, {alphaTab, betaTab}, false);

    name->setFocus();
    QTest::keyClicks(name, "alpha");
    tabs->setCurrentIndex(1);

    Items changedItems = initialItems;
    changedItems.push_back(makeMainWindowItem("alpha-two", "Alpha Two", "Axe", alphaTab));
    fixture.itemsManager->OnItemsRefreshed(changedItems, {alphaTab, betaTab}, false);

    // Search 1 is in the background, so this verifies the window-level F33
    // path rather than merely refiltering the active form.
    QCOMPARE(tabs->tabText(0), "Search 1 [2]");
    QCOMPARE(tabs->tabText(1), "Search 2 [3]");
}

void MainWindowTest::pendingEditFollowsOutgoingSearch()
{
    MainWindowFixture fixture;
    auto *tabs = findSearchTabs(*fixture.window);
    auto *name = findNameFilter(*fixture.window);
    QVERIFY(tabs);
    QVERIFY(name);

    const ItemLocation alphaTab = makeTestStashLocation("stash-alpha", "Alpha Tab", 0);
    const ItemLocation betaTab = makeTestStashLocation("stash-beta", "Beta Tab", 1);
    Items items;
    items.push_back(makeMainWindowItem("alpha-item", "Alpha", "Sword", alphaTab));
    items.push_back(makeMainWindowItem("beta-item", "Beta", "Shield", betaTab));
    fixture.itemsManager->OnItemsRefreshed(items, {alphaTab, betaTab}, false);

    name->setFocus();
    QTest::keyClicks(name, "alpha");
    // No elapsed-time wait: OnTabChange synchronously flushes the 350ms
    // debounce while Search 1 is still the outgoing search, and the flushed
    // caption targets Search 1's own tab (F41).
    tabs->setCurrentIndex(1);
    QCOMPARE(tabs->tabText(0), "Search 1 [1]");
    tabs->setCurrentIndex(0);

    // Re-activating renders the saved caption through the widget tree.
    QCOMPARE(tabs->tabText(0), "Search 1 [1]");
}

void MainWindowTest::deleteTabDance()
{
    {
        MainWindowFixture fixture;
        auto *tabs = findSearchTabs(*fixture.window);
        QVERIFY(tabs);
        QCOMPARE(tabs->count(), 2);

        const QRect onlySearch = tabs->tabRect(0);
        QVERIFY(onlySearch.isValid());
        QTest::mouseClick(tabs, Qt::MiddleButton, Qt::NoModifier, onlySearch.center());

        QCOMPARE(tabs->count(), 2);
        QVERIFY(tabs->tabText(0).startsWith("Search 2"));
        QCOMPARE(tabs->tabText(1), "+");
    }

    {
        MainWindowFixture fixture;
        auto *tabs = findSearchTabs(*fixture.window);
        QVERIFY(tabs);

        tabs->setCurrentIndex(1);
        tabs->setCurrentIndex(2);
        QCOMPARE(tabs->count(), 4);
        QCOMPARE(tabs->currentIndex(), 2);

        const QRect currentSearch = tabs->tabRect(2);
        QVERIFY(currentSearch.isValid());
        QTest::mouseClick(tabs, Qt::MiddleButton, Qt::NoModifier, currentSearch.center());

        QCOMPARE(tabs->count(), 3);
        QCOMPARE(tabs->currentIndex(), 1);
        QVERIFY(tabs->tabText(1).startsWith("Search 2"));
    }
}

void MainWindowTest::currentViewStatePins()
{
    {
        MainWindowFixture fixture;
        auto *tabs = findSearchTabs(*fixture.window);
        auto *tree = fixture.window->findChild<QTreeView *>("treeView");
        auto *name = findNameFilter(*fixture.window);
        QVERIFY(tabs);
        QVERIFY(tree);
        QVERIFY(name);

        const ItemLocation alphaTab = makeTestStashLocation("stash-alpha", "Alpha Tab", 0);
        const ItemLocation betaTab = makeTestStashLocation("stash-beta", "Beta Tab", 1);
        Items items;
        items.push_back(makeMainWindowItem("alpha-item", "Alpha", "Sword", alphaTab));
        items.push_back(makeMainWindowItem("beta-item", "Beta", "Shield", betaTab));
        fixture.itemsManager->OnItemsRefreshed(items, {alphaTab, betaTab}, false);

        const QModelIndex bucket = findBucket(*tree->model(), alphaTab.GetHeader());
        QVERIFY(bucket.isValid());
        tree->expand(bucket);
        QVERIFY(tree->isExpanded(bucket));
        const QModelIndex item = tree->model()->index(0, 0, bucket);
        QVERIFY(item.isValid());
        tree->selectionModel()->setCurrentIndex(item,
                                                QItemSelectionModel::ClearAndSelect
                                                    | QItemSelectionModel::Rows);

        tabs->setCurrentIndex(1);
        name->setFocus();
        QTest::keyClicks(name, "beta");
        tabs->setCurrentIndex(0);
        const QModelIndex restoredBucket = findBucket(*tree->model(), alphaTab.GetHeader());
        QVERIFY(restoredBucket.isValid());
        QVERIFY(tree->isExpanded(restoredBucket));
        const QModelIndexList selectedRows = tree->selectionModel()->selectedRows();
        QCOMPARE(selectedRows.size(), 1);
        QCOMPARE(selectedRows.front().data().toString(), "Alpha Sword");
    }

    {
        MainWindowFixture fixture;
        auto *tabs = findSearchTabs(*fixture.window);
        auto *tree = fixture.window->findChild<QTreeView *>("treeView");
        auto *name = findNameFilter(*fixture.window);
        auto *nameLabel = fixture.window->findChild<QLabel *>("nameLabel");
        auto *buyoutType = fixture.window->findChild<QComboBox *>("buyoutTypeComboBox");
        auto *buyoutCurrency = fixture.window->findChild<QComboBox *>("buyoutCurrencyComboBox");
        auto *buyoutValue = fixture.window->findChild<QLineEdit *>("buyoutValueLineEdit");
        QVERIFY(tabs);
        QVERIFY(tree);
        QVERIFY(name);
        QVERIFY(nameLabel);
        QVERIFY(buyoutType);
        QVERIFY(buyoutCurrency);
        QVERIFY(buyoutValue);

        const ItemLocation alphaTab = makeTestStashLocation("stash-alpha", "Alpha Tab", 0);
        const ItemLocation betaTab = makeTestStashLocation("stash-beta", "Beta Tab", 1);
        Items items;
        items.push_back(makeMainWindowItem("alpha-item", "Alpha", "Sword", alphaTab));
        items.push_back(makeMainWindowItem("beta-item", "Beta", "Shield", betaTab));
        fixture.itemsManager->OnItemsRefreshed(items, {alphaTab, betaTab}, false);

        name->setFocus();
        QTest::keyClicks(name, "alpha");
        tabs->setCurrentIndex(1);
        QTest::keyClicks(name, "beta");
        tabs->setCurrentIndex(0);
        fixture.buyoutFixture.manager->SetTab(alphaTab, makeChaosBuyout(7));

        const QModelIndex alphaBucket = findBucket(*tree->model(), alphaTab.GetHeader());
        QVERIFY(alphaBucket.isValid());
        tree->selectionModel()->setCurrentIndex(alphaBucket,
                                                QItemSelectionModel::ClearAndSelect
                                                    | QItemSelectionModel::Rows);
        QCOMPARE(nameLabel->text(), alphaTab.GetHeader());
        QCOMPARE(buyoutValue->text(), "7");

        tabs->setCurrentIndex(1);
        QVERIFY(!findBucket(*tree->model(), alphaTab.GetHeader()).isValid());
        QVERIFY(findBucket(*tree->model(), betaTab.GetHeader()).isValid());
        QCOMPARE(nameLabel->text(), "Select an item");
        QCOMPARE(buyoutType->currentIndex(), static_cast<int>(Buyout::BUYOUT_TYPE_INHERIT));
        QCOMPARE(buyoutCurrency->currentIndex(), static_cast<int>(Currency::CURRENCY_NONE));
        QCOMPARE(buyoutValue->text(), "");
        QVERIFY(!buyoutType->isEnabled());
        QVERIFY(!buyoutCurrency->isEnabled());
        QVERIFY(!buyoutValue->isEnabled());

        tabs->setCurrentIndex(0);
        QCOMPARE(nameLabel->text(), alphaTab.GetHeader());
        QCOMPARE(buyoutValue->text(), "7");
        // The restored bucket is highlighted in the tree, not just named in
        // the panel (F43).
        const QModelIndex restoredBucket = findBucket(*tree->model(), alphaTab.GetHeader());
        QVERIFY(restoredBucket.isValid());
        const QModelIndexList selectedRows = tree->selectionModel()->selectedRows();
        QCOMPARE(selectedRows.size(), 1);
        QCOMPARE(selectedRows.front(), restoredBucket);
    }

    {
        MainWindowFixture fixture;
        auto *tree = fixture.window->findChild<QTreeView *>("treeView");
        QVERIFY(tree);

        const ItemLocation alphaTab = makeTestStashLocation("stash-alpha", "Alpha Tab", 0);
        Items items;
        items.push_back(makeMainWindowItem("zulu-item", "Zulu", "Sword", alphaTab));
        items.push_back(makeMainWindowItem("alpha-item", "Alpha", "Sword", alphaTab));
        items.push_back(makeMainWindowItem("middle-item", "Middle", "Sword", alphaTab));
        fixture.itemsManager->OnItemsRefreshed(items, {alphaTab}, false);

        auto *model = tree->model();
        const QModelIndex bucket = model->index(0, 0);
        const QModelIndex firstItem = model->index(0, 0, bucket);
        const QModelIndex secondItem = model->index(1, 0, bucket);
        QVERIFY(firstItem.isValid());
        QVERIFY(secondItem.isValid());

        tree->selectionModel()->setCurrentIndex(firstItem,
                                                QItemSelectionModel::ClearAndSelect
                                                    | QItemSelectionModel::Rows);
        tree->selectionModel()->select(secondItem,
                                       QItemSelectionModel::Select | QItemSelectionModel::Rows);
        QCOMPARE(tree->selectionModel()->selectedRows().size(), 2);
        QStringList initiallySelectedNames;
        for (const QModelIndex &selected : tree->selectionModel()->selectedRows()) {
            initiallySelectedNames.append(selected.data().toString());
        }
        initiallySelectedNames.sort();

        QSignalSpy layoutChanged(model, &QAbstractItemModel::layoutChanged);
        const Qt::SortOrder nextOrder = tree->header()->sortIndicatorOrder() == Qt::AscendingOrder
                                            ? Qt::DescendingOrder
                                            : Qt::AscendingOrder;
        // Drive the header's sort path without depending on geometry from an
        // unshown window.
        tree->header()->setSortIndicator(0, nextOrder);

        QCOMPARE(layoutChanged.count(), 1);
        const QModelIndexList selectedRows = tree->selectionModel()->selectedRows();
        QCOMPARE(selectedRows.size(), 2);
        QStringList selectedNames;
        for (const QModelIndex &selected : selectedRows) {
            selectedNames.append(selected.data().toString());
        }
        selectedNames.sort();
        QCOMPARE(selectedNames, initiallySelectedNames);
    }
}

// M2 D6/R5-1 (stage 3): bucket identity is the stable (type, id) and each
// bucket renders the freshest metadata seen for that key. Mid-refresh deltas
// for a moved tab, a renamed tab, and a tab whose fresh position collides
// with an unrefreshed tab's stale position must produce exactly one bucket
// per stable tab id — no split, no stale header, no merge.
void MainWindowTest::bucketsKeyOnStableIdDuringRefresh()
{
    MainWindowFixture fixture;
    auto *tree = fixture.window->findChild<QTreeView *>("treeView");
    QVERIFY(tree);

    const ItemLocation tabA = makeTestStashLocation("stash-aaaa", "Alpha", 0);
    const ItemLocation tabB = makeTestStashLocation("stash-bbbb", "Beta", 1);
    const ItemLocation tabC = makeTestStashLocation("stash-cccc", "Gamma", 2);
    const ItemLocation tabD = makeTestStashLocation("stash-dddd", "Delta", 3);
    Items items;
    items.push_back(makeMainWindowItem("item-a", "AlphaItem", "Sword", tabA));
    items.push_back(makeMainWindowItem("item-b", "BetaItem", "Shield", tabB));
    items.push_back(makeMainWindowItem("item-c", "GammaItem", "Axe", tabC));
    items.push_back(makeMainWindowItem("item-d", "DeltaItem", "Wand", tabD));
    fixture.itemsManager->OnItemsRefreshed(items, {tabA, tabB, tabC, tabD}, false);

    // Mid-refresh deltas: A moves to position 5, B is renamed in place, and
    // C's fresh position collides with unrefreshed D's stale position 3.
    const ItemLocation movedA = makeTestStashLocation("stash-aaaa", "Alpha", 5);
    const ItemLocation renamedB = makeTestStashLocation("stash-bbbb", "Beta Renamed", 1);
    const ItemLocation collidedC = makeTestStashLocation("stash-cccc", "Gamma", 3);
    fixture.itemsManager
        ->OnTabRefreshed(movedA, {makeMainWindowItem("item-a2", "AlphaItem", "Sword", movedA)});
    fixture.itemsManager
        ->OnTabRefreshed(renamedB, {makeMainWindowItem("item-b2", "BetaItem", "Shield", renamedB)});
    fixture.itemsManager
        ->OnTabRefreshed(collidedC, {makeMainWindowItem("item-c2", "GammaItem", "Axe", collidedC)});

    // Force a refilter of the current search mid-refresh (the D9 throttle
    // path lands separately; any refilter must bucket soundly).
    fixture.window->OnSearchFormChange();

    const QAbstractItemModel *model = tree->model();
    QVERIFY(model);
    // Exactly one bucket per stable tab id: four tabs, four buckets.
    QCOMPARE(model->rowCount(), 4);
    QStringList headers;
    for (int row = 0; row < model->rowCount(); ++row) {
        headers.append(model->index(row, 0).data().toString());
    }
    // Fresh metadata renders: B under its new name, A at its new position
    // (after C and D, which collide on position 3 but never merge — the
    // stable id orders them deterministically).
    QCOMPARE(headers,
             QStringList(
                 {"#2, \"Beta Renamed\"", "#4, \"Gamma\"", "#4, \"Delta\"", "#6, \"Alpha\""}));
    // Each bucket holds exactly its tab's item: no split or merge moved an
    // item to a neighbor.
    QCOMPARE(visibleItemNames(*tree),
             QStringList({"BetaItem Shield", "GammaItem Axe", "DeltaItem Wand", "AlphaItem Sword"}));
}

// M2 D9/R6-1 (stage 3): metadata carried only by empty deltas — a renamed,
// a moved, and a newly discovered empty tab — starts no timer (no item
// intersects), and the next refilter of an unfiltered search renders those
// buckets from the canonical inventory's fresh anchors.
void MainWindowTest::emptyDeltaMetadataLandsAtNextRefilter()
{
    MainWindowFixture fixture;
    fixture.window->SetDeltaThrottleInterval(200);
    auto *tree = fixture.window->findChild<QTreeView *>("treeView");
    QVERIFY(tree);

    const ItemLocation tabA = makeTestStashLocation("stash-aaaa", "Alpha", 0);
    const ItemLocation tabB = makeTestStashLocation("stash-bbbb", "Beta", 1);
    const ItemLocation tabC = makeTestStashLocation("stash-cccc", "Gamma", 2);
    Items items;
    items.push_back(makeMainWindowItem("item-a", "AlphaItem", "Sword", tabA));
    fixture.itemsManager->OnItemsRefreshed(items, {tabA, tabB, tabC}, false);

    QSignalSpy resets(tree->model(), &QAbstractItemModel::modelReset);

    // Empty deltas: B renamed, C moved, and E newly discovered (never in
    // the published tab list).
    fixture.itemsManager->OnTabRefreshed(makeTestStashLocation("stash-bbbb", "Beta Renamed", 1), {});
    fixture.itemsManager->OnTabRefreshed(makeTestStashLocation("stash-cccc", "Gamma", 5), {});
    fixture.itemsManager->OnTabRefreshed(makeTestStashLocation("stash-eeee", "Epsilon", 4), {});

    // No item intersects, so no throttled refilter fires.
    QTest::qWait(400);
    QCOMPARE(resets.count(), 0);

    // The next refilter — whatever its cause — renders the fresh anchors.
    fixture.window->OnSearchFormChange();
    QCOMPARE(resets.count(), 1);
    const QAbstractItemModel *model = tree->model();
    QStringList headers;
    for (int row = 0; row < model->rowCount(); ++row) {
        headers.append(model->index(row, 0).data().toString());
    }
    QCOMPARE(headers,
             QStringList(
                 {"#1, \"Alpha\"", "#2, \"Beta Renamed\"", "#5, \"Epsilon\"", "#6, \"Gamma\""}));
}

// M2 D9 rules 1-2 (stage 3): a delta not intersecting the current search
// performs no model operation and marks every search items-dirty, including
// the current one — observed through the extended activation gate.
void MainWindowTest::backgroundDeltaLeavesModelUntouched()
{
    MainWindowFixture fixture;
    fixture.window->SetDeltaThrottleInterval(200);
    auto *tabs = findSearchTabs(*fixture.window);
    auto *tree = fixture.window->findChild<QTreeView *>("treeView");
    auto *name = findNameFilter(*fixture.window);
    QVERIFY(tabs && tree && name);

    const ItemLocation tabA = makeTestStashLocation("stash-aaaa", "Alpha", 0);
    const ItemLocation tabB = makeTestStashLocation("stash-bbbb", "Beta", 1);
    Items items;
    items.push_back(makeMainWindowItem("item-a", "AlphaItem", "Sword", tabA));
    items.push_back(makeMainWindowItem("item-b", "BetaItem", "Shield", tabB));
    fixture.itemsManager->OnItemsRefreshed(items, {tabA, tabB}, false);

    // Search 2 unfiltered in the background; search 1 current, filtered to
    // alpha so tab B contributes nothing visible.
    tabs->setCurrentIndex(1);
    tabs->setCurrentIndex(0);
    name->setFocus();
    QTest::keyClicks(name, "alphaitem");
    // The form edit is debounced; wait for its refilter to land before
    // observing the delta path.
    QTRY_COMPARE_WITH_TIMEOUT(visibleItemNames(*tree), QStringList({"AlphaItem Sword"}), 2000);

    QAbstractItemModel *current_model = tree->model();
    QSignalSpy resets(current_model, &QAbstractItemModel::modelReset);

    // A delta for tab B: no visible source, no filter match — no model
    // operation on the current search within S.
    fixture.itemsManager
        ->OnTabRefreshed(tabB, {makeMainWindowItem("item-b2", "BetaItem Two", "Shield", tabB)});
    QTest::qWait(400);
    QCOMPARE(resets.count(), 0);

    // But every search was marked dirty. The background search refilters on
    // activation and shows the new item...
    tabs->setCurrentIndex(1);
    QVERIFY(visibleItemNames(*tree).contains("BetaItem Two Shield"));
    // ...and the current search was marked too: switching back refilters it
    // through the extended activation gate (a plain tab change would skip).
    QSignalSpy current_resets(current_model, &QAbstractItemModel::modelReset);
    tabs->setCurrentIndex(0);
    QCOMPARE(current_resets.count(), 1);
}

// M2 D9 (stage 3): the removal half of the intersection test — an empty
// delta whose fetch source has items in the visible filtered result
// schedules the throttled refilter, and the tick empties the bucket.
void MainWindowTest::removalOnlyDeltaIntersects()
{
    MainWindowFixture fixture;
    fixture.window->SetDeltaThrottleInterval(200);
    auto *tree = fixture.window->findChild<QTreeView *>("treeView");
    QVERIFY(tree);

    const ItemLocation tabA = makeTestStashLocation("stash-aaaa", "Alpha", 0);
    Items items;
    items.push_back(makeMainWindowItem("item-a", "AlphaItem", "Sword", tabA));
    fixture.itemsManager->OnItemsRefreshed(items, {tabA}, false);
    QCOMPARE(visibleItemNames(*tree), QStringList({"AlphaItem Sword"}));

    QSignalSpy resets(tree->model(), &QAbstractItemModel::modelReset);
    fixture.itemsManager->OnTabRefreshed(tabA, {});

    // The delta carries no items, but something visible was fetched from
    // its source: the throttled refilter runs and the item leaves the view.
    QTRY_COMPARE_WITH_TIMEOUT(resets.count(), 1, 2000);
    QCOMPARE(visibleItemNames(*tree), QStringList());
}

// M2 D9 rule 2 (stage 3): a non-resetting trailing throttle — deltas
// arriving faster than S produce at most one refilter per S, and the first
// delta's deadline is not pushed back by later arrivals.
void MainWindowTest::throttleDoesNotRearm()
{
    MainWindowFixture fixture;
    fixture.window->SetDeltaThrottleInterval(500);
    auto *tree = fixture.window->findChild<QTreeView *>("treeView");
    QVERIFY(tree);

    const ItemLocation tabA = makeTestStashLocation("stash-aaaa", "Alpha", 0);
    Items items;
    items.push_back(makeMainWindowItem("item-a", "AlphaItem", "Sword", tabA));
    fixture.itemsManager->OnItemsRefreshed(items, {tabA}, false);

    QSignalSpy resets(tree->model(), &QAbstractItemModel::modelReset);

    // First intersecting delta at t=0 starts the window (deadline ~500ms);
    // a second at ~250ms must not push it to ~750ms.
    fixture.itemsManager
        ->OnTabRefreshed(tabA, {makeMainWindowItem("item-a2", "AlphaItem", "Sword", tabA)});
    QTest::qWait(250);
    fixture.itemsManager
        ->OnTabRefreshed(tabA, {makeMainWindowItem("item-a3", "AlphaItem", "Sword", tabA)});
    // At ~700ms the original deadline has passed and a re-armed one has
    // not: exactly one refilter proves the deadline held.
    QTest::qWait(450);
    QCOMPARE(resets.count(), 1);
    // And no second tick follows from the coalesced arrivals.
    QTest::qWait(600);
    QCOMPARE(resets.count(), 1);
}

// M2 D9 rule 4 (stage 3): switching searches with a tick pending cancels
// the timer; the old search refilters on its next activation via its
// items-dirty flag — nothing is lost.
void MainWindowTest::tabSwitchBeforeTickPreservesDirty()
{
    MainWindowFixture fixture;
    fixture.window->SetDeltaThrottleInterval(300);
    auto *tabs = findSearchTabs(*fixture.window);
    auto *tree = fixture.window->findChild<QTreeView *>("treeView");
    QVERIFY(tabs && tree);

    const ItemLocation tabA = makeTestStashLocation("stash-aaaa", "Alpha", 0);
    Items items;
    items.push_back(makeMainWindowItem("item-a", "AlphaItem", "Sword", tabA));
    fixture.itemsManager->OnItemsRefreshed(items, {tabA}, false);
    // A second search to switch to.
    tabs->setCurrentIndex(1);
    tabs->setCurrentIndex(0);

    QAbstractItemModel *first_model = tree->model();
    QSignalSpy first_resets(first_model, &QAbstractItemModel::modelReset);

    // Arm the tick, then switch away before it fires.
    fixture.itemsManager
        ->OnTabRefreshed(tabA, {makeMainWindowItem("item-a2", "AlphaItem Two", "Sword", tabA)});
    tabs->setCurrentIndex(1);
    const int resets_after_switch = first_resets.count();

    // The canceled tick never fires against the backgrounded search.
    QTest::qWait(600);
    QCOMPARE(first_resets.count(), resets_after_switch);

    // Its dirty flag carries the update to the next activation.
    tabs->setCurrentIndex(0);
    QCOMPARE(first_resets.count(), resets_after_switch + 1);
    QVERIFY(visibleItemNames(*tree).contains("AlphaItem Two Sword"));
}

// M2 D9 rule 4 (stage 3): deleting the current search with a tick pending
// fires nothing against the dead search.
void MainWindowTest::searchDeleteCancelsPendingTimer()
{
    MainWindowFixture fixture;
    fixture.window->SetDeltaThrottleInterval(300);
    auto *tabs = findSearchTabs(*fixture.window);
    auto *tree = fixture.window->findChild<QTreeView *>("treeView");
    QVERIFY(tabs && tree);

    const ItemLocation tabA = makeTestStashLocation("stash-aaaa", "Alpha", 0);
    Items items;
    items.push_back(makeMainWindowItem("item-a", "AlphaItem", "Sword", tabA));
    fixture.itemsManager->OnItemsRefreshed(items, {tabA}, false);
    // Search 2 becomes current; arm its tick.
    tabs->setCurrentIndex(1);
    fixture.itemsManager
        ->OnTabRefreshed(tabA, {makeMainWindowItem("item-a2", "AlphaItem", "Sword", tabA)});

    // Delete the current search with the tick pending. Search 1 takes over;
    // no tick may fire against the deleted search or spuriously reset the
    // survivor.
    fixture.window->OnDeleteTabClicked(1);
    QAbstractItemModel *survivor_model = tree->model();
    QSignalSpy survivor_resets(survivor_model, &QAbstractItemModel::modelReset);
    QTest::qWait(600);
    QCOMPARE(survivor_resets.count(), 0);
}

// M2 D9 rule 5 (stage 3): the final snapshot cancels a pending tick and the
// full path clears all items-dirty flags.
void MainWindowTest::finalSnapshotCancelsPendingTick()
{
    MainWindowFixture fixture;
    fixture.window->SetDeltaThrottleInterval(300);
    auto *tabs = findSearchTabs(*fixture.window);
    auto *tree = fixture.window->findChild<QTreeView *>("treeView");
    QVERIFY(tabs && tree);

    const ItemLocation tabA = makeTestStashLocation("stash-aaaa", "Alpha", 0);
    Items items;
    items.push_back(makeMainWindowItem("item-a", "AlphaItem", "Sword", tabA));
    fixture.itemsManager->OnItemsRefreshed(items, {tabA}, false);

    QSignalSpy resets(tree->model(), &QAbstractItemModel::modelReset);
    Items refreshed = items;
    refreshed.push_back(makeMainWindowItem("item-a2", "AlphaItem Two", "Sword", tabA));
    fixture.itemsManager->OnTabRefreshed(tabA, refreshed);

    // The final snapshot lands before the tick: one refilter from the
    // snapshot path, and the canceled tick adds no second reset.
    fixture.itemsManager->OnItemsRefreshed(refreshed, {tabA}, false);
    QCOMPARE(resets.count(), 1);
    QTest::qWait(600);
    QCOMPARE(resets.count(), 1);

    // All flags were cleared: switching away and back skips the refilter
    // (the activation gate finds nothing dirty).
    tabs->setCurrentIndex(1);
    tabs->setCurrentIndex(0);
    QCOMPARE(resets.count(), 1);
}

// M2 D9/R5-5 (stage 3): a user-initiated or form-edit refilter of the
// current search with a tick pending cancels the timer and clears the flag;
// no redundant reset follows.
void MainWindowTest::successfulRefilterCancelsPendingTick()
{
    MainWindowFixture fixture;
    fixture.window->SetDeltaThrottleInterval(300);
    auto *tree = fixture.window->findChild<QTreeView *>("treeView");
    QVERIFY(tree);

    const ItemLocation tabA = makeTestStashLocation("stash-aaaa", "Alpha", 0);
    Items items;
    items.push_back(makeMainWindowItem("item-a", "AlphaItem", "Sword", tabA));
    fixture.itemsManager->OnItemsRefreshed(items, {tabA}, false);

    QSignalSpy resets(tree->model(), &QAbstractItemModel::modelReset);
    fixture.itemsManager
        ->OnTabRefreshed(tabA, {makeMainWindowItem("item-a2", "AlphaItem Two", "Sword", tabA)});

    // The user pays for a refilter before the tick: the work is done once
    // and the pending deadline is canceled, not inherited.
    fixture.window->OnSearchFormChange();
    QCOMPARE(resets.count(), 1);
    QVERIFY(visibleItemNames(*tree).contains("AlphaItem Two Sword"));
    QTest::qWait(600);
    QCOMPARE(resets.count(), 1);
}

// M2 D9/R5-3, outcome (a) (stage 3): deltas followed by a terminal failure
// with a tick pending — the tick still fires and the view catches up within
// S plus one reset-plus-restore duration despite no final snapshot ever
// arriving. (The pin depends on the absence of a final snapshot, not on the
// terminal event, which lands in stage 6.)
void MainWindowTest::pendingTickSurvivesTerminalFailure()
{
    MainWindowFixture fixture;
    fixture.window->SetDeltaThrottleInterval(300);
    auto *tree = fixture.window->findChild<QTreeView *>("treeView");
    QVERIFY(tree);

    const ItemLocation tabA = makeTestStashLocation("stash-aaaa", "Alpha", 0);
    Items items;
    items.push_back(makeMainWindowItem("item-a", "AlphaItem", "Sword", tabA));
    fixture.itemsManager->OnItemsRefreshed(items, {tabA}, false);

    QSignalSpy resets(tree->model(), &QAbstractItemModel::modelReset);
    QElapsedTimer elapsed;
    elapsed.start();
    fixture.itemsManager
        ->OnTabRefreshed(tabA, {makeMainWindowItem("item-a2", "AlphaItem Two", "Sword", tabA)});

    // The update fails terminally: no final ItemsRefreshed ever arrives and
    // nothing cancels the tick. It fires at ~S and the view catches up —
    // within the amended freshness bound of S plus one reset-plus-restore.
    QTRY_COMPARE_WITH_TIMEOUT(resets.count(), 1, 2000);
    QVERIFY(elapsed.elapsed() < 300 + 1000); // S plus a generous reset bound
    QCOMPARE(visibleItemNames(*tree), QStringList({"AlphaItem Two Sword"}));
}

// M2 D9/R5-2/R6-2 (stage 3, plan-level addition): a ChildrenReconciled whose
// expected set excludes visible ghost children schedules the throttled
// refilter, and after a terminal failure (no final snapshot) the tick still
// fires and the ghosts leave the view.
void MainWindowTest::childReconciliationIntersectsVisibleGhosts()
{
    MainWindowFixture fixture;
    fixture.window->SetDeltaThrottleInterval(300);
    auto *tree = fixture.window->findChild<QTreeView *>("treeView");
    QVERIFY(tree);

    // A parent tab whose published items include one fetched from a child
    // source (a Map-style ghost candidate).
    const ItemLocation parent = makeTestStashLocation("stash-pppp", "Maps", 0);
    ItemLocation child_fetch = parent;
    child_fetch.setFetchId("child-0001");
    Items items;
    items.push_back(makeMainWindowItem("item-p", "ParentItem", "Sword", parent));
    items.push_back(makeMainWindowItem("item-m", "GhostItem", "Shield", child_fetch));
    fixture.itemsManager->OnItemsRefreshed(items, {parent}, false);
    QCOMPARE(visibleItemNames(*tree).size(), 2);

    QSignalSpy resets(tree->model(), &QAbstractItemModel::modelReset);

    // The reconciliation's expected set is the parent alone: the visible
    // ghost intersects, so the throttled refilter is scheduled even though
    // no primary delta touched the view.
    fixture.itemsManager->OnChildrenReconciled(parent,
                                               {FetchSourceKey{ItemLocationType::STASH,
                                                               "stash-pppp"}});

    // A terminal failure follows — no final snapshot. The tick still fires
    // and the ghost leaves the view.
    QTRY_COMPARE_WITH_TIMEOUT(resets.count(), 1, 2000);
    QCOMPARE(visibleItemNames(*tree), QStringList({"ParentItem Sword"}));
}

// M2 R6-3 (stage 5): expansion is keyed by the stable (type, id), so a
// delta that renames the expanded tab itself does not orphan the expansion
// state across the throttled reset — and the bucket renders the new name.
void MainWindowTest::expansionSurvivesRenameByStableKey()
{
    MainWindowFixture fixture;
    fixture.window->SetDeltaThrottleInterval(100);
    auto *tree = fixture.window->findChild<QTreeView *>("treeView");
    QVERIFY(tree);

    const ItemLocation tabA = makeTestStashLocation("stash-aaaa", "Alpha", 0);
    const ItemLocation tabB = makeTestStashLocation("stash-bbbb", "Beta", 1);
    Items items;
    items.push_back(makeMainWindowItem("item-a", "AlphaItem", "Sword", tabA));
    items.push_back(makeMainWindowItem("item-b", "BetaItem", "Shield", tabB));
    fixture.itemsManager->OnItemsRefreshed(items, {tabA, tabB}, false);

    const QModelIndex bucket = findBucket(*tree->model(), tabA.GetHeader());
    QVERIFY(bucket.isValid());
    tree->expand(bucket);
    QVERIFY(tree->isExpanded(bucket));

    // The delta carries the same stable id under a new label.
    const ItemLocation renamed = makeTestStashLocation("stash-aaaa", "AlphaPrime", 0);
    QSignalSpy resets(tree->model(), &QAbstractItemModel::modelReset);
    fixture.itemsManager
        ->OnTabRefreshed(renamed, {makeMainWindowItem("item-a", "AlphaItem", "Sword", renamed)});
    QTRY_COMPARE_WITH_TIMEOUT(resets.count(), 1, 2000);

    // The bucket renders the fresh metadata and is still expanded; the old
    // header no longer exists anywhere.
    QVERIFY(!findBucket(*tree->model(), tabA.GetHeader()).isValid());
    const QModelIndex renamedBucket = findBucket(*tree->model(), renamed.GetHeader());
    QVERIFY(renamedBucket.isValid());
    QVERIFY(tree->isExpanded(renamedBucket));
}

// M2 R6-3 (stage 5): a streamed replacement swaps the selected item's
// object for a new one with the same stable id. The selection follows the
// id, and the details panel adopts the replacement object rather than
// rendering the dead one.
void MainWindowTest::selectionSurvivesReplacementByStableIdentity()
{
    MainWindowFixture fixture;
    fixture.window->SetDeltaThrottleInterval(100);
    auto *tree = fixture.window->findChild<QTreeView *>("treeView");
    auto *locationLabel = fixture.window->findChild<QLabel *>("locationLabel");
    QVERIFY(tree);
    QVERIFY(locationLabel);

    const ItemLocation tabA = makeTestStashLocation("stash-aaaa", "Alpha", 0);
    Items items;
    items.push_back(makeMainWindowItem("item-a", "AlphaItem", "Sword", tabA));
    items.push_back(makeMainWindowItem("item-z", "ZuluItem", "Sword", tabA));
    fixture.itemsManager->OnItemsRefreshed(items, {tabA}, false);

    const QModelIndex bucket = findBucket(*tree->model(), tabA.GetHeader());
    QVERIFY(bucket.isValid());
    tree->expand(bucket);
    QModelIndex selected;
    for (int row = 0; row < tree->model()->rowCount(bucket); ++row) {
        const QModelIndex index = tree->model()->index(row, 0, bucket);
        if (index.data().toString() == "AlphaItem Sword") {
            selected = index;
        }
    }
    QVERIFY(selected.isValid());
    tree->selectionModel()->setCurrentIndex(selected,
                                            QItemSelectionModel::ClearAndSelect
                                                | QItemSelectionModel::Rows);

    // The replacement delta: a NEW Item object with the same stable id and
    // a different rendered name.
    QSignalSpy resets(tree->model(), &QAbstractItemModel::modelReset);
    fixture.itemsManager->OnTabRefreshed(tabA,
                                         {makeMainWindowItem("item-a",
                                                             "AlphaItem Two",
                                                             "Sword",
                                                             tabA),
                                          makeMainWindowItem("item-z", "ZuluItem", "Sword", tabA)});
    QTRY_COMPARE_WITH_TIMEOUT(resets.count(), 1, 2000);

    // The selection followed the stable id to the replacement object.
    const QModelIndexList selectedRows = tree->selectionModel()->selectedRows();
    QCOMPARE(selectedRows.size(), 1);
    QCOMPARE(selectedRows.front().data().toString(), "AlphaItem Two Sword");

    // The details panel re-rendered from the adopted object (the deferred
    // update path a user selection takes).
    QTRY_COMPARE_WITH_TIMEOUT(locationLabel->text(), tabA.GetHeader(), 2000);
}

// M2 R6-3 (stage 5): the throttled reset captures scroll immediately before
// resetting and restores by top-row anchor; when the anchored row was
// removed, the raw scrollbar value is the fallback — the anchor's bucket
// header is never scrolled to the top in its place.
void MainWindowTest::scrollAndCaptureSurviveThrottledReset()
{
    MainWindowFixture fixture;
    fixture.window->SetDeltaThrottleInterval(100);
    auto *tree = fixture.window->findChild<QTreeView *>("treeView");
    QVERIFY(tree);
    fixture.window->resize(900, 500);
    fixture.window->show();

    // Enough items that the tree scrolls.
    const ItemLocation tabA = makeTestStashLocation("stash-aaaa", "Alpha", 0);
    const ItemLocation tabB = makeTestStashLocation("stash-bbbb", "Beta", 1);
    Items items;
    for (int n = 0; n < 40; ++n) {
        items.push_back(makeMainWindowItem(QString("item-a%1").arg(n),
                                           QString("AlphaItem %1").arg(n),
                                           "Sword",
                                           tabA));
    }
    items.push_back(makeMainWindowItem("item-b", "BetaItem", "Shield", tabB));
    fixture.itemsManager->OnItemsRefreshed(items, {tabA, tabB}, false);

    const QModelIndex bucket = findBucket(*tree->model(), tabA.GetHeader());
    QVERIFY(bucket.isValid());
    tree->expand(bucket);
    // Let the offscreen window lay out so the view has a real viewport and
    // scroll range.
    QTRY_VERIFY_WITH_TIMEOUT(tree->verticalScrollBar()->maximum() > 0, 2000);

    // Scroll so an ordinary item row is the top row, and remember it.
    tree->verticalScrollBar()->setValue(tree->verticalScrollBar()->maximum() / 2);
    const QModelIndex topBefore = tree->indexAt(QPoint(0, 0));
    QVERIFY(topBefore.isValid());
    QVERIFY(topBefore.parent().isValid());
    const QString topName = topBefore.data().toString();

    // A delta that keeps every item: after the tick the same item is back
    // on top (anchor restore, not raw-value coincidence).
    QSignalSpy resets(tree->model(), &QAbstractItemModel::modelReset);
    fixture.itemsManager
        ->OnTabRefreshed(tabB, {makeMainWindowItem("item-b", "BetaItem Two", "Shield", tabB)});
    QTRY_COMPARE_WITH_TIMEOUT(resets.count(), 1, 2000);
    const QModelIndex topAfter = tree->indexAt(QPoint(0, 0));
    QVERIFY(topAfter.isValid());
    QCOMPARE(topAfter.data().toString(), topName);

    // Now remove the anchored item: the fallback is the raw scrollbar
    // value, never the anchor's bucket header scrolled to the top.
    const int valueBefore = tree->verticalScrollBar()->value();
    Items shrunk;
    for (int n = 0; n < 40; ++n) {
        const QString name = QString("AlphaItem %1").arg(n);
        if (name != topName.section(" Sword", 0, 0)) {
            shrunk.push_back(makeMainWindowItem(QString("item-a%1").arg(n), name, "Sword", tabA));
        }
    }
    fixture.itemsManager->OnTabRefreshed(tabA, shrunk);
    QTRY_COMPARE_WITH_TIMEOUT(resets.count(), 2, 2000);
    QCOMPARE(tree->verticalScrollBar()->value(), valueBefore);
    const QModelIndex topFallback = tree->indexAt(QPoint(0, 0));
    QVERIFY(topFallback.isValid());
    // Not the bucket header pinned to the top.
    QVERIFY(topFallback.parent().isValid());
}

// M2 R6-3 post-freeze amendment (stage 5): reselection is a GLOBAL
// stable-identity lookup — an item that moved to another tab mid-refresh
// keeps its selection under the new tab, with the replacement object
// adopted for the details panel. (The spike prototype was bucket-scoped;
// this pins the production behavior.)
void MainWindowTest::reselectionSurvivesCrossTabMove()
{
    MainWindowFixture fixture;
    fixture.window->SetDeltaThrottleInterval(100);
    auto *tree = fixture.window->findChild<QTreeView *>("treeView");
    auto *locationLabel = fixture.window->findChild<QLabel *>("locationLabel");
    QVERIFY(tree);
    QVERIFY(locationLabel);

    const ItemLocation tabA = makeTestStashLocation("stash-aaaa", "Alpha", 0);
    const ItemLocation tabB = makeTestStashLocation("stash-bbbb", "Beta", 1);
    Items items;
    items.push_back(makeMainWindowItem("item-x", "Mover", "Sword", tabA));
    items.push_back(makeMainWindowItem("item-b", "BetaItem", "Shield", tabB));
    fixture.itemsManager->OnItemsRefreshed(items, {tabA, tabB}, false);

    const QModelIndex bucketA = findBucket(*tree->model(), tabA.GetHeader());
    QVERIFY(bucketA.isValid());
    tree->expand(bucketA);
    const QModelIndex mover = tree->model()->index(0, 0, bucketA);
    QVERIFY(mover.isValid());
    QCOMPARE(mover.data().toString(), "Mover Sword");
    tree->selectionModel()->setCurrentIndex(mover,
                                            QItemSelectionModel::ClearAndSelect
                                                | QItemSelectionModel::Rows);

    // The move streams as two deltas inside one throttle window: the item
    // leaves tab A and arrives in tab B as a new object with the same id.
    QSignalSpy resets(tree->model(), &QAbstractItemModel::modelReset);
    fixture.itemsManager->OnTabRefreshed(tabA, {});
    fixture.itemsManager->OnTabRefreshed(tabB,
                                         {makeMainWindowItem("item-x", "Mover", "Sword", tabB),
                                          makeMainWindowItem("item-b",
                                                             "BetaItem",
                                                             "Shield",
                                                             tabB)});
    QTRY_COMPARE_WITH_TIMEOUT(resets.count(), 1, 2000);

    // The selection followed the item into tab B.
    const QModelIndexList selectedRows = tree->selectionModel()->selectedRows();
    QCOMPARE(selectedRows.size(), 1);
    QCOMPARE(selectedRows.front().data().toString(), "Mover Sword");
    const QModelIndex bucketB = findBucket(*tree->model(), tabB.GetHeader());
    QVERIFY(bucketB.isValid());
    QCOMPARE(selectedRows.front().parent(), bucketB);

    // The details panel adopted the replacement object: it renders the NEW
    // location.
    QTRY_COMPARE_WITH_TIMEOUT(locationLabel->text(), tabB.GetHeader(), 2000);
}

QTEST_MAIN(MainWindowTest)

#include "tst_mainwindow.moc"
