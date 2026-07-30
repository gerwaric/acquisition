#include <QtTest/QtTest>

#include <QSet>

#include "itemcategories.h"
#include "spikedataset.h"

// Pins the properties M2 relies on from the dataset generator: the same
// (config, seed) reproduces the same collection and churn sequence (M2-M2's
// fixed recorded shapes, R7-3), and churn breaks pointer identity while
// stable item identity survives (what D6/R6-3 key on).
class SpikeDatasetTest : public QObject
{
    Q_OBJECT

private slots:
    void initTestCase();
    void sameSeedReproducesCollectionAndChurn();
    void churnPreservesStableIdentityNotPointers();
};

static SpikeDataset::Config smallConfig()
{
    SpikeDataset::Config config;
    config.tab_count = 20;
    config.mean_items_per_tab = 30;
    return config;
}

static QStringList itemIds(const Items &items)
{
    QStringList ids;
    ids.reserve(items.size());
    for (const auto &item : items) {
        ids.push_back(item->id());
    }
    return ids;
}

void SpikeDatasetTest::initTestCase()
{
    // Item construction reads the process-global category tables.
    InitItemClasses(R"json({"TestClass":{"name":"Weapons"}})json");
    InitItemBaseTypes(
        R"json({"Metadata/Items/TestSword":{"item_class":"TestClass","name":"Test Sword","release_state":"released"}})json");
}

void SpikeDatasetTest::sameSeedReproducesCollectionAndChurn()
{
    SpikeDataset a(smallConfig());
    SpikeDataset b(smallConfig());

    QCOMPARE(a.tabCount(), 20);
    QCOMPARE(b.totalItems(), a.totalItems());
    QCOMPARE(static_cast<qsizetype>(a.allItems().size()), a.totalItems());
    for (int t = 0; t < a.tabCount(); ++t) {
        QCOMPARE(b.tabName(t), a.tabName(t));
        QCOMPARE(itemIds(b.tabItems(t)), itemIds(a.tabItems(t)));
    }

    // The churn sequence is part of the reproducible shape, not just the
    // starting collection.
    QCOMPARE(itemIds(b.ChurnTab(3, 0.5)), itemIds(a.ChurnTab(3, 0.5)));
    QCOMPARE(itemIds(b.ChurnTab(3, 0.5)), itemIds(a.ChurnTab(3, 0.5)));
}

void SpikeDatasetTest::churnPreservesStableIdentityNotPointers()
{
    SpikeDataset dataset(smallConfig());
    const int tab = 5;

    const Items before = dataset.tabItems(tab);
    const QStringList before_id_list = itemIds(before);
    const QSet<QString> before_ids(before_id_list.begin(), before_id_list.end());
    const Items after = dataset.ChurnTab(tab, 0.5);
    const QStringList after_id_list = itemIds(after);
    const QSet<QString> after_ids(after_id_list.begin(), after_id_list.end());

    // Removals are matched by arrivals, so the tab size is unchanged.
    QCOMPARE(after.size(), before.size());

    // Kept and modified items keep their ids; arrivals are genuinely new.
    const auto surviving = after_ids & before_ids;
    const auto arrivals = after_ids - before_ids;
    QVERIFY(!surviving.isEmpty());
    QVERIFY(!arrivals.isEmpty());
    QCOMPARE(surviving.size() + arrivals.size(), after_ids.size());

    // Every materialized Item is a new object, even for kept specs.
    for (const auto &new_item : after) {
        for (const auto &old_item : before) {
            QVERIFY(new_item.get() != old_item.get());
        }
    }
}

QTEST_GUILESS_MAIN(SpikeDatasetTest)

#include "tst_spikedataset.moc"
