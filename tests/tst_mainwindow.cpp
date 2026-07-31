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

    // Items-pipeline M2, stage 3, post-S4: D6 stable-identity bucketing
    // and the surviving delta-consumer pins. Deltas apply immediately to
    // an active By-Tab search (M3 D3); the timer pins below are
    // fallback-scoped to the By-Item throttled seam through S4 and die
    // with the timer in S5 (supersession map).
    // `emptyDeltaMetadataLandsAtNextRefilter` was deleted in S4: R1-4
    // retired the M2 R7-2 exception it encoded — successor
    // `metadataDeltaAppliesWithoutItemIntersection`.
    void bucketsKeyOnStableIdDuringRefresh();
    void backgroundDeltaLeavesModelUntouched();
    void removalOnlyDeltaIntersects();
    void throttleDoesNotRearm();
    void tabSwitchBeforeTickPreservesDirty();
    void searchDeleteCancelsPendingTimer();
    void finalSnapshotCancelsPendingTick();
    void successfulRefilterCancelsPendingTick();
    void pendingTickSurvivesTerminalFailure();
    void childReconciliationIntersectsVisibleGhosts();

    // Items-pipeline M2, stage 5, post-S4: the R6-3 restore-fidelity
    // contract. The rename and replacement pins now assert the no-reset
    // delta path; scroll capture stays pinned to the By-Item fallback
    // seam through S4 (its D6 retarget lands in S5).
    // `reselectionSurvivesCrossTabMove` was deleted in S4 — its named
    // successor `selectionIntentSurvivesCrossTabMoveAcrossDeltas` lands
    // in the same commit window (supersession map; never red).
    void expansionSurvivesRenameByStableKey();
    void selectionSurvivesReplacementByStableIdentity();
    void scrollAndCaptureSurviveThrottledReset();

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
    void buyoutReorderScopedToAffectedMaterializedBuckets();

    // Items-pipeline M3, S4: D3's bucket-scoped delta operations, the
    // R1-4 metadata half, R1-7's renegotiated dirty flag (By-Tab half),
    // and the R1-3/R2-1 selection-intent contract.
    void unrelatedDeltaLeavesOtherBucketsUntouched();
    void deltaReplacesExactlyItsSourceRows();
    void childDeltaPreservesSiblingSourcesInParentBucket();
    void emptyDeltaEmptiesBucketWithoutRemovingIt();
    void deltaUpdatesVisibleIndexesIncrementally();
    void bucketRepositionsByMoveOnMetadataDelta();
    void metadataDeltaAppliesWithoutItemIntersection();
    void collapsedInvalidBucketResortsOnReexpand();
    void staleOrderNeverSurvivesDelta();
    void selectionIntentSurvivesCrossTabMoveAcrossDeltas();
    void selectionIntentClearsOnTerminalFailure();
    void appliedDeltasLeaveActiveSearchClean();

    // Items-pipeline M3, S4 review round 1. The first two are
    // fallback-scoped (the By-Item seam) and are renegotiated when the
    // seam dies in S5; the last two are permanent.
    void selectionIntentCoversByItemFallback();
    void modeSwitchConsumesFallbackDirtiness();
    void filteredSearchDropsEmptiedBucket();
    void metadataDeltaRefreshesSelectedPresentation();

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

// Fallback scoping for the surviving M2 timer pins (M3 S4, supersession
// map): after S4 only a By-Item active search still reaches the D9
// throttled path, so the pins covering that seam drive it explicitly.
// They die with the timer in S5.
static void switchToByItemView(MainWindowFixture &fixture)
{
    auto *viewCombo = fixture.window->findChild<QComboBox *>("viewComboBox");
    if (!viewCombo) {
        qFatal("switchToByItemView: viewComboBox not found");
    }
    viewCombo->setCurrentIndex(1);
    emit viewCombo->activated(1);
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

// M2 D9 rule 1, post-S4 (R1-7): a delta not intersecting the current
// search performs no model operation; background searches are marked
// items-dirty verbatim, while the current search — which processed the
// delta by correctly adjudicating "no visible change" — stays clean, so
// switching away and back triggers no spurious refilter.
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
    // operation on the current search, immediately or later.
    fixture.itemsManager
        ->OnTabRefreshed(tabB, {makeMainWindowItem("item-b2", "BetaItem Two", "Shield", tabB)});
    QCOMPARE(resets.count(), 0);
    QCOMPARE(visibleItemNames(*tree), QStringList({"AlphaItem Sword"}));

    // The background search was marked dirty (rule 1 verbatim) and
    // refilters on activation...
    tabs->setCurrentIndex(1);
    QVERIFY(visibleItemNames(*tree).contains("BetaItem Two Shield"));
    // ...but the current search processed the delta and stayed clean
    // (R1-7): switching back skips the refilter — the M2 behavior (dirty
    // → spurious full refilter) is explicitly renegotiated.
    QSignalSpy current_resets(current_model, &QAbstractItemModel::modelReset);
    tabs->setCurrentIndex(0);
    QCOMPARE(current_resets.count(), 0);
    QCOMPARE(visibleItemNames(*tree), QStringList({"AlphaItem Sword"}));
}

// M2 D9, post-S4: the removal half of the intersection test at the new
// grain — an empty delta whose fetch source has visible rows removes
// exactly those rows immediately, as row operations, never a reset. The
// intersection contract's metadata half (R1-4) rides the same delta: the
// location anchor renders even though the item half is empty.
void MainWindowTest::removalOnlyDeltaIntersects()
{
    MainWindowFixture fixture;
    auto *tree = fixture.window->findChild<QTreeView *>("treeView");
    QVERIFY(tree);

    const ItemLocation tabA = makeTestStashLocation("stash-aaaa", "Alpha", 0);
    Items items;
    items.push_back(makeMainWindowItem("item-a", "AlphaItem", "Sword", tabA));
    fixture.itemsManager->OnItemsRefreshed(items, {tabA}, false);
    QCOMPARE(visibleItemNames(*tree), QStringList({"AlphaItem Sword"}));

    QSignalSpy resets(tree->model(), &QAbstractItemModel::modelReset);
    QSignalSpy removals(tree->model(), &QAbstractItemModel::rowsRemoved);
    // The empty delta also carries a fresh anchor (renamed in place):
    // both halves of the intersection contract apply at once.
    fixture.itemsManager->OnTabRefreshed(makeTestStashLocation("stash-aaaa", "Alpha Renamed", 0),
                                         {});

    // The rows leave now, by row operations; the bucket stays (deletion
    // is a snapshot-boundary effect) and renders the fresh name.
    QCOMPARE(resets.count(), 0);
    QCOMPARE(removals.count(), 1);
    QCOMPARE(visibleItemNames(*tree), QStringList());
    QCOMPARE(tree->model()->rowCount(), 1);
    const ItemLocation renamed = makeTestStashLocation("stash-aaaa", "Alpha Renamed", 0);
    QVERIFY(findBucket(*tree->model(), renamed.GetHeader()).isValid());
}

// M2 D9 rule 2, fallback-scoped to the By-Item seam (M3 S4; dies with the
// timer in S5): a non-resetting trailing throttle — deltas arriving faster
// than S produce at most one refilter per S, and the first delta's
// deadline is not pushed back by later arrivals.
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
    switchToByItemView(fixture);

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

// M2 D9 rule 4, fallback-scoped to the By-Item seam (M3 S4; dies with the
// timer in S5): switching searches with a tick pending cancels the timer;
// the old search refilters on its next activation via its items-dirty
// flag — nothing is lost.
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
    switchToByItemView(fixture);
    // A second search to switch to.
    tabs->setCurrentIndex(1);
    tabs->setCurrentIndex(0);

    QAbstractItemModel *first_model = tree->model();
    QSignalSpy first_resets(first_model, &QAbstractItemModel::modelReset);

    // Scroll an ordinary item row to the top and remember it: the switch
    // away must capture this (R6-3) — the reactivation refilter resets the
    // model while the view still shows the OTHER search, so the capture at
    // switch-away is the only one there is.
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

// M2 D9 rule 4, fallback-scoped to the By-Item seam (M3 S4; dies with the
// timer in S5): deleting the current search with a tick pending fires
// nothing against the dead search.
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
    // Search 2 becomes current, By-Item (the throttled seam); arm its tick.
    tabs->setCurrentIndex(1);
    switchToByItemView(fixture);
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

// M2 D9 rule 5, fallback-scoped to the By-Item seam (M3 S4; dies with the
// timer in S5): the final snapshot cancels a pending tick and the full
// path clears all items-dirty flags.
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
    switchToByItemView(fixture);

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

// M2 D9/R5-5, fallback-scoped to the By-Item seam (M3 S4; dies with the
// timer in S5): a user-initiated or form-edit refilter of the current
// search with a tick pending cancels the timer and clears the flag; no
// redundant reset follows.
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
    switchToByItemView(fixture);

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

// M2 D9/R5-3, outcome (a), fallback-scoped to the By-Item seam (M3 S4;
// dies with the timer in S5): deltas followed by a terminal failure with a
// tick pending — the tick still fires and the view catches up within S
// plus one reset-plus-restore duration despite no final snapshot ever
// arriving.
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
    switchToByItemView(fixture);

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

// M2 D9/R5-2/R6-2, post-S4 (D3): a ChildrenReconciled whose expected set
// excludes visible ghost children applies immediately as row removals
// scoped to the parent's bucket — no timer, no reset — and the applied
// state persists through a terminal failure (no final snapshot).
void MainWindowTest::childReconciliationIntersectsVisibleGhosts()
{
    MainWindowFixture fixture;
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
    QSignalSpy removals(tree->model(), &QAbstractItemModel::rowsRemoved);

    // The reconciliation's expected set is the parent alone: the visible
    // ghost leaves now, by row removals scoped to the parent's bucket.
    fixture.itemsManager->OnChildrenReconciled(parent,
                                               {FetchSourceKey{ItemLocationType::STASH,
                                                               "stash-pppp"}});
    QCOMPARE(resets.count(), 0);
    QCOMPARE(removals.count(), 1);
    QCOMPARE(visibleItemNames(*tree), QStringList({"ParentItem Sword"}));

    // A terminal failure follows — no final snapshot. The applied state
    // persists; nothing resurrects the ghost.
    fixture.window->OnRefreshFinished(RefreshOutcome{FailedRefresh{RateLimit::FetchError{}}});
    QCOMPARE(resets.count(), 0);
    QCOMPARE(visibleItemNames(*tree), QStringList({"ParentItem Sword"}));
}

