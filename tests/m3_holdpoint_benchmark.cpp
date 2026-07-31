// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

// M3 hold-point harness (items-pipeline M3, implementation sequence,
// working rule 3): runs the conditional budget rows for the stages that
// have one, against the real MainWindow/Search/ItemsModel path on the
// recorded spike presets. Not a test: run by hand, offscreen, in a
// Release build:
//
//   ./m3_holdpoint_benchmark --preset 100k
//   ./m3_holdpoint_benchmark --preset 1m
//
// S3 rows (this file's initial scope): worst-case unfiltered By-Tab
// refilter (default collapsed, sort share attributed); single-bucket
// expand with cold keys; broad-filter default-expanded refilter (R1-8's
// worst case, driven by the ilvl >= 2 filter, which matches ~99% of the
// dataset while still excluding items); collapsed-default and background
// resident-key memory. S4 row: delta application on the current search,
// By-Tab visible bucket (full source replacement of the largest expanded
// bucket — removal runs plus a maximal merge). By-Item rows wait for
// S5's machinery.
//
// Attribution follows the M2-M2 discipline: the live windows are timed
// end-to-end, sort/key work is attributed by the model probes (counts)
// and by micro-benchmarks on identical data outside the timed windows
// (filter loop, key build + sort at collection scale).

#include <QApplication>
#include <QComboBox>
#include <QCommandLineParser>
#include <QElapsedTimer>
#include <QHeaderView>
#include <QLabel>
#include <QLineEdit>
#include <QTabBar>
#include <QTreeView>

#include <algorithm>
#include <cstdio>
#include <map>
#include <memory>
#include <vector>

#include <spdlog/sinks/dist_sink.h>
#include <spdlog/spdlog.h>

#include "bucket.h"
#include "column.h"
#include "filters/filterspec.h"
#include "filters/filterstate.h"
#include "mainwindowfixture.h"
#include "modelprobes.h"
#include "search.h"
#include "spikedataset.h"

namespace {

    double toMs(qint64 ns)
    {
        return static_cast<double>(ns) / 1e6;
    }

    qint64 median(std::vector<qint64> values)
    {
        if (values.empty()) {
            return 0;
        }
        std::sort(values.begin(), values.end());
        return values[values.size() / 2];
    }

    QLineEdit *findMinMaxMin(MainWindow &window, const QString &caption)
    {
        for (auto *label : window.findChildren<QLabel *>()) {
            if (label->text() != caption) {
                continue;
            }
            auto *group = label->parentWidget();
            if (!group) {
                continue;
            }
            const auto edits = group->findChildren<QLineEdit *>();
            if (!edits.isEmpty()) {
                return edits.front(); // min precedes max in the form layout
            }
        }
        return nullptr;
    }

    QTabBar *findSearchTabs(MainWindow &window)
    {
        for (auto *tabs : window.findChildren<QTabBar *>()) {
            if ((tabs->count() > 0) && (tabs->tabText(tabs->count() - 1) == "+")) {
                return tabs;
            }
        }
        return nullptr;
    }

    void drainEvents()
    {
        for (int n = 0; n < 8; ++n) {
            QCoreApplication::processEvents(QEventLoop::AllEvents);
        }
    }

    struct BudgetRow
    {
        const char *name;
        double measured_ms;
        double budget_ms; // < 0: informational, no budget at this preset
    };

    void printRows(const std::vector<BudgetRow> &rows)
    {
        bool missed = false;
        for (const auto &row : rows) {
            if (row.budget_ms < 0) {
                std::printf("  %-46s %10.3f ms   (informational)\n", row.name, row.measured_ms);
            } else {
                const bool pass = row.measured_ms <= row.budget_ms;
                missed = missed || !pass;
                std::printf("  %-46s %10.3f ms   budget %8.1f ms   %s\n",
                            row.name,
                            row.measured_ms,
                            row.budget_ms,
                            pass ? "PASS" : "MISS");
            }
        }
        if (missed) {
            std::printf("  *** at least one row MISSED its budget ***\n");
        }
    }

} // namespace

