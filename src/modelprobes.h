// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#pragma once

#include <cstdint>
#include <map>

#include "locationinventory.h"

// The M3 probe surface (items-pipeline M3, S0): cheap counters the model
// layer increments and only tests read. Test-observable without logging,
// per the M2 test-access convention; production code never reads these.
// Everything here lives on the UI thread, like the code that increments
// it, so plain integers suffice.
//
// Disabled by default: with `enabled` false every site is one predicted
// branch and no state accumulates — no per-location map growth over a
// process lifetime, no retained addresses of deleted models, and no
// per-comparison increments inside timed measurement windows (the M1-M3
// harness keeps probes disabled unless a row needs attribution). Tests
// set `enabled` in their init, call reset(), and assert deltas; they
// must scope per-model assertions to models they hold alive, since a
// freed model's address can be reused within an enabled window.
//
// Fields whose production call sites land in later stages are declared
// now so the pins can assert them from zero; each names its landing
// stage.
struct ModelProbes
{
    bool enabled = false;

    // Sites live today (S0):
    std::int64_t comparator_calls = 0; // Column::lt / PriceColumn::lt / DateColumn::lt entries
    std::int64_t bucket_sorts = 0;     // Bucket::Sort entries
    // Bucket::Sort entries keyed by the bucket's stable (type, id) display
    // key; the By-Item flat bucket reports KeyFor(ItemLocation{}).
    std::map<LocationInventory::Key, std::int64_t> bucket_sorts_by_location;
    std::int64_t model_resets = 0; // ItemsModel::beginUpdate (beginResetModel)
    std::map<const void *, std::int64_t> model_resets_by_model; // keyed by the ItemsModel
    std::int64_t refilters = 0;          // Search::FilterItems runs past the TabChanged gate
    std::int64_t index_rebuilds = 0;     // whole-collection visible-index rebuilds (FilterItems)
    std::int64_t expansion_captures = 0; // MainWindow::SaveViewExpansion
    std::int64_t expansion_restores = 0; // MainWindow::RestoreViewExpansion
    std::int64_t scroll_captures = 0;    // MainWindow::SaveViewScroll
    std::int64_t scroll_restores = 0;    // MainWindow::RestoreViewScroll
    std::int64_t reselects = 0;          // MainWindow::ReselectCurrentItem

    // Sites land in S1 (D5 + keyed sorting):
    std::int64_t key_builds = 0; // key-vector builds, keyed like bucket sorts
    std::map<LocationInventory::Key, std::int64_t> key_builds_by_location;
    std::int64_t keyed_compares = 0; // tuple comparisons inside a keyed sort

    // Site lands in S2 (buyout choke point + batching): batched
    // reorder/model-update emissions at outer batch boundaries.
    std::int64_t model_updates = 0;

    // Gauge, not a counter; sites land in S3 (D1 residency): bytes of
    // resident sort keys, adjusted at hydration and eviction.
    std::int64_t live_key_bytes = 0;

    static ModelProbes &instance()
    {
        static ModelProbes probes;
        return probes;
    }

    // Clears the counters. `enabled` and the live_key_bytes gauge
    // survive: the gauge tracks live key ownership (S3 residency), so a
    // test's counter reset must not falsify it — zeroing it while keys
    // are resident would drive it negative at the next eviction.
    void reset()
    {
        const bool was_enabled = enabled;
        const std::int64_t live = live_key_bytes;
        *this = ModelProbes();
        enabled = was_enabled;
        live_key_bytes = live;
    }
};