// M2 R6-3, post-S4 (D3/R1-4): expansion is keyed by the stable (type,
// id), and a rename delta now applies in place — dataChanged on the
// bucket row, no reset, no restore machinery — so the expanded tab keeps
// its expansion trivially and renders the new name immediately.
void MainWindowTest::expansionSurvivesRenameByStableKey()
{
    MainWindowFixture fixture;
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
    auto &probes = ModelProbes::instance();
    probes.reset();
    probes.enabled = true;
    fixture.itemsManager
        ->OnTabRefreshed(renamed, {makeMainWindowItem("item-a", "AlphaItem", "Sword", renamed)});

    // The bucket renders the fresh metadata immediately and is still
    // expanded; no reset ran and no restore machinery was needed (the
    // untouched state simply never moved).
    QCOMPARE(resets.count(), 0);
    QCOMPARE(probes.expansion_captures, 0);
    QCOMPARE(probes.expansion_restores, 0);
    probes.enabled = false;
    QVERIFY(!findBucket(*tree->model(), tabA.GetHeader()).isValid());
    const QModelIndex renamedBucket = findBucket(*tree->model(), renamed.GetHeader());
    QVERIFY(renamedBucket.isValid());
    QVERIFY(tree->isExpanded(renamedBucket));
}

// M2 R6-3, post-S4: a streamed replacement swaps the selected item's
// object for a new one with the same stable id, now applied as row
// operations. The selection follows the id through the intent machinery
// (R1-3 — this pin is re-expressed over it), and the details panel adopts
// the replacement object rather than rendering the dead one.
void MainWindowTest::selectionSurvivesReplacementByStableIdentity()
{
    MainWindowFixture fixture;
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
    // a different rendered name, applied immediately as row operations.
    QSignalSpy resets(tree->model(), &QAbstractItemModel::modelReset);
    fixture.itemsManager
        ->OnTabRefreshed(tabA,
                         {makeMainWindowItem("item-a", "AlphaItem Two", "Sword", tabA),
                          makeMainWindowItem("item-z", "ZuluItem", "Sword", tabA)});
    QCOMPARE(resets.count(), 0);

    // The selection followed the stable id to the replacement object.
    const QModelIndexList selectedRows = tree->selectionModel()->selectedRows();
    QCOMPARE(selectedRows.size(), 1);
    QCOMPARE(selectedRows.front().data().toString(), "AlphaItem Two Sword");

    // The details panel re-rendered from the adopted object (the deferred
    // update path a user selection takes).
    QTRY_COMPARE_WITH_TIMEOUT(locationLabel->text(), tabA.GetHeader(), 2000);
}

// M2 R6-3, fallback-scoped to the By-Item seam (M3 S4; deleted with the
// timer in S5, its capture/restore machinery retargeted to D6's
// user-refilter reset as scrollAndCaptureSurviveUserRefilter): the
// throttled reset captures scroll immediately before resetting and
// restores by top-row anchor; when the anchored row was removed, the raw
// scrollbar value is the fallback — the anchor's bucket header is never
// scrolled to the top in its place.
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
    switchToByItemView(fixture);
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

// M3 S3, review round 1: the buyout batch's layout operation matches its
// affected MATERIALIZED scope. A Price-active batch touching only a
// collapsed bucket performs no reorder and emits no layout signals — the
// invalidation is flag-only, and the deferred sort pays at expansion with
// the fresh buyout in the order. A batch touching one expanded bucket
// scopes the whole layout dance to that bucket and leaves other expanded
// buckets' sorts untouched.
void MainWindowTest::buyoutReorderScopedToAffectedMaterializedBuckets()
{
    MainWindowFixture fixture;
    auto *tree = fixture.window->findChild<QTreeView *>("treeView");
    QVERIFY(tree);

    const ItemLocation tabA = makeTestStashLocation("stash-alpha", "Alpha Tab", 0);
    const ItemLocation tabB = makeTestStashLocation("stash-beta", "Beta Tab", 1);
    const auto alpha = makeMainWindowItem("item-a", "Alpha", "Sword", tabA);
    const auto delta = makeMainWindowItem("item-d", "Delta", "Sword", tabB);
    Items items;
    items.push_back(alpha);
    items.push_back(makeMainWindowItem("item-b", "Bravo", "Sword", tabA));
    items.push_back(delta);
    items.push_back(makeMainWindowItem("item-e", "Echo", "Sword", tabB));
    fixture.itemsManager->OnItemsRefreshed(items, {tabA, tabB}, false);

    // Price ascending; only bucket A is materialized.
    tree->header()->setSortIndicator(1, Qt::AscendingOrder);
    auto *model = tree->model();
    const QModelIndex bucketA = findBucket(*model, tabA.GetHeader());
    QVERIFY(bucketA.isValid());
    tree->expand(bucketA);
    QCOMPARE(bucketItemNames(*model, bucketA), QStringList({"Alpha Sword", "Bravo Sword"}));

    auto &probes = ModelProbes::instance();
    probes.reset();
    probes.enabled = true;
    QSignalSpy layouts(model, &QAbstractItemModel::layoutChanged);

    // A direct manager mutation (the scoped pricing pass's shape) pricing
    // an item in collapsed B: one batch, cells repaint, but nothing
    // materialized is affected — no layout operation at all.
    fixture.buyoutFixture.manager->Set(*delta, makeChaosBuyout(5));
    QCOMPARE(probes.model_updates, 1);
    QCOMPARE(probes.bucket_sorts, 0);
    QCOMPARE(layouts.count(), 0);

    // The deferred sort pays at expansion, with the new price in the
    // order: Echo (unpriced, rank 0) now precedes Delta's 5c under Price
    // ascending — the tie-break order would have been Delta first.
    const QModelIndex bucketB = findBucket(*model, tabB.GetHeader());
    QVERIFY(bucketB.isValid());
    tree->expand(bucketB);
    QCOMPARE(probes.bucket_sorts, 1);
    QCOMPARE(bucketItemNames(*model, bucketB), QStringList({"Echo Sword", "Delta Sword"}));

    // Pricing an item in expanded A: the layout operation scopes to A —
    // one layoutChanged naming A's subtree, one sort, no key build (the
    // resident entries rebuilt, R3-2), and B's sort count is untouched.
    probes.reset();
    layouts.clear();
    fixture.buyoutFixture.manager->Set(*alpha, makeChaosBuyout(7));
    QCOMPARE(probes.model_updates, 1);
    QCOMPARE(probes.bucket_sorts, 1);
    QCOMPARE(probes.bucket_sorts_by_location[LocationInventory::KeyFor(tabA)], 1);
    QCOMPARE(probes.bucket_sorts_by_location[LocationInventory::KeyFor(tabB)], 0);
    QCOMPARE(probes.key_builds, 0);
    QCOMPARE(layouts.count(), 1);
    const auto layout_parents = layouts.at(0).at(0).value<QList<QPersistentModelIndex>>();
    QCOMPARE(layout_parents.size(), 1);
    QCOMPARE(QModelIndex(layout_parents.front()), bucketA);
    probes.enabled = false;
    QCOMPARE(bucketItemNames(*model, bucketA), QStringList({"Bravo Sword", "Alpha Sword"}));
}

// M3 S4, the milestone's success criterion as a test (D3): refresh one
// tab; every other bucket's expansion, selection, and persistent indexes
// are untouched, and no restore machinery runs at all.
void MainWindowTest::unrelatedDeltaLeavesOtherBucketsUntouched()
{
    MainWindowFixture fixture;
    auto *tree = fixture.window->findChild<QTreeView *>("treeView");
    QVERIFY(tree);

    const ItemLocation tabA = makeTestStashLocation("stash-alpha", "Alpha Tab", 0);
    const ItemLocation tabB = makeTestStashLocation("stash-beta", "Beta Tab", 1);
    Items items;
    items.push_back(makeMainWindowItem("item-a1", "AlphaOne", "Sword", tabA));
    items.push_back(makeMainWindowItem("item-a2", "AlphaTwo", "Sword", tabA));
    items.push_back(makeMainWindowItem("item-b1", "BravoOne", "Sword", tabB));
    items.push_back(makeMainWindowItem("item-b2", "BravoTwo", "Sword", tabB));
    fixture.itemsManager->OnItemsRefreshed(items, {tabA, tabB}, false);

    auto *model = tree->model();
    const QModelIndex bucketA = findBucket(*model, tabA.GetHeader());
    const QModelIndex bucketB = findBucket(*model, tabB.GetHeader());
    QVERIFY(bucketA.isValid());
    QVERIFY(bucketB.isValid());
    tree->expand(bucketA);
    tree->expand(bucketB);

    // Select a row in B and pin persistent indexes across its subtree.
    const QModelIndex selected = findItemRow(*model, bucketB, "BravoOne Sword");
    QVERIFY(selected.isValid());
    tree->selectionModel()->setCurrentIndex(selected,
                                            QItemSelectionModel::ClearAndSelect
                                                | QItemSelectionModel::Rows);
    const QPersistentModelIndex persistentBucketB(bucketB);
    const QPersistentModelIndex persistentRowB(selected);
    const QStringList namesB = bucketItemNames(*model, bucketB);

    auto &probes = ModelProbes::instance();
    probes.reset();
    probes.enabled = true;
    QSignalSpy resets(model, &QAbstractItemModel::modelReset);

    // Refresh tab A: a full replacement with fresh objects.
    fixture.itemsManager->OnTabRefreshed(tabA,
                                         {makeMainWindowItem("item-a3", "AlphaThree", "Sword", tabA),
                                          makeMainWindowItem("item-a1", "AlphaOne", "Sword", tabA)});

    // Nothing outside A moved: no reset, no capture/restore/reselect
    // machinery, B's rows and persistent indexes bit-identical, and the
    // selection never lapsed.
    QCOMPARE(resets.count(), 0);
    QCOMPARE(probes.expansion_captures, 0);
    QCOMPARE(probes.expansion_restores, 0);
    QCOMPARE(probes.scroll_captures, 0);
    QCOMPARE(probes.scroll_restores, 0);
    QCOMPARE(probes.reselects, 0);
    QCOMPARE(probes.refilters, 0);
    probes.enabled = false;
    QVERIFY(tree->isExpanded(findBucket(*model, tabB.GetHeader())));
    QVERIFY(persistentBucketB.isValid());
    QVERIFY(persistentRowB.isValid());
    QCOMPARE(persistentRowB.data().toString(), "BravoOne Sword");
    QCOMPARE(bucketItemNames(*model, findBucket(*model, tabB.GetHeader())), namesB);
    const QModelIndexList selectedRows = tree->selectionModel()->selectedRows();
    QCOMPARE(selectedRows.size(), 1);
    QCOMPARE(selectedRows.front().data().toString(), "BravoOne Sword");
}

