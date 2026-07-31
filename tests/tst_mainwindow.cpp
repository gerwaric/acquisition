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
#include "modelprobes.h"

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

    // Items-pipeline M3, S0: the capture/restore half of the probe surface.
    void probeCountersTrackCaptureRestore();

    // Items-pipeline M3, S2: the buyout choke point and the five batching
    // rules (D1 rule 4, R1-6/R2-5/R3-3/R3-4). priceKeysFollowBuyoutEdits
    // closed its behavioral half in S2 (reorder at batch end) and
    // completed in S3 with its R3-2 resident-key assertions.
    void priceKeysFollowBuyoutEdits();
    void multiSelectionBuyoutEditReordersOnce();
    void pricingPassYieldsSingleModelUpdate();
    void snapshotPricingSequenceEmitsOneModelBatch();
    void priceCellsRepaintUnderAnySortColumn();
    void buyoutRepaintCoversEveryVisibleOccurrence();

    // Items-pipeline M3, S3: D2 deferred sorting (per-bucket flags, the
    // full transition table) and D1 key residency (hydration, eviction,
    // the invalidation contract). residentKeysScopedToActiveSearch is
    // PARTIAL here: its By-Tab laziness, deactivation-eviction, and
    // aggregate-memory clauses are S3's; its clean-By-Item
    // eager-hydration clause first becomes satisfiable in S5, where the
    // pin fully closes.
    void collapsedBucketsDeferSorting();
    void restoredExpansionSortsRestoredBucketsOnly();
    void filteredSearchSortsAllVisibleBuckets();
    void sortedOrderSurvivesCollapse();
    void keyResidencyFollowsMaterialization();
    void reexpandedBucketFlipHydratesOnce();
    void sortColumnSwitchResortsVisibleBucketsOnly();
    void residentKeysScopedToActiveSearch();

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

static std::shared_ptr<Item> makeMainWindowItemWithNote(const QString &id,
                                                        const QString &name,
                                                        const QString &typeLine,
                                                        const ItemLocation &location,
                                                        const QString &note)
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
        "note": "%4",
        "typeLine": "%3",
        "verified": false,
        "w": 1,
        "x": 0,
        "y": 0
    })json")
                                .arg(id, name, typeLine, note)
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

static QStringList bucketItemNames(const QAbstractItemModel &model, const QModelIndex &bucket)
{
    QStringList names;
    for (int row = 0; row < model.rowCount(bucket); ++row) {
        names.append(model.index(row, 0, bucket).data().toString());
    }
    return names;
}

static QModelIndex findItemRow(const QAbstractItemModel &model,
                               const QModelIndex &bucket,
                               const QString &name)
{
    for (int row = 0; row < model.rowCount(bucket); ++row) {
        const QModelIndex index = model.index(row, 0, bucket);
        if (index.data().toString() == name) {
            return index;
        }
    }
    return QModelIndex();
}

