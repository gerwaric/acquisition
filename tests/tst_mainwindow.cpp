// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#include <QtTest/QtTest>

#include <QAbstractItemModel>
#include <QComboBox>
#include <QHeaderView>
#include <QItemSelectionModel>
#include <QLabel>
#include <QLineEdit>
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

    // Items-pipeline M2, stage 3: D6 stable-identity bucketing.
    void bucketsKeyOnStableIdDuringRefresh();

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

QTEST_MAIN(MainWindowTest)

#include "tst_mainwindow.moc"