// M3 S4 (R1-1): a content delta's row operations touch exactly the rows
// fetched from its FetchSourceKey within the affected bucket; row
// accounting, filtered membership, and keyed order are correct after
// application.
void MainWindowTest::deltaReplacesExactlyItsSourceRows()
{
    MainWindowFixture fixture;
    auto *tree = fixture.window->findChild<QTreeView *>("treeView");
    auto *name = findNameFilter(*fixture.window);
    QVERIFY(tree);
    QVERIFY(name);

    const ItemLocation tabA = makeTestStashLocation("stash-alpha", "Alpha Tab", 0);
    const ItemLocation tabB = makeTestStashLocation("stash-beta", "Beta Tab", 1);
    Items items;
    items.push_back(makeMainWindowItem("item-a1", "AlphaItem One", "Sword", tabA));
    items.push_back(makeMainWindowItem("item-a2", "AlphaItem Two", "Sword", tabA));
    items.push_back(makeMainWindowItem("item-b1", "BetaItem", "Sword", tabB));
    fixture.itemsManager->OnItemsRefreshed(items, {tabA, tabB}, false);

    // An active filter, so filtered membership is exercised: "item"
    // matches every seeded name (the arrivals include one it rejects).
    name->setFocus();
    QTest::keyClicks(name, "item");
    fixture.window->OnSearchFormChange();
    tree->header()->setSortIndicator(0, Qt::AscendingOrder);

    auto *model = tree->model();
    // Expand A so the delta takes the visible-bucket merge path (R2-2).
    tree->expand(findBucket(*model, tabA.GetHeader()));
    QSignalSpy resets(model, &QAbstractItemModel::modelReset);
    QSignalSpy removals(model, &QAbstractItemModel::rowsRemoved);
    QSignalSpy insertions(model, &QAbstractItemModel::rowsInserted);

    // The delta replaces tab A: one retained id under a fresh object, one
    // new item, and one arrival the active filter rejects.
    fixture.itemsManager
        ->OnTabRefreshed(tabA,
                         {makeMainWindowItem("item-a4", "AlphaItem Zed", "Sword", tabA),
                          makeMainWindowItem("item-a2", "AlphaItem Two", "Sword", tabA),
                          makeMainWindowItem("item-a5", "NomatchAxe", "Sword", tabA)});

    // No reset; A holds exactly the filtered arrivals in keyed order; B's
    // rows were never touched.
    QCOMPARE(resets.count(), 0);
    QCOMPARE(removals.count(), 1);
    QVERIFY(insertions.count() >= 1);
    const QModelIndex bucketA = findBucket(*model, tabA.GetHeader());
    QVERIFY(bucketA.isValid());
    QCOMPARE(bucketItemNames(*model, bucketA),
             QStringList({"AlphaItem Two Sword", "AlphaItem Zed Sword"}));
    const QModelIndex bucketB = findBucket(*model, tabB.GetHeader());
    QVERIFY(bucketB.isValid());
    QCOMPARE(bucketItemNames(*model, bucketB), QStringList({"BetaItem Sword"}));
}

// M3 S4 (R1-1, R2-2): with an expanded parent bucket holding parent items
// plus two children's items whose sort keys interleave, one child's delta
// replaces only that child's rows AND the resulting bucket order is
// globally sorted — the arrivals are merged into the retained rows, not
// appended or sorted separately. Sibling and parent rows' persistent
// indexes survive.
void MainWindowTest::childDeltaPreservesSiblingSourcesInParentBucket()
{
    MainWindowFixture fixture;
    auto *tree = fixture.window->findChild<QTreeView *>("treeView");
    QVERIFY(tree);

    const ItemLocation parent = makeTestStashLocation("stash-pppp", "Maps", 0);
    ItemLocation child1 = parent;
    child1.setFetchId("child-0001");
    ItemLocation child2 = parent;
    child2.setFetchId("child-0002");

    // Names interleave under the Name sort across all three sources.
    Items items;
    items.push_back(makeMainWindowItem("item-p1", "Alpha", "Sword", parent));
    items.push_back(makeMainWindowItem("item-c1a", "Bravo", "Sword", child1));
    items.push_back(makeMainWindowItem("item-c2a", "Charlie", "Sword", child2));
    items.push_back(makeMainWindowItem("item-p2", "Echo", "Sword", parent));
    items.push_back(makeMainWindowItem("item-c1b", "Foxtrot", "Sword", child1));
    items.push_back(makeMainWindowItem("item-c2b", "Golf", "Sword", child2));
    fixture.itemsManager->OnItemsRefreshed(items, {parent}, false);

    auto *model = tree->model();
    tree->header()->setSortIndicator(0, Qt::AscendingOrder);
    const QModelIndex bucket = findBucket(*model, parent.GetHeader());
    QVERIFY(bucket.isValid());
    tree->expand(bucket);
    QCOMPARE(bucketItemNames(*model, bucket),
             QStringList({"Alpha Sword",
                          "Bravo Sword",
                          "Charlie Sword",
                          "Echo Sword",
                          "Foxtrot Sword",
                          "Golf Sword"}));

    // Pin the sibling's and parent's rows.
    const QPersistentModelIndex alphaRow(findItemRow(*model, bucket, "Alpha Sword"));
    const QPersistentModelIndex charlieRow(findItemRow(*model, bucket, "Charlie Sword"));
    const QPersistentModelIndex golfRow(findItemRow(*model, bucket, "Golf Sword"));
    QVERIFY(alphaRow.isValid() && charlieRow.isValid() && golfRow.isValid());

    QSignalSpy resets(model, &QAbstractItemModel::modelReset);
    // Child 1's delta: Bravo and Foxtrot leave; Delta and Hotel arrive,
    // interleaving with the retained rows under the sort.
    fixture.itemsManager->OnTabRefreshed(child1,
                                         {makeMainWindowItem("item-c1c", "Hotel", "Sword", child1),
                                          makeMainWindowItem("item-c1d", "Delta", "Sword", child1)});

    QCOMPARE(resets.count(), 0);
    // Globally sorted: the merge interleaved the arrivals; an append or a
    // separate arrivals-only sort would leave Delta and Hotel at the end.
    QCOMPARE(bucketItemNames(*model, findBucket(*model, parent.GetHeader())),
             QStringList({"Alpha Sword",
                          "Charlie Sword",
                          "Delta Sword",
                          "Echo Sword",
                          "Golf Sword",
                          "Hotel Sword"}));
    // Sibling and parent rows survived with their identities.
    QVERIFY(alphaRow.isValid());
    QVERIFY(charlieRow.isValid());
    QVERIFY(golfRow.isValid());
    QCOMPARE(alphaRow.data().toString(), "Alpha Sword");
    QCOMPARE(charlieRow.data().toString(), "Charlie Sword");
    QCOMPARE(golfRow.data().toString(), "Golf Sword");
}

// M3 S4: the M2 D6 boundary at the model layer — an empty replacement
// leaves an empty bucket row in an unfiltered search, never a bucket
// removal (deletion is a snapshot-boundary effect).
void MainWindowTest::emptyDeltaEmptiesBucketWithoutRemovingIt()
{
    MainWindowFixture fixture;
    auto *tree = fixture.window->findChild<QTreeView *>("treeView");
    QVERIFY(tree);

    const ItemLocation tabA = makeTestStashLocation("stash-alpha", "Alpha Tab", 0);
    const ItemLocation tabB = makeTestStashLocation("stash-beta", "Beta Tab", 1);
    Items items;
    items.push_back(makeMainWindowItem("item-a1", "AlphaItem", "Sword", tabA));
    items.push_back(makeMainWindowItem("item-b1", "BetaItem", "Sword", tabB));
    fixture.itemsManager->OnItemsRefreshed(items, {tabA, tabB}, false);

    auto *model = tree->model();
    const int top_level_rows = model->rowCount();
    QSignalSpy resets(model, &QAbstractItemModel::modelReset);

    fixture.itemsManager->OnTabRefreshed(tabA, {});

    QCOMPARE(resets.count(), 0);
    QCOMPARE(model->rowCount(), top_level_rows);
    const QModelIndex bucketA = findBucket(*model, tabA.GetHeader());
    QVERIFY(bucketA.isValid());
    QCOMPARE(model->rowCount(bucketA), 0);
    QCOMPARE(visibleItemNames(*tree), QStringList({"BetaItem Sword"}));
}