// One user buyout command (M3 R2-5): the widget states a user would leave
// behind, then the command boundary itself. setCurrentIndex/setText emit no
// activated/textEdited, so this drives OnBuyoutChange exactly once — the
// command applies to whatever the tree's selection model holds.
static void applyBuyoutCommand(MainWindowFixture &fixture,
                               int typeIndex,
                               int currencyIndex,
                               const QString &value)
{
    auto *buyoutType = fixture.window->findChild<QComboBox *>("buyoutTypeComboBox");
    auto *buyoutCurrency = fixture.window->findChild<QComboBox *>("buyoutCurrencyComboBox");
    auto *buyoutValue = fixture.window->findChild<QLineEdit *>("buyoutValueLineEdit");
    QVERIFY(buyoutType);
    QVERIFY(buyoutCurrency);
    QVERIFY(buyoutValue);
    buyoutType->setCurrentIndex(typeIndex);
    buyoutCurrency->setCurrentIndex(currencyIndex);
    buyoutValue->setText(value);
    fixture.window->OnBuyoutChange();
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
    fixture.window->resize(900, 500);
    fixture.window->show();

    // Enough items that the tree scrolls, so the switch-away scroll capture
    // is observable on reactivation.
    const ItemLocation tabA = makeTestStashLocation("stash-aaaa", "Alpha", 0);
    Items items;
    for (int n = 0; n < 40; ++n) {
        items.push_back(makeMainWindowItem(QString("item-a%1").arg(n),
                                           QString("AlphaItem %1").arg(n),
                                           "Sword",
                                           tabA));
    }
    fixture.itemsManager->OnItemsRefreshed(items, {tabA}, false);
    // A second search to switch to.
    tabs->setCurrentIndex(1);
    tabs->setCurrentIndex(0);

    QAbstractItemModel *first_model = tree->model();
    QSignalSpy first_resets(first_model, &QAbstractItemModel::modelReset);

    // Scroll an ordinary item row to the top and remember it: the switch
    // away must capture this (R6-3) — the reactivation refilter resets the
    // model while the view still shows the OTHER search, so the capture at
    // switch-away is the only one there is.
    const QModelIndex bucket = findBucket(*tree->model(), tabA.GetHeader());
    QVERIFY(bucket.isValid());
    tree->expand(bucket);
    QTRY_VERIFY_WITH_TIMEOUT(tree->verticalScrollBar()->maximum() > 0, 2000);
    tree->verticalScrollBar()->setValue(tree->verticalScrollBar()->maximum() / 2);
    const QModelIndex topBefore = tree->indexAt(QPoint(0, 0));
    QVERIFY(topBefore.isValid());
    QVERIFY(topBefore.parent().isValid());
    const QString topName = topBefore.data().toString();

    // Arm the tick, then switch away before it fires. The delta is the
    // tab's complete replacement (a delta REPLACES its fetch source): all
    // forty items, one renamed — so the anchored row survives and the
    // anchor is restorable after the reactivation refilter.
    Items delta;
    delta.push_back(makeMainWindowItem("item-a0", "AlphaItem 0 Two", "Sword", tabA));
    for (int n = 1; n < 40; ++n) {
        delta.push_back(makeMainWindowItem(QString("item-a%1").arg(n),
                                           QString("AlphaItem %1").arg(n),
                                           "Sword",
                                           tabA));
    }
    fixture.itemsManager->OnTabRefreshed(tabA, delta);
    tabs->setCurrentIndex(1);
    const int resets_after_switch = first_resets.count();

    // The canceled tick never fires against the backgrounded search.
    QTest::qWait(600);
    QCOMPARE(first_resets.count(), resets_after_switch);

    // Its dirty flag carries the update to the next activation, and the
    // scroll captured at switch-away survives the reactivation reset.
    tabs->setCurrentIndex(0);
    QCOMPARE(first_resets.count(), resets_after_switch + 1);
    QVERIFY(visibleItemNames(*tree).contains("AlphaItem 0 Two Sword"));
    const QModelIndex topAfter = tree->indexAt(QPoint(0, 0));
    QVERIFY(topAfter.isValid());
    QCOMPARE(topAfter.data().toString(), topName);
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
    fixture.itemsManager
        ->OnTabRefreshed(tabA,
                         {makeMainWindowItem("item-a", "AlphaItem Two", "Sword", tabA),
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
                                          makeMainWindowItem("item-b", "BetaItem", "Shield", tabB)});
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

// The S0 probe surface, window half: one snapshot refresh on a single
// search drives exactly one capture/restore cycle — the counters the M3
// pins will read to prove the restore machinery does NOT run on the
// delta path (`unrelatedDeltaLeavesOtherBucketsUntouched`).
void MainWindowTest::probeCountersTrackCaptureRestore()
{
    MainWindowFixture fixture;

    const ItemLocation tabA = makeTestStashLocation("stash-alpha", "Alpha Tab", 0);
    Items items;
    items.push_back(makeMainWindowItem("alpha-one", "Alpha One", "Sword", tabA));

    auto &probes = ModelProbes::instance();
    probes.reset();
    probes.enabled = true;
    fixture.itemsManager->OnItemsRefreshed(items, {tabA}, false);

    QCOMPARE(probes.expansion_captures, 1);
    QCOMPARE(probes.scroll_captures, 1);
    QCOMPARE(probes.expansion_restores, 1);
    QCOMPARE(probes.scroll_restores, 1);
    QCOMPARE(probes.reselects, 1);
    QCOMPARE(probes.refilters, 1);
    probes.enabled = false;
}

// M3 S2, behavioral half of `priceKeysFollowBuyoutEdits` (D1 rule 4): with
// Price active, user buyout edits — item and tab level, set and clear —
// reorder affected rows at command end, one model update per command
// (R2-5/R3-3); migration reorders within the snapshot's outer batch, not at
// a command end (R1-6/R2-6, R3-4); Date is symmetric. The resident-key
// assertions (R3-2: entries rebuild before the reorder) land in S3.
void MainWindowTest::priceKeysFollowBuyoutEdits()
{
    {
        MainWindowFixture fixture;
        auto *tree = fixture.window->findChild<QTreeView *>("treeView");
        QVERIFY(tree);

        const ItemLocation tabA = makeTestStashLocation("stash-alpha", "Alpha Tab", 0);
        Items items;
        items.push_back(makeMainWindowItem("item-a", "Alpha", "Sword", tabA));
        items.push_back(makeMainWindowItem("item-b", "Bravo", "Sword", tabA));
        items.push_back(makeMainWindowItem("item-c", "Charlie", "Sword", tabA));
        fixture.itemsManager->OnItemsRefreshed(items, {tabA}, false);

        // Price ascending: unpriced (currency rank 0) sorts before priced.
        tree->header()->setSortIndicator(1, Qt::AscendingOrder);
        auto *model = tree->model();
        const QModelIndex bucket = findBucket(*model, tabA.GetHeader());
        QVERIFY(bucket.isValid());
        tree->expand(bucket);
        QCOMPARE(bucketItemNames(*model, bucket),
                 QStringList({"Alpha Sword", "Bravo Sword", "Charlie Sword"}));

        auto &probes = ModelProbes::instance();
        probes.reset();
        probes.enabled = true;

        // The expanded bucket's Price keys are resident (S3): the R3-2
        // assertions below hold the whole command sequence to "entries
        // rebuild, then one reorder on the resident vector" — zero full
        // key builds, one bucket sort per command, never a re-sort on
        // stale keys (the order assertions would catch one).
        QVERIFY(probes.live_key_bytes > 0);

        // Item-level set: the affected row moves at command end.
        tree->selectionModel()->setCurrentIndex(findItemRow(*model, bucket, "Alpha Sword"),
                                                QItemSelectionModel::ClearAndSelect
                                                    | QItemSelectionModel::Rows);
        probes.reset();
        applyBuyoutCommand(fixture, Buyout::BUYOUT_TYPE_BUYOUT, Currency::CURRENCY_CHAOS_ORB, "7");
        QCOMPARE(probes.model_updates, 1);
        QCOMPARE(probes.bucket_sorts, 1);
        QCOMPARE(probes.key_builds, 0);
        QCOMPARE(bucketItemNames(*model, bucket),
                 QStringList({"Bravo Sword", "Charlie Sword", "Alpha Sword"}));

        // Tab-level set: the propagated inherited prices (9c on Bravo and
        // Charlie) reorder against Alpha's manual 7c, one update for the
        // whole command including the nested propagation pass (R3-3).
        tree->selectionModel()->setCurrentIndex(findBucket(*model, tabA.GetHeader()),
                                                QItemSelectionModel::ClearAndSelect
                                                    | QItemSelectionModel::Rows);
        probes.reset();
        applyBuyoutCommand(fixture, Buyout::BUYOUT_TYPE_BUYOUT, Currency::CURRENCY_CHAOS_ORB, "9");
        QCOMPARE(probes.model_updates, 1);
        QCOMPARE(probes.key_builds, 0);
        QCOMPARE(bucketItemNames(*model, bucket),
                 QStringList({"Alpha Sword", "Bravo Sword", "Charlie Sword"}));

        // Tab-level clear: the inherited prices vanish at command end.
        tree->selectionModel()->setCurrentIndex(findBucket(*model, tabA.GetHeader()),
                                                QItemSelectionModel::ClearAndSelect
                                                    | QItemSelectionModel::Rows);
        probes.reset();
        applyBuyoutCommand(fixture, Buyout::BUYOUT_TYPE_INHERIT, Currency::CURRENCY_NONE, "");
        QCOMPARE(probes.model_updates, 1);
        QCOMPARE(probes.key_builds, 0);
        QCOMPARE(bucketItemNames(*model, bucket),
                 QStringList({"Bravo Sword", "Charlie Sword", "Alpha Sword"}));

        // Item-level clear: back to the unpriced tie-break order.
        tree->selectionModel()->setCurrentIndex(findItemRow(*model, bucket, "Alpha Sword"),
                                                QItemSelectionModel::ClearAndSelect
                                                    | QItemSelectionModel::Rows);
        probes.reset();
        applyBuyoutCommand(fixture, Buyout::BUYOUT_TYPE_INHERIT, Currency::CURRENCY_NONE, "");
        QCOMPARE(probes.model_updates, 1);
        QCOMPARE(probes.key_builds, 0);
        QCOMPARE(bucketItemNames(*model, bucket),
                 QStringList({"Alpha Sword", "Bravo Sword", "Charlie Sword"}));

        // Date symmetric: descending puts the freshly priced item (the only
        // valid last_update) first at command end.
        tree->header()->setSortIndicator(2, Qt::DescendingOrder);
        QCOMPARE(bucketItemNames(*model, bucket),
                 QStringList({"Charlie Sword", "Bravo Sword", "Alpha Sword"}));
        tree->selectionModel()->setCurrentIndex(findItemRow(*model, bucket, "Bravo Sword"),
                                                QItemSelectionModel::ClearAndSelect
                                                    | QItemSelectionModel::Rows);
        probes.reset();
        applyBuyoutCommand(fixture, Buyout::BUYOUT_TYPE_BUYOUT, Currency::CURRENCY_CHAOS_ORB, "5");
        QCOMPARE(probes.model_updates, 1);
        QCOMPARE(probes.key_builds, 0); // the Date keys rebuilt entries, not the vector
        QCOMPARE(bucketItemNames(*model, bucket),
                 QStringList({"Bravo Sword", "Charlie Sword", "Alpha Sword"}));
        probes.enabled = false;
    }

    // Migration (MigrateItem via ItemsManager::MigrateBuyouts): reorders
    // within the snapshot's outer batch — one model update for the whole
    // snapshot, no command boundary involved.
    {
        MainWindowFixture fixture;
        auto *tree = fixture.window->findChild<QTreeView *>("treeView");
        QVERIFY(tree);

        const ItemLocation tabA = makeTestStashLocation("stash-alpha", "Alpha Tab", 0);
        Items items;
        const auto alpha = makeMainWindowItem("item-alpha", "Alpha", "Sword", tabA);
        items.push_back(makeMainWindowItem("item-zed", "Zed", "Sword", tabA));
        items.push_back(alpha);

        // Seed a buyout under Alpha's v4 hash: the fresh datastore's
        // db_version drives MigrateBuyouts to rekey it to the item id
        // during the snapshot's pricing sequence.
        fixture.buyoutFixture.manager->Set(*alpha, makeChaosBuyout(7));
        fixture.buyoutFixture.manager->MigrateItem(alpha->id(), alpha->hash_v4());
        QVERIFY(fixture.buyoutFixture.manager->Get(*alpha).IsNull());

        tree->header()->setSortIndicator(1, Qt::AscendingOrder);
        auto &probes = ModelProbes::instance();
        probes.reset();
        probes.enabled = true;
        fixture.itemsManager->OnItemsRefreshed(items, {tabA}, false);
        QCOMPARE(probes.model_updates, 1);
        probes.enabled = false;

        QVERIFY(fixture.buyoutFixture.manager->Get(*alpha) == makeChaosBuyout(7));
        const QModelIndex bucket = findBucket(*tree->model(), tabA.GetHeader());
        QVERIFY(bucket.isValid());
        tree->expand(bucket);
        // The migrated price is in the displayed order: unpriced Zed sorts
        // before Alpha's 7c under Price ascending.
        QCOMPARE(bucketItemNames(*tree->model(), bucket), QStringList({"Zed Sword", "Alpha Sword"}));
    }
}

// M3 R2-5: with Price active in By-Item, one buyout command over a
// many-row selection produces exactly one reorder / model update at
// command end — the trailing PropagateTabBuyouts pass included (R3-3) —
// never one per selected row.
void MainWindowTest::multiSelectionBuyoutEditReordersOnce()
{
    MainWindowFixture fixture;
    auto *tree = fixture.window->findChild<QTreeView *>("treeView");
    auto *viewCombo = fixture.window->findChild<QComboBox *>("viewComboBox");
    QVERIFY(tree);
    QVERIFY(viewCombo);

    const ItemLocation tabA = makeTestStashLocation("stash-alpha", "Alpha Tab", 0);
    const ItemLocation tabB = makeTestStashLocation("stash-beta", "Beta Tab", 1);
    Items items;
    items.push_back(makeMainWindowItem("item-a", "Alpha", "Sword", tabA));
    items.push_back(makeMainWindowItem("item-b", "Bravo", "Sword", tabA));
    items.push_back(makeMainWindowItem("item-c", "Charlie", "Sword", tabB));
    items.push_back(makeMainWindowItem("item-d", "Delta", "Sword", tabB));
    fixture.itemsManager->OnItemsRefreshed(items, {tabA, tabB}, false);

    tree->header()->setSortIndicator(1, Qt::AscendingOrder);
    viewCombo->setCurrentIndex(1);
    emit viewCombo->activated(1);

    auto *model = tree->model();
    const QModelIndex flatBucket = model->index(0, 0);
    QVERIFY(flatBucket.isValid());
    QCOMPARE(bucketItemNames(*model, flatBucket),
             QStringList({"Alpha Sword", "Bravo Sword", "Charlie Sword", "Delta Sword"}));

    // Select the first three rows; Delta stays unpriced.
    tree->selectionModel()->setCurrentIndex(model->index(0, 0, flatBucket),
                                            QItemSelectionModel::ClearAndSelect
                                                | QItemSelectionModel::Rows);
    tree->selectionModel()->select(model->index(1, 0, flatBucket),
                                   QItemSelectionModel::Select | QItemSelectionModel::Rows);
    tree->selectionModel()->select(model->index(2, 0, flatBucket),
                                   QItemSelectionModel::Select | QItemSelectionModel::Rows);
    QCOMPARE(tree->selectionModel()->selectedRows().size(), 3);

    auto &probes = ModelProbes::instance();
    probes.reset();
    probes.enabled = true;
    QSignalSpy layouts(model, &QAbstractItemModel::layoutChanged);
    applyBuyoutCommand(fixture, Buyout::BUYOUT_TYPE_BUYOUT, Currency::CURRENCY_CHAOS_ORB, "5");
    QCOMPARE(probes.model_updates, 1);
    QCOMPARE(probes.bucket_sorts, 1);
    QCOMPARE(layouts.count(), 1);
    probes.enabled = false;

    QCOMPARE(bucketItemNames(*model, flatBucket),
             QStringList({"Delta Sword", "Alpha Sword", "Bravo Sword", "Charlie Sword"}));
}

// M3 R1-6: a scoped (per-delta) or final pricing pass touching many items
// produces at most one reorder / model update on the active search, never
// one per Set.
void MainWindowTest::pricingPassYieldsSingleModelUpdate()
{
    MainWindowFixture fixture;
    auto *tree = fixture.window->findChild<QTreeView *>("treeView");
    QVERIFY(tree);

    const ItemLocation tabA = makeTestStashLocation("stash-alpha", "Alpha Tab", 0);
    Items items;
    items.push_back(makeMainWindowItem("item-a", "Alpha", "Sword", tabA));
    items.push_back(makeMainWindowItem("item-b", "Bravo", "Sword", tabA));
    items.push_back(makeMainWindowItem("item-c", "Charlie", "Sword", tabA));
    items.push_back(makeMainWindowItem("item-d", "Delta", "Sword", tabA));
    items.push_back(makeMainWindowItem("item-e", "Echo", "Sword", tabA));
    fixture.itemsManager->OnItemsRefreshed(items, {tabA}, false);
    tree->header()->setSortIndicator(1, Qt::AscendingOrder);

    auto &probes = ModelProbes::instance();
    probes.reset();
    probes.enabled = true;

    // The scoped pass: a delta whose notes price three items is one batch.
    Items delta;
    delta.push_back(makeMainWindowItemWithNote("item-a", "Alpha", "Sword", tabA, "~b/o 5 chaos"));
    delta.push_back(makeMainWindowItemWithNote("item-b", "Bravo", "Sword", tabA, "~b/o 3 chaos"));
    delta.push_back(makeMainWindowItemWithNote("item-c", "Charlie", "Sword", tabA, "~b/o 9 chaos"));
    delta.push_back(makeMainWindowItem("item-d", "Delta", "Sword", tabA));
    delta.push_back(makeMainWindowItem("item-e", "Echo", "Sword", tabA));
    probes.reset();
    fixture.itemsManager->OnTabRefreshed(tabA, delta);
    QCOMPARE(probes.model_updates, 1);

    // The final propagation pass alone: two inherited prices, one batch.
    fixture.buyoutFixture.manager->SetTab(tabA, makeChaosBuyout(2));
    probes.reset();
    fixture.itemsManager->PropagateTabBuyouts();
    QCOMPARE(probes.model_updates, 1);
    probes.enabled = false;
}

// M3 R3-4: one snapshot's MigrateBuyouts -> ApplyAutoTabBuyouts ->
// ApplyAutoItemBuyouts -> PropagateTabBuyouts sequence produces at most
// one reorder / model update on the active search, never one per pass;
// buyout persistence writes still occur.
void MainWindowTest::snapshotPricingSequenceEmitsOneModelBatch()
{
    MainWindowFixture fixture;
    auto *tree = fixture.window->findChild<QTreeView *>("treeView");
    QVERIFY(tree);

    // The tab's label auto-prices the tab, Alpha's note auto-prices the
    // item, and Bravo inherits the tab price: three distinct passes mutate
    // buyout state in this one snapshot.
    const ItemLocation tabA = makeTestStashLocation("stash-alpha", "~b/o 10 chaos", 0);
    Items items;
    items.push_back(makeMainWindowItemWithNote("item-a", "Alpha", "Sword", tabA, "~b/o 5 chaos"));
    items.push_back(makeMainWindowItem("item-b", "Bravo", "Sword", tabA));

    tree->header()->setSortIndicator(1, Qt::AscendingOrder);
    auto &probes = ModelProbes::instance();
    probes.reset();
    probes.enabled = true;
    fixture.itemsManager->OnItemsRefreshed(items, {tabA}, false);
    QCOMPARE(probes.model_updates, 1);
    probes.enabled = false;

    // Persistence is outside batching and unchanged: the pricing writes
    // reached the repo.
    const auto itemBuyouts = fixture.buyoutFixture.repo->getItemBuyouts();
    QVERIFY(itemBuyouts.contains("item-a"));
    const auto tabBuyouts = fixture.buyoutFixture.repo->getLocationBuyouts();
    QVERIFY(tabBuyouts.contains("stash-alpha"));
}

// M3 R1-6, rule 5: with Name active, a buyout batch emits dataChanged for
// the affected visible Price/Date cells and performs no reordering — cell
// repaint is independent of the active sort column.
void MainWindowTest::priceCellsRepaintUnderAnySortColumn()
{
    MainWindowFixture fixture;
    auto *tree = fixture.window->findChild<QTreeView *>("treeView");
    QVERIFY(tree);

    const ItemLocation tabA = makeTestStashLocation("stash-alpha", "Alpha Tab", 0);
    Items items;
    items.push_back(makeMainWindowItem("item-a", "Alpha", "Sword", tabA));
    items.push_back(makeMainWindowItem("item-b", "Bravo", "Sword", tabA));
    items.push_back(makeMainWindowItem("item-c", "Charlie", "Sword", tabA));
    fixture.itemsManager->OnItemsRefreshed(items, {tabA}, false);

    // Name (column 0) stays the active sort column.
    auto *model = tree->model();
    const QModelIndex bucket = findBucket(*model, tabA.GetHeader());
    QVERIFY(bucket.isValid());
    tree->expand(bucket);
    const QStringList namesBefore = bucketItemNames(*model, bucket);
    const QModelIndex target = findItemRow(*model, bucket, "Bravo Sword");
    QVERIFY(target.isValid());
    tree->selectionModel()->setCurrentIndex(target,
                                            QItemSelectionModel::ClearAndSelect
                                                | QItemSelectionModel::Rows);

    auto &probes = ModelProbes::instance();
    probes.reset();
    probes.enabled = true;
    QSignalSpy repaints(model, &QAbstractItemModel::dataChanged);
    QSignalSpy layouts(model, &QAbstractItemModel::layoutChanged);
    applyBuyoutCommand(fixture, Buyout::BUYOUT_TYPE_BUYOUT, Currency::CURRENCY_CHAOS_ORB, "7");

    // The batch emitted once, repainted the affected Price/Date cells, and
    // reordered nothing.
    QCOMPARE(probes.model_updates, 1);
    QCOMPARE(probes.bucket_sorts, 0);
    QCOMPARE(layouts.count(), 0);
    probes.enabled = false;
    QCOMPARE(bucketItemNames(*model, bucket), namesBefore);

    QCOMPARE(repaints.count(), 1);
    const QModelIndex topLeft = repaints.at(0).at(0).toModelIndex();
    const QModelIndex bottomRight = repaints.at(0).at(1).toModelIndex();
    QCOMPARE(topLeft.parent(), bucket);
    QVERIFY(topLeft.row() <= target.row());
    QVERIFY(bottomRight.row() >= target.row());
    // The span covers the Price (1) and Date (2) columns.
    QVERIFY(topLeft.column() <= 1);
    QVERIFY(bottomRight.column() >= 2);
}

// M3 S2, review round 1: the rule-5 repaint must reach EVERY visible
// occurrence of an affected id. The R6-3 id index keeps only the first
// occurrence of a duplicated id (mid-refresh divergence can show one id in
// two tabs) and holds no entry for id-less items, so affected ids the
// index cannot fully represent fall back to an every-bucket scan.
void MainWindowTest::buyoutRepaintCoversEveryVisibleOccurrence()
{
    MainWindowFixture fixture;
    auto *tree = fixture.window->findChild<QTreeView *>("treeView");
    QVERIFY(tree);

    const ItemLocation tabA = makeTestStashLocation("stash-alpha", "Alpha Tab", 0);
    const ItemLocation tabB = makeTestStashLocation("stash-beta", "Beta Tab", 1);
    Items items;
    items.push_back(makeMainWindowItem("item-x", "Mover", "Sword", tabA));
    items.push_back(makeMainWindowItem("item-x", "MoverTwo", "Sword", tabB));
    items.push_back(makeMainWindowItem("", "Ghost", "Sword", tabB));
    items.push_back(makeMainWindowItem("item-d", "Delta", "Sword", tabA));
    fixture.itemsManager->OnItemsRefreshed(items, {tabA, tabB}, false);

    // Name stays the active sort column: pure repaint, no reordering.
    auto *model = tree->model();
    const QModelIndex bucketA = findBucket(*model, tabA.GetHeader());
    const QModelIndex bucketB = findBucket(*model, tabB.GetHeader());
    QVERIFY(bucketA.isValid());
    QVERIFY(bucketB.isValid());
    tree->expand(bucketA);
    tree->expand(bucketB);

    // Editing the duplicated id through its tab-A occurrence repaints BOTH
    // occurrences — one span per affected bucket, each covering exactly
    // the affected row (the unaffected Delta and Ghost rows stay outside).
    const QModelIndex moverA = findItemRow(*model, bucketA, "Mover Sword");
    const QModelIndex moverB = findItemRow(*model, bucketB, "MoverTwo Sword");
    QVERIFY(moverA.isValid());
    QVERIFY(moverB.isValid());
    tree->selectionModel()->setCurrentIndex(moverA,
                                            QItemSelectionModel::ClearAndSelect
                                                | QItemSelectionModel::Rows);
    QSignalSpy repaints(model, &QAbstractItemModel::dataChanged);
    applyBuyoutCommand(fixture, Buyout::BUYOUT_TYPE_BUYOUT, Currency::CURRENCY_CHAOS_ORB, "7");
    QCOMPARE(repaints.count(), 2);
    for (int n = 0; n < repaints.count(); ++n) {
        const QModelIndex topLeft = repaints.at(n).at(0).toModelIndex();
        const QModelIndex bottomRight = repaints.at(n).at(1).toModelIndex();
        const QModelIndex expected = (topLeft.parent() == bucketA) ? moverA : moverB;
        QCOMPARE(topLeft.parent(), expected.parent());
        QCOMPARE(topLeft.row(), expected.row());
        QCOMPARE(bottomRight.row(), expected.row());
    }

    // Editing the id-less item repaints its row through the same fallback:
    // the empty id has no index entry, only the scan can find it.
    const QModelIndex ghost = findItemRow(*model, bucketB, "Ghost Sword");
    QVERIFY(ghost.isValid());
    tree->selectionModel()->setCurrentIndex(ghost,
                                            QItemSelectionModel::ClearAndSelect
                                                | QItemSelectionModel::Rows);
    repaints.clear();
    applyBuyoutCommand(fixture, Buyout::BUYOUT_TYPE_BUYOUT, Currency::CURRENCY_CHAOS_ORB, "3");
    QCOMPARE(repaints.count(), 1);
    const QModelIndex topLeft = repaints.at(0).at(0).toModelIndex();
    const QModelIndex bottomRight = repaints.at(0).at(1).toModelIndex();
    QCOMPARE(topLeft.parent(), bucketB);
    QCOMPARE(topLeft.row(), ghost.row());
    QCOMPARE(bottomRight.row(), ghost.row());
}

// M3 D2 (S3): an unfiltered By-Tab refilter sorts no collapsed bucket and
// builds no keys for one; expanding one bucket sorts exactly that bucket,
// correctly. Unsorted is not unstable — a collapsed bucket keeps arrival
// order until something changes its contents.
void MainWindowTest::collapsedBucketsDeferSorting()
{
    MainWindowFixture fixture;
    auto *tree = fixture.window->findChild<QTreeView *>("treeView");
    QVERIFY(tree);

    const ItemLocation tabA = makeTestStashLocation("stash-alpha", "Alpha Tab", 0);
    const ItemLocation tabB = makeTestStashLocation("stash-beta", "Beta Tab", 1);
    Items items;
    items.push_back(makeMainWindowItem("item-a", "Alpha", "Sword", tabA));
    items.push_back(makeMainWindowItem("item-c", "Charlie", "Sword", tabA));
    items.push_back(makeMainWindowItem("item-b", "Bravo", "Sword", tabA));
    items.push_back(makeMainWindowItem("item-e", "Echo", "Sword", tabB));
    items.push_back(makeMainWindowItem("item-d", "Delta", "Sword", tabB));
    fixture.itemsManager->OnItemsRefreshed(items, {tabA, tabB}, false);

    auto &probes = ModelProbes::instance();
    probes.reset();
    probes.enabled = true;

    // The user refilter of the default-collapsed unfiltered view: the
    // whole sort/key toll disappears, not merely cheapens.
    fixture.window->OnSearchFormChange();
    QCOMPARE(probes.refilters, 1);
    QCOMPARE(probes.bucket_sorts, 0);
    QCOMPARE(probes.key_builds, 0);
    QCOMPARE(probes.keyed_compares, 0);

    // Arrival order, deterministically, until expansion sorts.
    auto *model = tree->model();
    const QModelIndex bucketA = findBucket(*model, tabA.GetHeader());
    QVERIFY(bucketA.isValid());
    QCOMPARE(bucketItemNames(*model, bucketA),
             QStringList({"Alpha Sword", "Charlie Sword", "Bravo Sword"}));

    // Expanding sorts exactly the expanded bucket (default indicator:
    // Name descending), building exactly its keys.
    tree->expand(bucketA);
    QCOMPARE(probes.bucket_sorts, 1);
    QCOMPARE(probes.bucket_sorts_by_location[LocationInventory::KeyFor(tabA)], 1);
    QCOMPARE(probes.bucket_sorts_by_location[LocationInventory::KeyFor(tabB)], 0);
    QCOMPARE(probes.key_builds, 1);
    QCOMPARE(probes.key_builds_by_location[LocationInventory::KeyFor(tabB)], 0);
    probes.enabled = false;
    QCOMPARE(bucketItemNames(*model, bucketA),
             QStringList({"Charlie Sword", "Bravo Sword", "Alpha Sword"}));
}

// M3 D2 (S3): a user refilter with N saved expansions sorts exactly those
// N buckets on restore — the restore's expand signals find the fresh
// buckets' flags invalid and sort each, and nothing sorts the rest.
void MainWindowTest::restoredExpansionSortsRestoredBucketsOnly()
{
    MainWindowFixture fixture;
    auto *tree = fixture.window->findChild<QTreeView *>("treeView");
    QVERIFY(tree);

    const ItemLocation tabA = makeTestStashLocation("stash-alpha", "Alpha Tab", 0);
    const ItemLocation tabB = makeTestStashLocation("stash-beta", "Beta Tab", 1);
    const ItemLocation tabC = makeTestStashLocation("stash-gamma", "Gamma Tab", 2);
    Items items;
    items.push_back(makeMainWindowItem("item-a", "Alpha", "Sword", tabA));
    items.push_back(makeMainWindowItem("item-b", "Bravo", "Sword", tabA));
    items.push_back(makeMainWindowItem("item-d", "Delta", "Sword", tabB));
    items.push_back(makeMainWindowItem("item-c", "Charlie", "Sword", tabB));
    items.push_back(makeMainWindowItem("item-e", "Echo", "Sword", tabC));
    items.push_back(makeMainWindowItem("item-f", "Foxtrot", "Sword", tabC));
    fixture.itemsManager->OnItemsRefreshed(items, {tabA, tabB, tabC}, false);

    auto *model = tree->model();
    tree->expand(findBucket(*model, tabA.GetHeader()));
    tree->expand(findBucket(*model, tabC.GetHeader()));

    auto &probes = ModelProbes::instance();
    probes.reset();
    probes.enabled = true;
    fixture.window->OnSearchFormChange();
    QCOMPARE(probes.refilters, 1);
    QCOMPARE(probes.bucket_sorts, 2);
    QCOMPARE(probes.bucket_sorts_by_location[LocationInventory::KeyFor(tabA)], 1);
    QCOMPARE(probes.bucket_sorts_by_location[LocationInventory::KeyFor(tabB)], 0);
    QCOMPARE(probes.bucket_sorts_by_location[LocationInventory::KeyFor(tabC)], 1);
    QCOMPARE(probes.key_builds, 2);
    probes.enabled = false;

    // The restored buckets are sorted; the collapsed one keeps arrival
    // order.
    QCOMPARE(bucketItemNames(*model, findBucket(*model, tabA.GetHeader())),
             QStringList({"Bravo Sword", "Alpha Sword"}));
    QCOMPARE(bucketItemNames(*model, findBucket(*model, tabB.GetHeader())),
             QStringList({"Delta Sword", "Charlie Sword"}));
}

// M3 D2 rule 5 (S3): a filtered search is default-expanded, so every
// visible bucket of the result presents sorted — established eagerly in
// the refilter's one view-wide pass, not per expand signal.
void MainWindowTest::filteredSearchSortsAllVisibleBuckets()
{
    MainWindowFixture fixture;
    auto *tree = fixture.window->findChild<QTreeView *>("treeView");
    auto *name = findNameFilter(*fixture.window);
    QVERIFY(tree);
    QVERIFY(name);

    const ItemLocation tabA = makeTestStashLocation("stash-alpha", "Alpha Tab", 0);
    const ItemLocation tabB = makeTestStashLocation("stash-beta", "Beta Tab", 1);
    Items items;
    items.push_back(makeMainWindowItem("item-a", "Alpha", "Sword", tabA));
    items.push_back(makeMainWindowItem("item-c", "Charlie", "Sword", tabA));
    items.push_back(makeMainWindowItem("item-z", "Zulu", "Axe", tabA));
    items.push_back(makeMainWindowItem("item-e", "Echo", "Sword", tabB));
    items.push_back(makeMainWindowItem("item-d", "Delta", "Sword", tabB));
    fixture.itemsManager->OnItemsRefreshed(items, {tabA, tabB}, false);

    name->setFocus();
    QTest::keyClicks(name, "sword");
    auto &probes = ModelProbes::instance();
    probes.reset();
    probes.enabled = true;
    fixture.window->OnSearchFormChange();
    QCOMPARE(probes.refilters, 1);
    QCOMPARE(probes.bucket_sorts, 2);
    QCOMPARE(probes.key_builds, 2);
    probes.enabled = false;

    auto *model = tree->model();
    const QModelIndex bucketA = findBucket(*model, tabA.GetHeader());
    const QModelIndex bucketB = findBucket(*model, tabB.GetHeader());
    QVERIFY(bucketA.isValid());
    QVERIFY(bucketB.isValid());
    QVERIFY(tree->isExpanded(bucketA));
    QVERIFY(tree->isExpanded(bucketB));
    QCOMPARE(bucketItemNames(*model, bucketA), QStringList({"Charlie Sword", "Alpha Sword"}));
    QCOMPARE(bucketItemNames(*model, bucketB), QStringList({"Echo Sword", "Delta Sword"}));
}

// M3 R1-5 (S3): expand (sort), collapse, re-expand with no intervening
// invalidation — no key build and no sort runs the second time, and the
// order is the sorted one. Arrival order is never reconstructed; collapse
// is a view event, not a model event.
void MainWindowTest::sortedOrderSurvivesCollapse()
{
    MainWindowFixture fixture;
    auto *tree = fixture.window->findChild<QTreeView *>("treeView");
    QVERIFY(tree);

    const ItemLocation tabA = makeTestStashLocation("stash-alpha", "Alpha Tab", 0);
    Items items;
    items.push_back(makeMainWindowItem("item-a", "Alpha", "Sword", tabA));
    items.push_back(makeMainWindowItem("item-c", "Charlie", "Sword", tabA));
    items.push_back(makeMainWindowItem("item-b", "Bravo", "Sword", tabA));
    fixture.itemsManager->OnItemsRefreshed(items, {tabA}, false);

    auto *model = tree->model();
    const QModelIndex bucketA = findBucket(*model, tabA.GetHeader());
    QVERIFY(bucketA.isValid());
    tree->expand(bucketA);
    QCOMPARE(bucketItemNames(*model, bucketA),
             QStringList({"Charlie Sword", "Bravo Sword", "Alpha Sword"}));

    auto &probes = ModelProbes::instance();
    probes.reset();
    probes.enabled = true;
    tree->collapse(bucketA);
    tree->expand(bucketA);
    QCOMPARE(probes.bucket_sorts, 0);
    QCOMPARE(probes.key_builds, 0);
    QCOMPARE(probes.keyed_compares, 0);
    probes.enabled = false;
    QCOMPARE(bucketItemNames(*model, bucketA),
             QStringList({"Charlie Sword", "Bravo Sword", "Alpha Sword"}));
}

// M3 R2-3 (S3): an expanded bucket's keys persist across re-sorts and
// direction flips (no key rebuild on a flip); collapsing evicts its keys
// (live key bytes drop) while its order and flag persist.
void MainWindowTest::keyResidencyFollowsMaterialization()
{
    MainWindowFixture fixture;
    auto *tree = fixture.window->findChild<QTreeView *>("treeView");
    QVERIFY(tree);

    const ItemLocation tabA = makeTestStashLocation("stash-alpha", "Alpha Tab", 0);
    Items items;
    items.push_back(makeMainWindowItem("item-a", "Alpha", "Sword", tabA));
    items.push_back(makeMainWindowItem("item-c", "Charlie", "Sword", tabA));
    items.push_back(makeMainWindowItem("item-b", "Bravo", "Sword", tabA));
    fixture.itemsManager->OnItemsRefreshed(items, {tabA}, false);

    auto &probes = ModelProbes::instance();
    const std::int64_t baseline = probes.live_key_bytes;

    auto *model = tree->model();
    const QModelIndex bucketA = findBucket(*model, tabA.GetHeader());
    QVERIFY(bucketA.isValid());
    tree->expand(bucketA);
    const std::int64_t resident = probes.live_key_bytes;
    QVERIFY(resident > baseline);

    // The flip re-sorts on the resident keys: no rebuild, bytes steady.
    probes.reset();
    probes.enabled = true;
    tree->header()->setSortIndicator(0, Qt::AscendingOrder);
    QCOMPARE(probes.bucket_sorts, 1);
    QCOMPARE(probes.key_builds, 0);
    QCOMPARE(probes.live_key_bytes, resident);
    QCOMPARE(bucketItemNames(*model, bucketA),
             QStringList({"Alpha Sword", "Bravo Sword", "Charlie Sword"}));

    // Collapse evicts the keys and nothing else: the sorted order and
    // flag persist, so re-expansion does no work and shows the ascending
    // order.
    tree->collapse(bucketA);
    QCOMPARE(probes.live_key_bytes, baseline);
    tree->expand(bucketA);
    QCOMPARE(probes.bucket_sorts, 1);
    QCOMPARE(probes.key_builds, 0);
    probes.enabled = false;
    QCOMPARE(bucketItemNames(*model, bucketA),
             QStringList({"Alpha Sword", "Bravo Sword", "Charlie Sword"}));
}

// M3 R3-1 (S3): expand (keys built), collapse (keys evicted), re-expand
// (valid flag: no sort, no key build — sorted-but-keyless): a direction
// flip then hydrates the bucket's keys exactly once and re-sorts
// correctly; a second flip rebuilds nothing.
void MainWindowTest::reexpandedBucketFlipHydratesOnce()
{
    MainWindowFixture fixture;
    auto *tree = fixture.window->findChild<QTreeView *>("treeView");
    QVERIFY(tree);

    const ItemLocation tabA = makeTestStashLocation("stash-alpha", "Alpha Tab", 0);
    Items items;
    items.push_back(makeMainWindowItem("item-a", "Alpha", "Sword", tabA));
    items.push_back(makeMainWindowItem("item-c", "Charlie", "Sword", tabA));
    items.push_back(makeMainWindowItem("item-b", "Bravo", "Sword", tabA));
    fixture.itemsManager->OnItemsRefreshed(items, {tabA}, false);

    auto *model = tree->model();
    const QModelIndex bucketA = findBucket(*model, tabA.GetHeader());
    QVERIFY(bucketA.isValid());
    tree->expand(bucketA);
    tree->collapse(bucketA);

    auto &probes = ModelProbes::instance();
    probes.reset();
    probes.enabled = true;
    tree->expand(bucketA);
    QCOMPARE(probes.bucket_sorts, 0);
    QCOMPARE(probes.key_builds, 0);

    // The flip is the key-consuming event: one hydration, one sort.
    tree->header()->setSortIndicator(0, Qt::AscendingOrder);
    QCOMPARE(probes.bucket_sorts, 1);
    QCOMPARE(probes.key_builds, 1);
    QCOMPARE(bucketItemNames(*model, bucketA),
             QStringList({"Alpha Sword", "Bravo Sword", "Charlie Sword"}));

    // The second flip reuses the now-resident keys.
    tree->header()->setSortIndicator(0, Qt::DescendingOrder);
    QCOMPARE(probes.bucket_sorts, 2);
    QCOMPARE(probes.key_builds, 1);
    probes.enabled = false;
    QCOMPARE(bucketItemNames(*model, bucketA),
             QStringList({"Charlie Sword", "Bravo Sword", "Alpha Sword"}));
}

// M3 R1-5/R2-3 (S3): switching the active column discards resident keys,
// clears every sorted flag, and re-sorts materialized buckets only (their
// keys rebuild — the old column's keys cannot order the new one); a
// direction flip after the switch re-sorts on the resident keys with no
// rebuild; collapsed buckets sort at their next expansion.
void MainWindowTest::sortColumnSwitchResortsVisibleBucketsOnly()
{
    MainWindowFixture fixture;
    auto *tree = fixture.window->findChild<QTreeView *>("treeView");
    QVERIFY(tree);

    const ItemLocation tabA = makeTestStashLocation("stash-alpha", "Alpha Tab", 0);
    const ItemLocation tabB = makeTestStashLocation("stash-beta", "Beta Tab", 1);
    const ItemLocation tabC = makeTestStashLocation("stash-gamma", "Gamma Tab", 2);
    Items items;
    items.push_back(makeMainWindowItem("item-a", "Alpha", "Sword", tabA));
    items.push_back(makeMainWindowItem("item-b", "Bravo", "Sword", tabA));
    items.push_back(makeMainWindowItem("item-c", "Charlie", "Sword", tabB));
    items.push_back(makeMainWindowItem("item-d", "Delta", "Sword", tabB));
    items.push_back(makeMainWindowItem("item-f", "Foxtrot", "Sword", tabC));
    items.push_back(makeMainWindowItem("item-e", "Echo", "Sword", tabC));
    fixture.itemsManager->OnItemsRefreshed(items, {tabA, tabB, tabC}, false);

    auto *model = tree->model();
    tree->expand(findBucket(*model, tabA.GetHeader()));
    tree->expand(findBucket(*model, tabB.GetHeader()));

    // The switch (Name -> Quality): the materialized set rebuilds and
    // re-sorts; the collapsed bucket is untouched.
    auto &probes = ModelProbes::instance();
    probes.reset();
    probes.enabled = true;
    tree->header()->setSortIndicator(3, Qt::AscendingOrder);
    QCOMPARE(probes.bucket_sorts, 2);
    QCOMPARE(probes.key_builds, 2);
    QCOMPARE(probes.bucket_sorts_by_location[LocationInventory::KeyFor(tabC)], 0);
    QCOMPARE(probes.key_builds_by_location[LocationInventory::KeyFor(tabC)], 0);

    // The flip after the switch reuses the fresh resident keys.
    tree->header()->setSortIndicator(3, Qt::DescendingOrder);
    QCOMPARE(probes.bucket_sorts, 4);
    QCOMPARE(probes.key_builds, 2);

    // The collapsed bucket pays at its next expansion, under the new
    // column, exactly once.
    tree->expand(findBucket(*model, tabC.GetHeader()));
    QCOMPARE(probes.bucket_sorts, 5);
    QCOMPARE(probes.key_builds, 3);
    QCOMPARE(probes.bucket_sorts_by_location[LocationInventory::KeyFor(tabC)], 1);
    QCOMPARE(probes.key_builds_by_location[LocationInventory::KeyFor(tabC)], 1);
    probes.enabled = false;
}

// M3 R2-4 (S3, PARTIAL — fully closes in S5): with several searches,
// aggregate resident key memory never exceeds one search's worth;
// deactivating a search evicts its keys, and no refilter runs when the
// search is not dirty. Reactivation rehydrates a clean By-Tab search
// lazily, per bucket. The clean-By-Item eager-hydration clause (R3-1)
// lands with D4's eager activation in S5.
void MainWindowTest::residentKeysScopedToActiveSearch()
{
    MainWindowFixture fixture;
    auto *tabs = findSearchTabs(*fixture.window);
    auto *tree = fixture.window->findChild<QTreeView *>("treeView");
    auto *viewCombo = fixture.window->findChild<QComboBox *>("viewComboBox");
    QVERIFY(tabs);
    QVERIFY(tree);
    QVERIFY(viewCombo);

    const ItemLocation tabA = makeTestStashLocation("stash-alpha", "Alpha Tab", 0);
    const ItemLocation tabB = makeTestStashLocation("stash-beta", "Beta Tab", 1);
    Items items;
    items.push_back(makeMainWindowItem("item-a", "Alpha", "Sword", tabA));
    items.push_back(makeMainWindowItem("item-b", "Bravo", "Sword", tabA));
    items.push_back(makeMainWindowItem("item-c", "Charlie", "Sword", tabB));
    items.push_back(makeMainWindowItem("item-d", "Delta", "Sword", tabB));
    fixture.itemsManager->OnItemsRefreshed(items, {tabA, tabB}, false);

    auto &probes = ModelProbes::instance();
    const std::int64_t none = probes.live_key_bytes;

    // Search 1 (By-Tab): one expanded bucket holds the only resident keys.
    tree->expand(findBucket(*tree->model(), tabA.GetHeader()));
    QVERIFY(probes.live_key_bytes > none);

    // Creating and switching to Search 2 deactivates Search 1: its keys
    // evict (background searches hold exactly 0), while its order, flags,
    // and expansion persist for reactivation.
    tabs->setCurrentIndex(1);
    QCOMPARE(probes.live_key_bytes, none);

    // Search 2 goes By-Item: the flat bucket's keys are the aggregate's
    // entire contents — one search's worth.
    viewCombo->setCurrentIndex(1);
    emit viewCombo->activated(1);
    const std::int64_t one_by_item = probes.live_key_bytes;
    QVERIFY(one_by_item > none);

    // A second By-Item search: the deactivation eviction keeps the
    // aggregate at exactly one search's worth — N background By-Item
    // searches hold none, not one flat bucket each.
    tabs->setCurrentIndex(2);
    QCOMPARE(probes.live_key_bytes, none);
    viewCombo->setCurrentIndex(1);
    emit viewCombo->activated(1);
    QCOMPARE(probes.live_key_bytes, one_by_item);

    // Reactivating clean Search 1: no refilter (the search is not
    // dirty), no eager hydration — the By-Tab expanded bucket stays
    // sorted-but-keyless until a key-consuming event.
    probes.reset();
    probes.enabled = true;
    tabs->setCurrentIndex(0);
    QCOMPARE(probes.refilters, 0);
    QCOMPARE(probes.bucket_sorts, 0);
    QCOMPARE(probes.key_builds, 0);
    QCOMPARE(probes.live_key_bytes, none);
    const QModelIndex bucketA = findBucket(*tree->model(), tabA.GetHeader());
    QVERIFY(bucketA.isValid());
    QVERIFY(tree->isExpanded(bucketA));
    QCOMPARE(bucketItemNames(*tree->model(), bucketA), QStringList({"Bravo Sword", "Alpha Sword"}));

    // The lazy rehydration, per bucket: the flip hydrates exactly the
    // expanded bucket.
    tree->header()->setSortIndicator(0, Qt::AscendingOrder);
    QCOMPARE(probes.key_builds, 1);
    QCOMPARE(probes.key_builds_by_location[LocationInventory::KeyFor(tabA)], 1);
    probes.enabled = false;
    QVERIFY(probes.live_key_bytes > none);
}

QTEST_MAIN(MainWindowTest)

#include "tst_mainwindow.moc"