int main(int argc, char *argv[])
{
    if (qEnvironmentVariableIsEmpty("QT_QPA_PLATFORM")) {
        qputenv("QT_QPA_PLATFORM", "offscreen");
    }
    QApplication app(argc, argv);
    QElapsedTimer clock;
    clock.start();

    QCommandLineParser parser;
    const QCommandLineOption preset_option("preset",
                                           "Dataset preset: smoke, 100k, or 1m.",
                                           "preset",
                                           "100k");
    parser.addOption(preset_option);
    parser.process(app);

    const QString preset_name = parser.value(preset_option);
    const auto preset = SpikeDataset::Config::Preset(preset_name);
    if (!preset) {
        std::fprintf(stderr, "unknown preset: %s\n", qPrintable(preset_name));
        return 1;
    }
    // The S3 budget table (items-pipeline-m3.md acceptance criteria);
    // smoke is functional-only.
    const bool is_100k = (preset_name == "100k");
    const bool is_1m = (preset_name == "1m");
    const double budget_refilter_ms = is_100k ? 60.0 : is_1m ? 500.0 : -1.0;
    const double budget_sort_share_ms = (is_100k || is_1m) ? 5.0 : -1.0;
    const double budget_expand_ms = (is_100k || is_1m) ? 10.0 : -1.0;
    const double budget_broad_ms = is_100k ? 150.0 : is_1m ? 1200.0 : -1.0;
    // The S4 row's budget is stated at 1m only; 100k is informational.
    const double budget_delta_ms = is_1m ? 5.0 : -1.0;

    auto main_logger = std::make_shared<spdlog::logger>("main");
    main_logger->sinks().push_back(std::make_shared<spdlog::sinks::dist_sink_mt>());
    spdlog::register_logger(main_logger);
    spdlog::set_level(spdlog::level::warn);

    // The dataset materializes Items, which consult the process-global
    // category tables the fixture would otherwise initialize later.
    InitializeMainWindowTestCategories();

    std::printf("M3 hold-point harness: building dataset preset %s...\n", qPrintable(preset_name));
    SpikeDataset dataset(*preset);
    Items all_items = dataset.allItems();
    std::vector<ItemLocation> locations;
    locations.reserve(static_cast<size_t>(dataset.tabCount()));
    for (int t = 0; t < dataset.tabCount(); ++t) {
        locations.push_back(dataset.location(t));
    }
    std::printf("  %d tabs, %zu items\n", dataset.tabCount(), all_items.size());

    MainWindowFixture fixture;
    auto *tree = fixture.window->findChild<QTreeView *>("treeView");
    auto *tabs = findSearchTabs(*fixture.window);
    auto *ilvl_min = findMinMaxMin(*fixture.window, "ilvl");
    if (!tree || !tabs || !ilvl_min) {
        std::fprintf(stderr, "fixture widgets not found\n");
        return 1;
    }

    qint64 t0 = clock.nsecsElapsed();
    fixture.itemsManager->OnItemsRefreshed(all_items, locations, false);
    drainEvents();
    std::printf("  initial publish + refilter: %.1f ms\n", toMs(clock.nsecsElapsed() - t0));

    auto &probes = ModelProbes::instance();
    std::vector<BudgetRow> rows;

    // --- Row 1: worst-case unfiltered By-Tab refilter (default collapsed) --
    {
        std::vector<qint64> samples;
        for (int rep = 0; rep < 5; ++rep) {
            t0 = clock.nsecsElapsed();
            fixture.window->OnSearchFormChange();
            samples.push_back(clock.nsecsElapsed() - t0);
            drainEvents();
        }
        rows.push_back(
            {"unfiltered By-Tab refilter (median of 5)", toMs(median(samples)), budget_refilter_ms});

        // Sort-share attribution: the collapsed-default refilter must sort
        // nothing and build no keys (D2) — the probes prove the share is
        // literally zero.
        probes.reset();
        probes.enabled = true;
        fixture.window->OnSearchFormChange();
        probes.enabled = false;
        std::printf("  [attribution] unfiltered refilter: bucket_sorts=%lld key_builds=%lld "
                    "keyed_compares=%lld\n",
                    static_cast<long long>(probes.bucket_sorts),
                    static_cast<long long>(probes.key_builds),
                    static_cast<long long>(probes.keyed_compares));
        rows.push_back({"  sort share (probe-attributed)",
                        (probes.bucket_sorts == 0 && probes.key_builds == 0) ? 0.0 : 1e9,
                        budget_sort_share_ms});
        drainEvents();

        // Memory row: the collapsed-default view holds no resident keys.
        std::printf("  [memory] collapsed-default resident key bytes: %lld (row: == 0)%s\n",
                    static_cast<long long>(probes.live_key_bytes),
                    probes.live_key_bytes == 0 ? "  PASS" : "  MISS");

        // Filter-loop micro on identical data, outside the live window: a
        // bare Search's FilterItems is the loop + bucketing + index rebuild
        // without the view machinery.
        const FilterCatalog catalog = BuildFilterCatalog(*fixture.buyoutFixture.manager);
        Search bare(*fixture.buyoutFixture.manager, "micro", catalog);
        t0 = clock.nsecsElapsed();
        bare.FilterItems(all_items);
        std::printf("  [micro] bare FilterItems (no active filter): %.3f ms\n",
                    toMs(clock.nsecsElapsed() - t0));
    }

    // --- Row 2: single-bucket expand, cold keys ---------------------------
    {
        // The largest buckets are the honest worst case (quad tabs, up to
        // 576 items). Each rep refilters first so the expanded bucket's
        // keys are cold, then times the expand alone.
        std::vector<qint64> samples;
        int expanded_items = 0;
        for (int rep = 0; rep < 5; ++rep) {
            // Collapse everything first: a still-expanded bucket would be
            // saved and restored by the refilter (sorting it there), and
            // the timed expand() below would then be a signal-less no-op.
            for (int row = 0; row < tree->model()->rowCount(); ++row) {
                tree->collapse(tree->model()->index(row, 0));
            }
            fixture.window->OnSearchFormChange();
            drainEvents();
            auto *model = tree->model();
            int best_row = 0;
            int best_count = -1;
            for (int row = 0; row < model->rowCount(); ++row) {
                const int count = model->rowCount(model->index(row, 0));
                if (count > best_count) {
                    best_count = count;
                    best_row = row;
                }
            }
            expanded_items = best_count;
            const QModelIndex bucket = model->index(best_row, 0);
            t0 = clock.nsecsElapsed();
            tree->expand(bucket);
            samples.push_back(clock.nsecsElapsed() - t0);
            drainEvents();
        }
        std::printf("  [shape] expanded bucket item count: %d\n", expanded_items);
        rows.push_back({"single-bucket expand, cold keys (median of 5)",
                        toMs(median(samples)),
                        budget_expand_ms});
    }

    // --- Row 3: broad-filter default-expanded refilter (R1-8) -------------
    {
        ilvl_min->setText("2"); // matches ~85/86 of items; excludes some -> m_filtered
        std::vector<qint64> samples;
        for (int rep = 0; rep < 3; ++rep) {
            t0 = clock.nsecsElapsed();
            fixture.window->OnSearchFormChange();
            samples.push_back(clock.nsecsElapsed() - t0);
            drainEvents();
        }
        rows.push_back({"broad-filter default-expanded refilter (median)",
                        toMs(median(samples)),
                        budget_broad_ms});
        std::printf(
            "  [memory] broad-filter resident key bytes: %lld (~worst materialized shape)\n",
            static_cast<long long>(probes.live_key_bytes));

        // Attribution run + micros on identical data.
        probes.reset();
        probes.enabled = true;
        fixture.window->OnSearchFormChange();
        probes.enabled = false;
        std::printf("  [attribution] broad refilter: bucket_sorts=%lld key_builds=%lld "
                    "keyed_compares=%lld\n",
                    static_cast<long long>(probes.bucket_sorts),
                    static_cast<long long>(probes.key_builds),
                    static_cast<long long>(probes.keyed_compares));
        drainEvents();

        const FilterCatalog catalog = BuildFilterCatalog(*fixture.buyoutFixture.manager);
        Search bare(*fixture.buyoutFixture.manager, "micro", catalog);
        for (qsizetype n = 0; n < catalog.size(); ++n) {
            if (catalog[n].caption == "ilvl") {
                bare.setFilterState(n, MinMaxState{2.0, std::nullopt});
                break;
            }
        }
        t0 = clock.nsecsElapsed();
        bare.FilterItems(all_items);
        const qint64 micro_filter = clock.nsecsElapsed() - t0;

        Bucket flat{ItemLocation()};
        flat.AddItems(bare.items());
        NameColumn name_column;
        t0 = clock.nsecsElapsed();
        flat.Sort(name_column, Qt::DescendingOrder);
        const qint64 micro_sort = clock.nsecsElapsed() - t0;
        std::printf("  [micro] broad FilterItems: %.3f ms; key build + sort of the visible "
                    "result (one flat bucket): %.3f ms\n",
                    toMs(micro_filter),
                    toMs(micro_sort));

        ilvl_min->clear();
        fixture.window->OnSearchFormChange();
        drainEvents();
    }

    // --- Row 4: background resident-key memory (exactly 0) ----------------
    {
        // Materialize something in Search 1, then create/switch to Search 2:
        // deactivation must evict every resident vector (R2-4).
        auto *model = tree->model();
        tree->expand(model->index(0, 0));
        const std::int64_t while_active = probes.live_key_bytes;
        tabs->setCurrentIndex(1);
        drainEvents();
        std::printf("  [memory] background resident key bytes: %lld after deactivation "
                    "(was %lld active) (row: == 0)%s\n",
                    static_cast<long long>(probes.live_key_bytes),
                    static_cast<long long>(while_active),
                    probes.live_key_bytes == 0 ? "  PASS" : "  MISS");
    }

    // --- Row 5 (S4): delta application, By-Tab visible bucket -------------
    {
        // Back on Search 1, clean state, largest bucket expanded: the
        // delta lands on the current search's visible bucket.
        tabs->setCurrentIndex(0);
        fixture.window->OnSearchFormChange();
        drainEvents();
        auto *model = tree->model();
        int best_row = 0;
        int best_count = -1;
        for (int row = 0; row < model->rowCount(); ++row) {
            const int count = model->rowCount(model->index(row, 0));
            if (count > best_count) {
                best_count = count;
                best_row = row;
            }
        }
        tree->expand(model->index(best_row, 0));
        drainEvents();

        // The delta is a full replacement of the largest tab's fetch
        // source — the honest worst case: removal runs for every row plus
        // a maximal merge into the visible order.
        std::map<LocationInventory::Key, int> counts;
        for (const auto &item : all_items) {
            ++counts[LocationInventory::KeyFor(item->location())];
        }
        int best_tab = 0;
        int best_tab_count = -1;
        for (int t = 0; t < dataset.tabCount(); ++t) {
            const int count = counts[LocationInventory::KeyFor(dataset.location(t))];
            if (count > best_tab_count) {
                best_tab_count = count;
                best_tab = t;
            }
        }
        const ItemLocation target = dataset.location(best_tab);
        const auto target_key = LocationInventory::KeyFor(target);
        Items delta;
        delta.reserve(static_cast<size_t>(best_tab_count));
        for (const auto &item : all_items) {
            if (LocationInventory::KeyFor(item->location()) == target_key) {
                delta.push_back(item);
            }
        }
        // Make sure the delta's bucket is the expanded one: expand it too
        // (idempotent when best_row already covers it).
        for (int row = 0; row < model->rowCount(); ++row) {
            if (model->index(row, 0).data().toString().startsWith(target.GetHeader())) {
                tree->expand(model->index(row, 0));
                break;
            }
        }
        drainEvents();

        std::printf("  [shape] delta item count: %zu\n", delta.size());
        std::vector<qint64> samples;
        for (int rep = 0; rep < 5; ++rep) {
            t0 = clock.nsecsElapsed();
            fixture.window->OnTabRefreshed(target, delta);
            samples.push_back(clock.nsecsElapsed() - t0);
            drainEvents();
        }
        rows.push_back(
            {"S4 delta application, By-Tab visible bucket", toMs(median(samples)), budget_delta_ms});

        probes.reset();
        probes.enabled = true;
        fixture.window->OnTabRefreshed(target, delta);
        probes.enabled = false;
        std::printf("  [attribution] delta application: bucket_sorts=%lld key_builds=%lld "
                    "keyed_compares=%lld index_rebuilds=%lld refilters=%lld model_resets=%lld\n",
                    static_cast<long long>(probes.bucket_sorts),
                    static_cast<long long>(probes.key_builds),
                    static_cast<long long>(probes.keyed_compares),
                    static_cast<long long>(probes.index_rebuilds),
                    static_cast<long long>(probes.refilters),
                    static_cast<long long>(probes.model_resets));
        drainEvents();
    }

    std::printf("\n=== M3 S3+S4 hold-point result: preset %s, %d tabs, %zu items, Qt %s ===\n",
                qPrintable(preset_name),
                dataset.tabCount(),
                all_items.size(),
                qVersion());
    printRows(rows);
    return 0;
}