// M3 S4: after a delta, the visible indexes answer as if freshly
// refiltered with no whole-collection rebuild — observed through the
// rebuild probe staying at zero while identity-dependent behavior
// (replacement adoption, cross-tab re-adoption, second-removal
// adjudication) works against the maintained indexes.
void MainWindowTest::deltaUpdatesVisibleIndexesIncrementally()
{
    MainWindowFixture fixture;
    auto *tree = fixture.window->findChild<QTreeView *>("treeView");
    QVERIFY(tree);

    const ItemLocation tabA = makeTestStashLocation("stash-alpha", "Alpha Tab", 0);
    const ItemLocation tabB = makeTestStashLocation("stash-beta", "Beta Tab", 1);
    Items items;
    items.push_back(makeMainWindowItem("item-x", "Mover", "Sword", tabA));
    items.push_back(makeMainWindowItem("item-b1", "BetaItem", "Sword", tabB));
    fixture.itemsManager->OnItemsRefreshed(items, {tabA, tabB}, false);

    auto *model = tree->model();
    const QModelIndex bucketA = findBucket(*model, tabA.GetHeader());
    QVERIFY(bucketA.isValid());
    tree->expand(bucketA);
    const QModelIndex mover = findItemRow(*model, bucketA, "Mover Sword");
    QVERIFY(mover.isValid());
    tree->selectionModel()->setCurrentIndex(mover,
                                            QItemSelectionModel::ClearAndSelect
                                                | QItemSelectionModel::Rows);

    auto &probes = ModelProbes::instance();
    probes.reset();
    probes.enabled = true;
    QSignalSpy resets(model, &QAbstractItemModel::modelReset);
    QSignalSpy removals(model, &QAbstractItemModel::rowsRemoved);

    // Replacement adoption: the same id under a fresh object — selection
    // follows through the id index (it answered with the new object).
    fixture.itemsManager->OnTabRefreshed(tabA,
                                         {makeMainWindowItem("item-x", "Mover Two", "Sword", tabA)});
    QCOMPARE(tree->selectionModel()->selectedRows().size(), 1);
    QCOMPARE(tree->selectionModel()->selectedRows().front().data().toString(), "Mover Two Sword");

    // Cross-tab move: removal then insertion in another bucket; the
    // re-adoption walks the maintained id index.
    fixture.itemsManager->OnTabRefreshed(tabA, {});
    fixture.itemsManager->OnTabRefreshed(tabB,
                                         {makeMainWindowItem("item-x", "Mover Two", "Sword", tabB),
                                          makeMainWindowItem("item-b1", "BetaItem", "Sword", tabB)});
    const QModelIndex bucketB = findBucket(*model, tabB.GetHeader());
    QVERIFY(bucketB.isValid());
    QCOMPARE(tree->selectionModel()->selectedRows().size(), 1);
    QCOMPARE(tree->selectionModel()->selectedRows().front().parent(), bucketB);

    // Second-removal adjudication: tab A's source no longer has visible
    // rows, so its empty delta finds nothing to remove (the source index
    // was maintained) — no row operations at all.
    const int removals_before = removals.count();
    fixture.itemsManager->OnTabRefreshed(tabA, {});
    QCOMPARE(removals.count(), removals_before);

    // All of it without a single whole-collection rebuild or refilter.
    QCOMPARE(probes.index_rebuilds, 0);
    QCOMPARE(probes.refilters, 0);
    QCOMPARE(resets.count(), 0);
    probes.enabled = false;
}

// M3 S4 (D3): a metadata delta that changes display ordering repositions
// the bucket via move operations; expansion and selection follow the
// stable (type, id) key — M2 R6-3's pins extended from reset-restore to
// move.
void MainWindowTest::bucketRepositionsByMoveOnMetadataDelta()
{
    MainWindowFixture fixture;
    auto *tree = fixture.window->findChild<QTreeView *>("treeView");
    QVERIFY(tree);

    const ItemLocation tabA = makeTestStashLocation("stash-aaaa", "Alpha", 0);
    const ItemLocation tabB = makeTestStashLocation("stash-bbbb", "Beta", 1);
    const ItemLocation tabC = makeTestStashLocation("stash-cccc", "Gamma", 2);
    Items items;
    items.push_back(makeMainWindowItem("item-a", "AlphaItem", "Sword", tabA));
    items.push_back(makeMainWindowItem("item-b", "BetaItem", "Sword", tabB));
    items.push_back(makeMainWindowItem("item-c", "GammaItem", "Sword", tabC));
    fixture.itemsManager->OnItemsRefreshed(items, {tabA, tabB, tabC}, false);

    auto *model = tree->model();
    const QModelIndex bucketB = findBucket(*model, tabB.GetHeader());
    QVERIFY(bucketB.isValid());
    tree->expand(bucketB);
    const QModelIndex selected = model->index(0, 0, bucketB);
    QVERIFY(selected.isValid());
    tree->selectionModel()->setCurrentIndex(selected,
                                            QItemSelectionModel::ClearAndSelect
                                                | QItemSelectionModel::Rows);

    QSignalSpy resets(model, &QAbstractItemModel::modelReset);
    QSignalSpy moves(model, &QAbstractItemModel::rowsMoved);

    // The delta moves B to the end of the display order, carrying the
    // tab's items under the fresh anchor (a delta replaces its source; an
    // empty one would also empty the tab).
    const ItemLocation movedB = makeTestStashLocation("stash-bbbb", "Beta", 5);
    fixture.itemsManager
        ->OnTabRefreshed(movedB, {makeMainWindowItem("item-b", "BetaItem", "Sword", movedB)});

    QCOMPARE(resets.count(), 0);
    QCOMPARE(moves.count(), 1);
    QStringList headers;
    for (int row = 0; row < model->rowCount(); ++row) {
        headers.append(model->index(row, 0).data().toString());
    }
    QCOMPARE(headers, QStringList({"#1, \"Alpha\"", "#3, \"Gamma\"", "#6, \"Beta\""}));

    // Expansion and selection followed the stable key through the move.
    const QModelIndex movedBucket = findBucket(*model, movedB.GetHeader());
    QVERIFY(movedBucket.isValid());
    QCOMPARE(movedBucket.row(), 2);
    QVERIFY(tree->isExpanded(movedBucket));
    const QModelIndexList selectedRows = tree->selectionModel()->selectedRows();
    QCOMPARE(selectedRows.size(), 1);
    QCOMPARE(selectedRows.front().data().toString(), "BetaItem Sword");
    QCOMPARE(selectedRows.front().parent(), movedBucket);
}

// M3 S4 (R1-4): empty deltas carrying a rename, a move, and a
// new-empty-tab discovery each apply immediately — dataChanged / move /
// bucket insertion in an unfiltered search — with no final snapshot and
// no refilter, and the applied state persists after a terminal failure.
// (A color change rides the identical dataChanged path as the rename.)
// M2 R7-2's exception is retired: metadata-only deltas are inside the
// freshness statement now.
void MainWindowTest::metadataDeltaAppliesWithoutItemIntersection()
{
    MainWindowFixture fixture;
    auto *tree = fixture.window->findChild<QTreeView *>("treeView");
    QVERIFY(tree);

    const ItemLocation tabA = makeTestStashLocation("stash-aaaa", "Alpha", 0);
    const ItemLocation tabB = makeTestStashLocation("stash-bbbb", "Beta", 1);
    const ItemLocation tabC = makeTestStashLocation("stash-cccc", "Gamma", 2);
    Items items;
    items.push_back(makeMainWindowItem("item-a", "AlphaItem", "Sword", tabA));
    fixture.itemsManager->OnItemsRefreshed(items, {tabA, tabB, tabC}, false);

    auto *model = tree->model();
    auto &probes = ModelProbes::instance();
    probes.reset();
    probes.enabled = true;
    QSignalSpy resets(model, &QAbstractItemModel::modelReset);
    QSignalSpy insertions(model, &QAbstractItemModel::rowsInserted);

    // Empty deltas: B renamed in place, C moved, E newly discovered
    // (never in the published tab list).
    fixture.itemsManager->OnTabRefreshed(makeTestStashLocation("stash-bbbb", "Beta Renamed", 1), {});
    fixture.itemsManager->OnTabRefreshed(makeTestStashLocation("stash-cccc", "Gamma", 5), {});
    fixture.itemsManager->OnTabRefreshed(makeTestStashLocation("stash-eeee", "Epsilon", 4), {});

    // Everything rendered now: no reset, no refilter, no waiting for the
    // next user action or the final snapshot.
    const auto readHeaders = [&]() {
        QStringList headers;
        for (int row = 0; row < model->rowCount(); ++row) {
            headers.append(model->index(row, 0).data().toString());
        }
        return headers;
    };
    const QStringList expected(
        {"#1, \"Alpha\"", "#2, \"Beta Renamed\"", "#5, \"Epsilon\"", "#6, \"Gamma\""});
    QCOMPARE(readHeaders(), expected);
    QCOMPARE(resets.count(), 0);
    QCOMPARE(probes.refilters, 0);
    QCOMPARE(insertions.count(), 1); // Epsilon's bucket row

    // A terminal failure emits no final snapshot; the applied state
    // persists — the M2 "invisible until user action after terminal
    // failure" caveat is gone.
    fixture.window->OnRefreshFinished(RefreshOutcome{FailedRefresh{RateLimit::FetchError{}}});
    QCOMPARE(readHeaders(), expected);
    QCOMPARE(resets.count(), 0);
    QCOMPARE(probes.refilters, 0);
    probes.enabled = false;
}

// M3 S4 (R1-5): expand (sort), collapse, replace the bucket's contents by
// delta, re-expand — the bucket re-sorts exactly once, correctly. The
// delta's application on the collapsed bucket is arrival-ordered and
// sort-free; the deferred sort pays at expansion.
void MainWindowTest::collapsedInvalidBucketResortsOnReexpand()
{
    MainWindowFixture fixture;
    auto *tree = fixture.window->findChild<QTreeView *>("treeView");
    QVERIFY(tree);

    const ItemLocation tabA = makeTestStashLocation("stash-alpha", "Alpha Tab", 0);
    Items items;
    items.push_back(makeMainWindowItem("item-a1", "Charlie", "Sword", tabA));
    items.push_back(makeMainWindowItem("item-a2", "Alpha", "Sword", tabA));
    fixture.itemsManager->OnItemsRefreshed(items, {tabA}, false);

    auto *model = tree->model();
    tree->header()->setSortIndicator(0, Qt::AscendingOrder);
    const QModelIndex bucket = findBucket(*model, tabA.GetHeader());
    QVERIFY(bucket.isValid());
    tree->expand(bucket);
    QCOMPARE(bucketItemNames(*model, bucket), QStringList({"Alpha Sword", "Charlie Sword"}));
    tree->collapse(bucket);

    // The replacement lands on the collapsed bucket: no sort runs during
    // application (probe), and the rows sit in arrival order.
    auto &probes = ModelProbes::instance();
    probes.reset();
    probes.enabled = true;
    fixture.itemsManager->OnTabRefreshed(tabA,
                                         {makeMainWindowItem("item-a3", "Zulu", "Sword", tabA),
                                          makeMainWindowItem("item-a4", "Bravo", "Sword", tabA),
                                          makeMainWindowItem("item-a5", "Yankee", "Sword", tabA)});
    QCOMPARE(probes.bucket_sorts, 0);
    QCOMPARE(probes.key_builds, 0);

    // Re-expansion sorts exactly that bucket, exactly once, correctly.
    tree->expand(findBucket(*model, tabA.GetHeader()));
    QCOMPARE(probes.bucket_sorts, 1);
    QCOMPARE(probes.bucket_sorts_by_location[LocationInventory::KeyFor(tabA)], 1);
    probes.enabled = false;
    QCOMPARE(bucketItemNames(*model, findBucket(*model, tabA.GetHeader())),
             QStringList({"Bravo Sword", "Yankee Sword", "Zulu Sword"}));
}

// M3 S4: a delta replacing a visible bucket's items yields fresh keyed
// order as part of application; stale order never persists on a visible
// bucket, and the order refresh is counted for that bucket alone.
void MainWindowTest::staleOrderNeverSurvivesDelta()
{
    MainWindowFixture fixture;
    auto *tree = fixture.window->findChild<QTreeView *>("treeView");
    QVERIFY(tree);

    const ItemLocation tabA = makeTestStashLocation("stash-alpha", "Alpha Tab", 0);
    const ItemLocation tabB = makeTestStashLocation("stash-beta", "Beta Tab", 1);
    Items items;
    items.push_back(makeMainWindowItem("item-a1", "Charlie", "Sword", tabA));
    items.push_back(makeMainWindowItem("item-a2", "Alpha", "Sword", tabA));
    items.push_back(makeMainWindowItem("item-b1", "BetaItem", "Sword", tabB));
    fixture.itemsManager->OnItemsRefreshed(items, {tabA, tabB}, false);

    auto *model = tree->model();
    tree->header()->setSortIndicator(0, Qt::AscendingOrder);
    const QModelIndex bucketA = findBucket(*model, tabA.GetHeader());
    const QModelIndex bucketB = findBucket(*model, tabB.GetHeader());
    QVERIFY(bucketA.isValid());
    QVERIFY(bucketB.isValid());
    tree->expand(bucketA);
    tree->expand(bucketB);

    auto &probes = ModelProbes::instance();
    probes.reset();
    probes.enabled = true;
    QSignalSpy resets(model, &QAbstractItemModel::modelReset);

    // Arrival order (Zulu, Bravo, Mike) deliberately disagrees with the
    // sort; the application must present keyed order immediately.
    fixture.itemsManager->OnTabRefreshed(tabA,
                                         {makeMainWindowItem("item-a3", "Zulu", "Sword", tabA),
                                          makeMainWindowItem("item-a4", "Bravo", "Sword", tabA),
                                          makeMainWindowItem("item-a5", "Mike", "Sword", tabA)});

    QCOMPARE(resets.count(), 0);
    QCOMPARE(bucketItemNames(*model, findBucket(*model, tabA.GetHeader())),
             QStringList({"Bravo Sword", "Mike Sword", "Zulu Sword"}));
    // The order refresh was counted for A alone; B's order never moved.
    QCOMPARE(probes.bucket_sorts_by_location[LocationInventory::KeyFor(tabA)], 1);
    QCOMPARE(probes.bucket_sorts_by_location[LocationInventory::KeyFor(tabB)], 0);
    probes.enabled = false;
    QCOMPARE(bucketItemNames(*model, findBucket(*model, tabB.GetHeader())),
             QStringList({"BetaItem Sword"}));
}

// M3 S4 (R1-3): the successor of M2's reselectionSurvivesCrossTabMove
// under no-reset machinery. With an item selected, one delta removes it;
// several deltas later another inserts it in a different tab — the
// selection re-adopts by stable id through the global index. If the
// refresh ends without the id reappearing, the final boundary clears the
// selection; a user selection made in between wins outright.
void MainWindowTest::selectionIntentSurvivesCrossTabMoveAcrossDeltas()
{
    // Clause 1: removal → unrelated delta → insertion re-adopts.
    {
        MainWindowFixture fixture;
        auto *tree = fixture.window->findChild<QTreeView *>("treeView");
        auto *locationLabel = fixture.window->findChild<QLabel *>("locationLabel");
        QVERIFY(tree);
        QVERIFY(locationLabel);

        const ItemLocation tabA = makeTestStashLocation("stash-aaaa", "Alpha", 0);
        const ItemLocation tabB = makeTestStashLocation("stash-bbbb", "Beta", 1);
        const ItemLocation tabC = makeTestStashLocation("stash-cccc", "Gamma", 2);
        Items items;
        items.push_back(makeMainWindowItem("item-x", "Mover", "Sword", tabA));
        items.push_back(makeMainWindowItem("item-b", "BetaItem", "Shield", tabB));
        items.push_back(makeMainWindowItem("item-c", "GammaItem", "Axe", tabC));
        fixture.itemsManager->OnItemsRefreshed(items, {tabA, tabB, tabC}, false);

        auto *model = tree->model();
        const QModelIndex bucketA = findBucket(*model, tabA.GetHeader());
        QVERIFY(bucketA.isValid());
        tree->expand(bucketA);
        const QModelIndex mover = model->index(0, 0, bucketA);
        QVERIFY(mover.isValid());
        QCOMPARE(mover.data().toString(), "Mover Sword");
        tree->selectionModel()->setCurrentIndex(mover,
                                                QItemSelectionModel::ClearAndSelect
                                                    | QItemSelectionModel::Rows);

        QSignalSpy resets(model, &QAbstractItemModel::modelReset);

        // The removal delta: the visual selection lapses, the intent lives.
        fixture.itemsManager->OnTabRefreshed(tabA, {});
        QCOMPARE(resets.count(), 0);
        QCOMPARE(tree->selectionModel()->selectedRows().size(), 0);

        // Several deltas later — an unrelated one in between.
        fixture.itemsManager
            ->OnTabRefreshed(tabC, {makeMainWindowItem("item-c", "GammaItem", "Axe", tabC)});
        QCOMPARE(tree->selectionModel()->selectedRows().size(), 0);

        // The insertion in another tab: re-adoption by stable id.
        fixture.itemsManager
            ->OnTabRefreshed(tabB,
                             {makeMainWindowItem("item-x", "Mover", "Sword", tabB),
                              makeMainWindowItem("item-b", "BetaItem", "Shield", tabB)});
        QCOMPARE(resets.count(), 0);
        const QModelIndexList selectedRows = tree->selectionModel()->selectedRows();
        QCOMPARE(selectedRows.size(), 1);
        QCOMPARE(selectedRows.front().data().toString(), "Mover Sword");
        const QModelIndex bucketB = findBucket(*model, tabB.GetHeader());
        QVERIFY(bucketB.isValid());
        QCOMPARE(selectedRows.front().parent(), bucketB);
        // The details panel adopted the replacement object.
        QTRY_COMPARE_WITH_TIMEOUT(locationLabel->text(), tabB.GetHeader(), 2000);
    }

    // Clause 2: the refresh ends without the id reappearing — the final
    // boundary clears the selection, and a later refresh reinserting the
    // id does not resurrect it.
    {
        MainWindowFixture fixture;
        auto *tree = fixture.window->findChild<QTreeView *>("treeView");
        QVERIFY(tree);

        const ItemLocation tabA = makeTestStashLocation("stash-aaaa", "Alpha", 0);
        Items items;
        items.push_back(makeMainWindowItem("item-x", "Mover", "Sword", tabA));
        items.push_back(makeMainWindowItem("item-y", "Stayer", "Sword", tabA));
        fixture.itemsManager->OnItemsRefreshed(items, {tabA}, false);

        auto *model = tree->model();
        const QModelIndex bucketA = findBucket(*model, tabA.GetHeader());
        tree->expand(bucketA);
        const QModelIndex mover = findItemRow(*model, bucketA, "Mover Sword");
        QVERIFY(mover.isValid());
        tree->selectionModel()->setCurrentIndex(mover,
                                                QItemSelectionModel::ClearAndSelect
                                                    | QItemSelectionModel::Rows);

        const auto stayer = makeMainWindowItem("item-y", "Stayer", "Sword", tabA);
        fixture.itemsManager->OnTabRefreshed(tabA, {stayer});
        QCOMPARE(tree->selectionModel()->selectedRows().size(), 0);

        // The refresh completes; the id never reappeared.
        Items final_items;
        final_items.push_back(stayer);
        fixture.itemsManager->OnItemsRefreshed(final_items, {tabA}, false);
        fixture.window->OnRefreshFinished(RefreshOutcome{CompletedRefresh{}});
        QCOMPARE(tree->selectionModel()->selectedRows().size(), 0);

        // A later refresh reinserting the id must not reselect it.
        fixture.itemsManager->OnTabRefreshed(tabA,
                                             {makeMainWindowItem("item-x", "Mover", "Sword", tabA),
                                              stayer});
        QCOMPARE(tree->selectionModel()->selectedRows().size(), 0);
    }

    // Clause 3: a user selection made mid-window wins outright.
    {
        MainWindowFixture fixture;
        auto *tree = fixture.window->findChild<QTreeView *>("treeView");
        QVERIFY(tree);

        const ItemLocation tabA = makeTestStashLocation("stash-aaaa", "Alpha", 0);
        const ItemLocation tabB = makeTestStashLocation("stash-bbbb", "Beta", 1);
        Items items;
        items.push_back(makeMainWindowItem("item-x", "Mover", "Sword", tabA));
        items.push_back(makeMainWindowItem("item-b", "BetaItem", "Shield", tabB));
        fixture.itemsManager->OnItemsRefreshed(items, {tabA, tabB}, false);

        auto *model = tree->model();
        const QModelIndex bucketA = findBucket(*model, tabA.GetHeader());
        const QModelIndex bucketB = findBucket(*model, tabB.GetHeader());
        tree->expand(bucketA);
        tree->expand(bucketB);
        tree->selectionModel()->setCurrentIndex(findItemRow(*model, bucketA, "Mover Sword"),
                                                QItemSelectionModel::ClearAndSelect
                                                    | QItemSelectionModel::Rows);

        // Removal lapses the selection; the user then picks another item.
        fixture.itemsManager->OnTabRefreshed(tabA, {});
        tree->selectionModel()->setCurrentIndex(findItemRow(*model,
                                                            findBucket(*model, tabB.GetHeader()),
                                                            "BetaItem Shield"),
                                                QItemSelectionModel::ClearAndSelect
                                                    | QItemSelectionModel::Rows);

        // The old id reappears: the user's selection stands.
        fixture.itemsManager->OnTabRefreshed(tabA,
                                             {makeMainWindowItem("item-x", "Mover", "Sword", tabA)});
        const QModelIndexList selectedRows = tree->selectionModel()->selectedRows();
        QCOMPARE(selectedRows.size(), 1);
        QCOMPARE(selectedRows.front().data().toString(), "BetaItem Shield");
    }
}

// M3 S4 (R2-1): deltas remove the selected item, then the refresh fails
// terminally — no final snapshot. The intent is cleared at the terminal
// event, and a later refresh reinserting the same id does not reselect it.
void MainWindowTest::selectionIntentClearsOnTerminalFailure()
{
    MainWindowFixture fixture;
    auto *tree = fixture.window->findChild<QTreeView *>("treeView");
    QVERIFY(tree);

    const ItemLocation tabA = makeTestStashLocation("stash-aaaa", "Alpha", 0);
    Items items;
    items.push_back(makeMainWindowItem("item-x", "Mover", "Sword", tabA));
    items.push_back(makeMainWindowItem("item-y", "Stayer", "Sword", tabA));
    fixture.itemsManager->OnItemsRefreshed(items, {tabA}, false);

    auto *model = tree->model();
    const QModelIndex bucketA = findBucket(*model, tabA.GetHeader());
    QVERIFY(bucketA.isValid());
    tree->expand(bucketA);
    const QModelIndex mover = findItemRow(*model, bucketA, "Mover Sword");
    QVERIFY(mover.isValid());
    tree->selectionModel()->setCurrentIndex(mover,
                                            QItemSelectionModel::ClearAndSelect
                                                | QItemSelectionModel::Rows);

    // The removal delta lapses the selection; the intent is alive.
    fixture.itemsManager->OnTabRefreshed(tabA,
                                         {makeMainWindowItem("item-y", "Stayer", "Sword", tabA)});
    QCOMPARE(tree->selectionModel()->selectedRows().size(), 0);

    // Terminal failure: no final snapshot, and the intent window closes
    // with an absence check against the visible result.
    fixture.window->OnRefreshFinished(RefreshOutcome{FailedRefresh{RateLimit::FetchError{}}});

    // A later refresh reinserts the id: no reselection.
    fixture.itemsManager->OnTabRefreshed(tabA,
                                         {makeMainWindowItem("item-x", "Mover", "Sword", tabA),
                                          makeMainWindowItem("item-y", "Stayer", "Sword", tabA)});
    QCOMPARE(tree->selectionModel()->selectedRows().size(), 0);
}

// M3 S4 (R1-7), By-Tab half — the By-Item half completes in S5 with the
// fallback's deletion: after the active search applies (or correctly
// adjudicates) a series of deltas, switching away and back triggers no
// refilter; a delta arriving while application is impossible (the S4
// By-Item fallback) leaves the flag dirty and the next activation pays.
void MainWindowTest::appliedDeltasLeaveActiveSearchClean()
{
    MainWindowFixture fixture;
    auto *tabs = findSearchTabs(*fixture.window);
    auto *tree = fixture.window->findChild<QTreeView *>("treeView");
    QVERIFY(tabs && tree);

    const ItemLocation tabA = makeTestStashLocation("stash-aaaa", "Alpha", 0);
    const ItemLocation tabB = makeTestStashLocation("stash-bbbb", "Beta", 1);
    Items items;
    items.push_back(makeMainWindowItem("item-a", "AlphaItem", "Sword", tabA));
    items.push_back(makeMainWindowItem("item-b", "BetaItem", "Sword", tabB));
    fixture.itemsManager->OnItemsRefreshed(items, {tabA, tabB}, false);
    // A second search to switch through.
    tabs->setCurrentIndex(1);
    tabs->setCurrentIndex(0);

    // The reset spy is scoped to THIS search's model: the background
    // search refilters on its own activation by design (rule 1), and that
    // must not count against the active search's cleanliness.
    QAbstractItemModel *first_model = tree->model();
    QSignalSpy first_resets(first_model, &QAbstractItemModel::modelReset);

    // A series the active By-Tab search processes: a content replacement,
    // an empty (removal) delta, and a metadata-only delta.
    fixture.itemsManager
        ->OnTabRefreshed(tabA, {makeMainWindowItem("item-a2", "AlphaItem Two", "Sword", tabA)});
    fixture.itemsManager->OnTabRefreshed(tabB, {});
    fixture.itemsManager->OnTabRefreshed(makeTestStashLocation("stash-bbbb", "Beta Renamed", 1), {});
    QCOMPARE(first_resets.count(), 0);

    // Switching away and back triggers no refilter: the deltas left the
    // search clean (M2 rule 1 renegotiated for the active search).
    tabs->setCurrentIndex(1);
    tabs->setCurrentIndex(0);
    QCOMPARE(first_resets.count(), 0);
    QCOMPARE(visibleItemNames(*tree), QStringList({"AlphaItem Two Sword"}));

    // Fail-safe direction: in By-Item the S4 fallback cannot apply, so the
    // delta leaves the flag dirty and the next activation refilters —
    // exactly once.
    switchToByItemView(fixture);
    const int resets_after_switch = first_resets.count();
    fixture.itemsManager
        ->OnTabRefreshed(tabA, {makeMainWindowItem("item-a3", "AlphaItem Three", "Sword", tabA)});
    QCOMPARE(first_resets.count(), resets_after_switch);
    tabs->setCurrentIndex(1);
    tabs->setCurrentIndex(0);
    QCOMPARE(first_resets.count(), resets_after_switch + 1);
    QVERIFY(visibleItemNames(*tree).contains("AlphaItem Three Sword"));
}

// M3 S4 review round 1 (R1-3/R2-1 over the fallback seam; renegotiated
// when the seam dies in S5): the selection-intent contract covers the
// By-Item throttled path — the seam defers application, never intent.
void MainWindowTest::selectionIntentCoversByItemFallback()
{
    // Clause 1: removal tick, then insertion tick — the intent re-adopts
    // through the refilter's refreshed index.
    {
        MainWindowFixture fixture;
        fixture.window->SetDeltaThrottleInterval(100);
        auto *tree = fixture.window->findChild<QTreeView *>("treeView");
        QVERIFY(tree);

        const ItemLocation tabA = makeTestStashLocation("stash-aaaa", "Alpha", 0);
        const ItemLocation tabB = makeTestStashLocation("stash-bbbb", "Beta", 1);
        Items items;
        items.push_back(makeMainWindowItem("item-x", "Mover", "Sword", tabA));
        items.push_back(makeMainWindowItem("item-b", "BetaItem", "Shield", tabB));
        fixture.itemsManager->OnItemsRefreshed(items, {tabA, tabB}, false);
        switchToByItemView(fixture);

        auto *model = tree->model();
        const QModelIndex flatBucket = model->index(0, 0);
        QVERIFY(flatBucket.isValid());
        const QModelIndex mover = findItemRow(*model, flatBucket, "Mover Sword");
        QVERIFY(mover.isValid());
        tree->selectionModel()->setCurrentIndex(mover,
                                                QItemSelectionModel::ClearAndSelect
                                                    | QItemSelectionModel::Rows);

        QSignalSpy resets(model, &QAbstractItemModel::modelReset);
        // The removal delta rides the throttled seam; its tick lapses the
        // visual selection while the intent stays alive.
        fixture.itemsManager->OnTabRefreshed(tabA, {});
        QTRY_COMPARE_WITH_TIMEOUT(resets.count(), 1, 2000);
        QCOMPARE(tree->selectionModel()->selectedRows().size(), 0);

        // The insertion in another tab applies at the next tick: the
        // intent re-adopts by stable id.
        fixture.itemsManager
            ->OnTabRefreshed(tabB,
                             {makeMainWindowItem("item-x", "Mover", "Sword", tabB),
                              makeMainWindowItem("item-b", "BetaItem", "Shield", tabB)});
        QTRY_COMPARE_WITH_TIMEOUT(resets.count(), 2, 2000);
        const QModelIndexList selectedRows = tree->selectionModel()->selectedRows();
        QCOMPARE(selectedRows.size(), 1);
        QCOMPARE(selectedRows.front().data().toString(), "Mover Sword");
    }

    // Clause 2: terminal failure while the index is stale — the closure
    // defers to the tick's refilter, which finds the id absent and drops
    // the intent; a later reinsertion must not reselect.
    {
        MainWindowFixture fixture;
        fixture.window->SetDeltaThrottleInterval(100);
        auto *tree = fixture.window->findChild<QTreeView *>("treeView");
        QVERIFY(tree);

        const ItemLocation tabA = makeTestStashLocation("stash-aaaa", "Alpha", 0);
        Items items;
        items.push_back(makeMainWindowItem("item-x", "Mover", "Sword", tabA));
        items.push_back(makeMainWindowItem("item-y", "Stayer", "Sword", tabA));
        fixture.itemsManager->OnItemsRefreshed(items, {tabA}, false);
        switchToByItemView(fixture);

        auto *model = tree->model();
        const QModelIndex flatBucket = model->index(0, 0);
        const QModelIndex mover = findItemRow(*model, flatBucket, "Mover Sword");
        QVERIFY(mover.isValid());
        tree->selectionModel()->setCurrentIndex(mover,
                                                QItemSelectionModel::ClearAndSelect
                                                    | QItemSelectionModel::Rows);

        QSignalSpy resets(model, &QAbstractItemModel::modelReset);
        // Removal delta, then the terminal failure lands BEFORE the tick:
        // the still-stale index would claim the item is present.
        fixture.itemsManager
            ->OnTabRefreshed(tabA, {makeMainWindowItem("item-y", "Stayer", "Sword", tabA)});
        fixture.window->OnRefreshFinished(RefreshOutcome{FailedRefresh{RateLimit::FetchError{}}});

        // The pending tick fires; the refilter is the first honest state
        // and the deferred closure drops the intent.
        QTRY_COMPARE_WITH_TIMEOUT(resets.count(), 1, 2000);
        QCOMPARE(tree->selectionModel()->selectedRows().size(), 0);

        // A later refresh reinserting the id must not reselect it.
        fixture.itemsManager->OnTabRefreshed(tabA,
                                             {makeMainWindowItem("item-x", "Mover", "Sword", tabA),
                                              makeMainWindowItem("item-y", "Stayer", "Sword", tabA)});
        QTRY_COMPARE_WITH_TIMEOUT(resets.count(), 2, 2000);
        QCOMPARE(tree->selectionModel()->selectedRows().size(), 0);
    }

    // Clause 3: the id reappears mid-refresh but its application is
    // deferred past the terminal failure — the deferred window still
    // adopts it at the tick.
    {
        MainWindowFixture fixture;
        fixture.window->SetDeltaThrottleInterval(100);
        auto *tree = fixture.window->findChild<QTreeView *>("treeView");
        QVERIFY(tree);

        const ItemLocation tabA = makeTestStashLocation("stash-aaaa", "Alpha", 0);
        const ItemLocation tabB = makeTestStashLocation("stash-bbbb", "Beta", 1);
        Items items;
        items.push_back(makeMainWindowItem("item-x", "Mover", "Sword", tabA));
        items.push_back(makeMainWindowItem("item-b", "BetaItem", "Shield", tabB));
        fixture.itemsManager->OnItemsRefreshed(items, {tabA, tabB}, false);
        switchToByItemView(fixture);

        auto *model = tree->model();
        const QModelIndex mover = findItemRow(*model, model->index(0, 0), "Mover Sword");
        QVERIFY(mover.isValid());
        tree->selectionModel()->setCurrentIndex(mover,
                                                QItemSelectionModel::ClearAndSelect
                                                    | QItemSelectionModel::Rows);

        QSignalSpy resets(model, &QAbstractItemModel::modelReset);
        fixture.itemsManager->OnTabRefreshed(tabA, {});
        fixture.itemsManager
            ->OnTabRefreshed(tabB,
                             {makeMainWindowItem("item-x", "Mover", "Sword", tabB),
                              makeMainWindowItem("item-b", "BetaItem", "Shield", tabB)});
        fixture.window->OnRefreshFinished(RefreshOutcome{FailedRefresh{RateLimit::FetchError{}}});

        QTRY_COMPARE_WITH_TIMEOUT(resets.count(), 1, 2000);
        const QModelIndexList selectedRows = tree->selectionModel()->selectedRows();
        QCOMPARE(selectedRows.size(), 1);
        QCOMPARE(selectedRows.front().data().toString(), "Mover Sword");
    }

    // Clause 4 (review round 2): a non-intersecting delta marks the
    // search dirty WITHOUT arming the timer, so the terminal closure
    // must adjudicate immediately — by the intersection test's
    // definition such deltas changed nothing visible, so the index is
    // honest despite the flag. A deferral here would stay open into the
    // NEXT refresh, where its late firing could clear the intent between
    // a removal and a cross-tab insertion.
    {
        MainWindowFixture fixture;
        fixture.window->SetDeltaThrottleInterval(100);
        auto *tree = fixture.window->findChild<QTreeView *>("treeView");
        QVERIFY(tree);

        const ItemLocation tabA = makeTestStashLocation("stash-aaaa", "Alpha", 0);
        const ItemLocation tabB = makeTestStashLocation("stash-bbbb", "Beta", 1);
        const ItemLocation tabC = makeTestStashLocation("stash-cccc", "Gamma", 2);
        Items items;
        items.push_back(makeMainWindowItem("item-x", "Mover", "Sword", tabA));
        items.push_back(makeMainWindowItem("item-b", "BetaItem", "Shield", tabB));
        fixture.itemsManager->OnItemsRefreshed(items, {tabA, tabB, tabC}, false);
        switchToByItemView(fixture);

        auto *model = tree->model();
        const QModelIndex mover = findItemRow(*model, model->index(0, 0), "Mover Sword");
        QVERIFY(mover.isValid());
        tree->selectionModel()->setCurrentIndex(mover,
                                                QItemSelectionModel::ClearAndSelect
                                                    | QItemSelectionModel::Rows);

        // Tab C has nothing visible: dirty, no timer. The terminal
        // closure runs now and correctly retains the still-visible id.
        fixture.itemsManager->OnTabRefreshed(tabC, {});
        fixture.window->OnRefreshFinished(RefreshOutcome{FailedRefresh{RateLimit::FetchError{}}});

        // The NEXT refresh moves the item cross-tab through two ticks; a
        // stale deferral would have cleared the intent at the first one.
        QSignalSpy resets(model, &QAbstractItemModel::modelReset);
        fixture.itemsManager->OnTabRefreshed(tabA, {});
        QTRY_COMPARE_WITH_TIMEOUT(resets.count(), 1, 2000);
        QCOMPARE(tree->selectionModel()->selectedRows().size(), 0);
        fixture.itemsManager
            ->OnTabRefreshed(tabB,
                             {makeMainWindowItem("item-x", "Mover", "Sword", tabB),
                              makeMainWindowItem("item-b", "BetaItem", "Shield", tabB)});
        QTRY_COMPARE_WITH_TIMEOUT(resets.count(), 2, 2000);
        const QModelIndexList selectedRows = tree->selectionModel()->selectedRows();
        QCOMPARE(selectedRows.size(), 1);
        QCOMPARE(selectedRows.front().data().toString(), "Mover Sword");
    }
}

// M3 S4 review round 1 (D6; renegotiated when the seam dies in S5): a
// view-mode switch consumes fallback dirtiness — the arriving mode never
// renders un-applied state, and the pending fallback tick is canceled
// rather than left to reset a By-Tab model the throttle no longer owns.
void MainWindowTest::modeSwitchConsumesFallbackDirtiness()
{
    // Clause 1: an intersecting delta armed the tick; the switch pays
    // for the staleness itself and the tick never fires.
    {
        MainWindowFixture fixture;
        fixture.window->SetDeltaThrottleInterval(200);
        auto *tree = fixture.window->findChild<QTreeView *>("treeView");
        auto *viewCombo = fixture.window->findChild<QComboBox *>("viewComboBox");
        QVERIFY(tree && viewCombo);

        const ItemLocation tabA = makeTestStashLocation("stash-aaaa", "Alpha", 0);
        Items items;
        items.push_back(makeMainWindowItem("item-a", "AlphaItem", "Sword", tabA));
        fixture.itemsManager->OnItemsRefreshed(items, {tabA}, false);
        switchToByItemView(fixture);

        // The delta rides the seam: the flat view is stale until a tick.
        fixture.itemsManager
            ->OnTabRefreshed(tabA, {makeMainWindowItem("item-a2", "AlphaItem Two", "Sword", tabA)});
        QVERIFY(visibleItemNames(*tree).contains("AlphaItem Sword"));

        // The switch to By-Tab refilters NOW — as ONE reset (round 2):
        // the dirty flip is quiet and the refilter's reset announces the
        // flip and the fresh content together.
        auto *model = tree->model();
        QSignalSpy resets(model, &QAbstractItemModel::modelReset);
        viewCombo->setCurrentIndex(0);
        emit viewCombo->activated(0);
        QCOMPARE(resets.count(), 1);
        QVERIFY(visibleItemNames(*tree).contains("AlphaItem Two Sword"));
        QVERIFY(!visibleItemNames(*tree).contains("AlphaItem Sword"));

        // And the canceled tick adds nothing afterward.
        const int settled = resets.count();
        QTest::qWait(500);
        QCOMPARE(resets.count(), settled);
    }

    // Clause 2: a metadata-only delta arms no tick; after a terminal
    // failure the switch still renders the fresh anchor immediately.
    {
        MainWindowFixture fixture;
        auto *tree = fixture.window->findChild<QTreeView *>("treeView");
        auto *viewCombo = fixture.window->findChild<QComboBox *>("viewComboBox");
        QVERIFY(tree && viewCombo);

        const ItemLocation tabA = makeTestStashLocation("stash-aaaa", "Alpha", 0);
        const ItemLocation tabB = makeTestStashLocation("stash-bbbb", "Beta", 1);
        Items items;
        items.push_back(makeMainWindowItem("item-a", "AlphaItem", "Sword", tabA));
        fixture.itemsManager->OnItemsRefreshed(items, {tabA, tabB}, false);
        switchToByItemView(fixture);

        const ItemLocation renamedB = makeTestStashLocation("stash-bbbb", "Beta Renamed", 1);
        fixture.itemsManager->OnTabRefreshed(renamedB, {});
        fixture.window->OnRefreshFinished(RefreshOutcome{FailedRefresh{RateLimit::FetchError{}}});

        viewCombo->setCurrentIndex(0);
        emit viewCombo->activated(0);
        QVERIFY(findBucket(*tree->model(), renamedB.GetHeader()).isValid());
        QVERIFY(!findBucket(*tree->model(), tabB.GetHeader()).isValid());
    }
}

// M3 S4 review round 1: a filtered search hides empty buckets, so a
// bucket a delta empties leaves the view — the delta path converges to
// the freshly-refiltered state — and reappears when a delta brings
// matching items back. The unfiltered search keeps its empty row
// (emptyDeltaEmptiesBucketWithoutRemovingIt, unchanged).
void MainWindowTest::filteredSearchDropsEmptiedBucket()
{
    MainWindowFixture fixture;
    auto *tree = fixture.window->findChild<QTreeView *>("treeView");
    auto *name = findNameFilter(*fixture.window);
    auto *nameLabel = fixture.window->findChild<QLabel *>("nameLabel");
    QVERIFY(tree && name && nameLabel);

    const ItemLocation tabA = makeTestStashLocation("stash-aaaa", "Alpha", 0);
    const ItemLocation tabB = makeTestStashLocation("stash-bbbb", "Beta", 1);
    const ItemLocation tabC = makeTestStashLocation("stash-cccc", "Gamma", 2);
    Items items;
    items.push_back(makeMainWindowItem("item-a", "AlphaItem", "Sword", tabA));
    items.push_back(makeMainWindowItem("item-b", "BetaItem", "Sword", tabB));
    items.push_back(makeMainWindowItem("item-c", "NomatchThing", "Sword", tabC));
    fixture.itemsManager->OnItemsRefreshed(items, {tabA, tabB, tabC}, false);

    // Filter "item": C's only item is rejected, so the search is filtered
    // and C's empty bucket is hidden — two buckets visible.
    name->setFocus();
    QTest::keyClicks(name, "item");
    fixture.window->OnSearchFormChange();
    auto *model = tree->model();
    QCOMPARE(model->rowCount(), 2);

    // Select A's header so the drop also clears the details pane.
    const QModelIndex bucketA = findBucket(*model, tabA.GetHeader());
    QVERIFY(bucketA.isValid());
    tree->selectionModel()->setCurrentIndex(bucketA,
                                            QItemSelectionModel::ClearAndSelect
                                                | QItemSelectionModel::Rows);
    QCOMPARE(nameLabel->text(), tabA.GetHeader());

    QSignalSpy resets(model, &QAbstractItemModel::modelReset);
    QSignalSpy removals(model, &QAbstractItemModel::rowsRemoved);

    // The empty delta empties A: its rows leave, then its bucket row.
    fixture.itemsManager->OnTabRefreshed(tabA, {});
    QCOMPARE(resets.count(), 0);
    QCOMPARE(removals.count(), 2); // the child-row run, then the bucket row
    QCOMPARE(model->rowCount(), 1);
    QVERIFY(!findBucket(*model, tabA.GetHeader()).isValid());
    QCOMPARE(nameLabel->text(), "Select an item");

    // Matching arrivals bring the bucket back at its display position.
    fixture.itemsManager
        ->OnTabRefreshed(tabA, {makeMainWindowItem("item-a2", "AlphaItem Two", "Sword", tabA)});
    QCOMPARE(resets.count(), 0);
    QCOMPARE(model->rowCount(), 2);
    QCOMPARE(model->index(0, 0).data().toString(), tabA.GetHeader());

    // A replacement whose arrivals all fail the filter empties too.
    fixture.itemsManager
        ->OnTabRefreshed(tabB, {makeMainWindowItem("item-b2", "NomatchAxe", "Sword", tabB)});
    QCOMPARE(resets.count(), 0);
    QCOMPARE(model->rowCount(), 1);
    QVERIFY(!findBucket(*model, tabB.GetHeader()).isValid());

    // Round 2: "filtered" means ANY filter is active, not "something was
    // rejected" — the old snapshot could be flipped by one delta, a
    // whole-view change no bucket-scoped operation expresses. An active
    // filter that happens to reject nothing still hides empty tabs, and
    // a delta whose arrivals are rejected converges the same way.
    {
        MainWindowFixture fixture2;
        auto *tree2 = fixture2.window->findChild<QTreeView *>("treeView");
        auto *name2 = findNameFilter(*fixture2.window);
        QVERIFY(tree2 && name2);

        const ItemLocation tabD = makeTestStashLocation("stash-dddd", "Delta", 0);
        const ItemLocation tabE = makeTestStashLocation("stash-eeee", "Echo", 1);
        Items all_match;
        all_match.push_back(makeMainWindowItem("item-d", "DeltaItem", "Sword", tabD));
        fixture2.itemsManager->OnItemsRefreshed(all_match, {tabD, tabE}, false);

        // The filter matches every visible item: the search is filtered
        // regardless, so E's empty bucket is hidden.
        name2->setFocus();
        QTest::keyClicks(name2, "item");
        fixture2.window->OnSearchFormChange();
        auto *model2 = tree2->model();
        QCOMPARE(model2->rowCount(), 1);

        // A delta whose arrivals are all rejected empties D, and the
        // emptied bucket leaves — the flag cannot be flipped by a delta.
        QSignalSpy resets2(model2, &QAbstractItemModel::modelReset);
        fixture2.itemsManager
            ->OnTabRefreshed(tabD, {makeMainWindowItem("item-d2", "Nomatch", "Sword", tabD)});
        QCOMPARE(resets2.count(), 0);
        QCOMPARE(model2->rowCount(), 0);
    }
}

// M3 S4 review round 1 (R1-4): a metadata delta refreshes the selected
// presentation state the reset-reselect cycle used to refresh
// implicitly — a selected bucket header re-renders its pane, and a
// RETAINED item (a sibling source's rows untouched by the delta) renders
// its tab's canonical header on the location line.
void MainWindowTest::metadataDeltaRefreshesSelectedPresentation()
{
    MainWindowFixture fixture;
    auto *tree = fixture.window->findChild<QTreeView *>("treeView");
    auto *nameLabel = fixture.window->findChild<QLabel *>("nameLabel");
    auto *locationLabel = fixture.window->findChild<QLabel *>("locationLabel");
    QVERIFY(tree && nameLabel && locationLabel);

    const ItemLocation parent = makeTestStashLocation("stash-pppp", "Maps", 0);
    ItemLocation child = parent;
    child.setFetchId("child-0001");
    const ItemLocation tabB = makeTestStashLocation("stash-bbbb", "Beta", 1);
    Items items;
    items.push_back(makeMainWindowItem("item-p", "ParentItem", "Sword", parent));
    items.push_back(makeMainWindowItem("item-m", "ChildItem", "Shield", child));
    items.push_back(makeMainWindowItem("item-b", "BetaItem", "Sword", tabB));
    fixture.itemsManager->OnItemsRefreshed(items, {parent, tabB}, false);

    auto *model = tree->model();

    // A selected bucket header follows a rename in place.
    const QModelIndex bucketB = findBucket(*model, tabB.GetHeader());
    QVERIFY(bucketB.isValid());
    tree->selectionModel()->setCurrentIndex(bucketB,
                                            QItemSelectionModel::ClearAndSelect
                                                | QItemSelectionModel::Rows);
    QCOMPARE(nameLabel->text(), tabB.GetHeader());
    const ItemLocation renamedB = makeTestStashLocation("stash-bbbb", "Beta Renamed", 1);
    fixture.itemsManager
        ->OnTabRefreshed(renamedB, {makeMainWindowItem("item-b", "BetaItem", "Sword", renamedB)});
    QCOMPARE(nameLabel->text(), renamedB.GetHeader());

    // A retained item renders the canonical header: the child delta
    // renames the parent's anchor without touching the parent-source
    // rows, and the selected ParentItem's location line follows.
    const QModelIndex parentBucket = findBucket(*model, parent.GetHeader());
    QVERIFY(parentBucket.isValid());
    tree->expand(parentBucket);
    const QModelIndex parentItem = findItemRow(*model, parentBucket, "ParentItem Sword");
    QVERIFY(parentItem.isValid());
    tree->selectionModel()->setCurrentIndex(parentItem,
                                            QItemSelectionModel::ClearAndSelect
                                                | QItemSelectionModel::Rows);
    QTRY_COMPARE_WITH_TIMEOUT(locationLabel->text(), parent.GetHeader(), 2000);

    ItemLocation renamedChild = makeTestStashLocation("stash-pppp", "Maps Renamed", 0);
    renamedChild.setFetchId("child-0001");
    fixture.itemsManager
        ->OnTabRefreshed(renamedChild,
                         {makeMainWindowItem("item-m2", "ChildItem Two", "Shield", renamedChild)});
    const ItemLocation renamedParent = makeTestStashLocation("stash-pppp", "Maps Renamed", 0);
    QCOMPARE(locationLabel->text(), renamedParent.GetHeader());
    // The selection never lapsed: the retained row was untouched.
    const QModelIndexList selectedRows = tree->selectionModel()->selectedRows();
    QCOMPARE(selectedRows.size(), 1);
    QCOMPARE(selectedRows.front().data().toString(), "ParentItem Sword");
}

QTEST_MAIN(MainWindowTest)

#include "tst_mainwindow.moc"
