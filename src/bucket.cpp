// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

#include "bucket.h"

#include <algorithm>
#include <numeric>

#include "locationinventory.h"
#include "modelprobes.h"
#include "util/fatalerror.h"

namespace {

    // Estimated heap bytes behind one QString: the shared-data header plus
    // UTF-16 payload. An estimate is enough — the gauge exists so tests can
    // assert residency transitions (built / evicted / rebuilt), and the
    // authoritative memory budgets are measured at process level (M1-M3).
    std::int64_t stringBytes(const QString &value)
    {
        if (value.isNull()) {
            return 0;
        }
        return static_cast<std::int64_t>(24 + 2 * value.size());
    }

    // Bytes one key uniquely accounts for. The D1 buffer sharing is
    // honored: on the base string path the head's s1/s2 share the suffix
    // PrettyName's buffer (or each other's), and a shared buffer counts
    // once. Deliberately a pure function of the key, so an entry rebuild
    // can return the old entry's bytes without the item in hand.
    std::int64_t keyBytes(const ItemSortKey &key)
    {
        std::int64_t bytes = sizeof(ItemSortKey);
        const QString &pretty = std::get<0>(key.suffix);
        bytes += stringBytes(pretty);
        bytes += stringBytes(std::get<1>(key.suffix));
        bytes += stringBytes(std::get<2>(key.suffix));
        if (const auto *base = std::get_if<ItemSortKey::BaseHead>(&key.head)) {
            const QString &s1 = std::get<1>(*base);
            const QString &s2 = std::get<3>(*base);
            if (!s1.isSharedWith(pretty)) {
                bytes += stringBytes(s1);
            }
            if (!s2.isSharedWith(pretty) && !s2.isSharedWith(s1)) {
                bytes += stringBytes(s2);
            }
        }
        return bytes;
    }

} // namespace

void ResidentKeyStore::Release()
{
    // The gauge is maintained unconditionally (unlike the counters): it
    // must balance across probe enable/disable boundaries or a test
    // enabling probes mid-residency would drive it negative at eviction.
    ModelProbes::instance().live_key_bytes -= bytes;
    // Swap-with-empty rather than clear()+shrink_to_fit(): shrink_to_fit
    // is a non-binding request, and "background searches hold exactly 0"
    // (R2-4) must not depend on the library honoring it.
    std::vector<ItemSortKey>().swap(keys);
    column = nullptr;
    bytes = 0;
}

Bucket::Bucket(const ItemLocation &location)
    : m_location(location)
{}

void Bucket::AddItem(const std::shared_ptr<Item> &item)
{
    m_items.push_back(item);
}

void Bucket::AddItems(const Items &items)
{
    m_items.reserve(m_items.size() + items.size());
    for (const auto &item : items) {
        m_items.push_back(item);
    }
}

// The A′ replace translation (S5 remedy): while ReplaceSourceRows emits
// its notification stream, the bucket's row queries answer through this
// window instead of a physically spliced vector. Two phases:
//
// - Removal phase, runs notified back-to-front: with runs
//   [low_removed, removal_count) logically removed, rows below the
//   lowest removed run are identity into the OLD vector; rows at or
//   above it are retained rows — mapped through the "all runs removed"
//   piecewise offsets, entered at t = row - removed_before[low_removed].
// - Insertion phase, runs notified front-to-back: with runs
//   [0, inserted_upto) logically inserted, rows below the next pending
//   run's final position are identity into the FINAL vector; rows at or
//   above it are retained rows still awaiting their shift — mapped by
//   adding the pending runs' lengths, entered at
//   t = row - inserted_before[inserted_upto].
//
// All arrays are precomputed once; a query is one branch plus at most
// one binary search over the run boundaries.
struct Bucket::ReplaceWindow
{
    struct Run
    {
        int first;
        int len;
    };

    const Items *old_items{nullptr};
    const Items *final_items{nullptr};
    int old_count{0};
    int retained_count{0};

    // Removal runs, ascending in old coordinates. removed_before[j] is
    // the total length of runs before run j (one extra trailing entry =
    // total removed); retained_at[j] = first - removed_before[j] is the
    // run's position in retained coordinates (strictly increasing).
    std::vector<Run> removals;
    std::vector<int> removed_before;
    std::vector<int> retained_at;

    // Insertion runs, ascending in final coordinates. inserted_before
    // and retained_insert_at are the symmetric arrays.
    std::vector<Run> insertions;
    std::vector<int> inserted_before;
    std::vector<int> retained_insert_at;

    bool inserting{false};
    int low_removed{0};   // removal phase: runs [low_removed, k) are out
    int inserted_upto{0}; // insertion phase: runs [0, inserted_upto) are in

    int size() const
    {
        if (!inserting) {
            const int total = removed_before[removals.size()];
            return old_count - (total - removed_before[static_cast<size_t>(low_removed)]);
        }
        return retained_count + inserted_before[static_cast<size_t>(inserted_upto)];
    }

    const std::shared_ptr<Item> &at(int row) const
    {
        if (!inserting) {
            const int k = static_cast<int>(removals.size());
            if ((low_removed == k) || (row < removals[static_cast<size_t>(low_removed)].first)) {
                return (*old_items)[static_cast<size_t>(row)];
            }
            const int t = row - removed_before[static_cast<size_t>(low_removed)];
            // Largest run j (>= low_removed) with retained_at[j] <= t.
            const auto begin = retained_at.begin() + low_removed;
            const auto it = std::upper_bound(begin, retained_at.end(), t);
            const auto j = static_cast<size_t>(std::distance(retained_at.begin(), it)) - 1;
            return (*old_items)[static_cast<size_t>(t + removed_before[j + 1])];
        }
        const int l = static_cast<int>(insertions.size());
        if ((inserted_upto == l) || (row < insertions[static_cast<size_t>(inserted_upto)].first)) {
            return (*final_items)[static_cast<size_t>(row)];
        }
        const int t = row - inserted_before[static_cast<size_t>(inserted_upto)];
        // Largest pending run m (>= inserted_upto) with
        // retained_insert_at[m] <= t.
        const auto begin = retained_insert_at.begin() + inserted_upto;
        const auto it = std::upper_bound(begin, retained_insert_at.end(), t);
        const auto m = static_cast<size_t>(std::distance(retained_insert_at.begin(), it)) - 1;
        return (*final_items)[static_cast<size_t>(t + inserted_before[m + 1])];
    }
};

int Bucket::size() const
{
    if (m_replace_window) {
        return m_replace_window->size();
    }
    return static_cast<int>(m_items.size());
}

bool Bucket::has_item(int row) const
{
    return (row >= 0) && (row < size());
}

const std::shared_ptr<Item> &Bucket::item(int row) const
{
    const int item_count = size();
    if ((row < 0) || (row >= item_count)) {
        const QString message
            = QString("Bucket item row out of bounds: %1 item count: %2. Program will abort")
                  .arg(QString::number(row), QString::number(item_count));
        FatalError(message);
    }
    if (m_replace_window) {
        return m_replace_window->at(row);
    }
    return m_items[row];
}

void Bucket::HydrateKeys(const Column &column)
{
    if (m_keys.column == &column) {
        return; // resident for this column: reuse (R2-3)
    }
    // Keys resident for another column cannot order this sort; the eager
    // eviction on a column switch (D1 rule 2) normally precedes this, so
    // the Release here is a self-defense, not a path.
    m_keys.Release();

    if (auto &probes = ModelProbes::instance(); probes.enabled) {
        ++probes.key_builds;
        ++probes.key_builds_by_location[LocationInventory::KeyFor(m_location)];
    }
    m_keys.keys.reserve(m_items.size());
    std::int64_t bytes = 0;
    for (const auto &item : m_items) {
        m_keys.keys.push_back(column.key(*item));
        bytes += keyBytes(m_keys.keys.back());
    }
    m_keys.column = &column;
    m_keys.bytes = bytes;
    ModelProbes::instance().live_key_bytes += bytes;
}

void Bucket::Sort(const Column &column, Qt::SortOrder order)
{
    auto &probes = ModelProbes::instance();
    if (probes.enabled) {
        ++probes.bucket_sorts;
        ++probes.bucket_sorts_by_location[LocationInventory::KeyFor(m_location)];
    }

    // The M3 keyed sort (items-pipeline-m3.md D1/D5): every operation that
    // consumes keys hydrates missing ones first (R3-1), then a permutation
    // of the (key, item) pairs is ordered by plain tuple comparison and
    // both vectors adopt it — the keys stay resident, aligned with the new
    // item order.
    HydrateKeys(column);
    const auto &keys = m_keys.keys;
    std::vector<std::uint32_t> perm(m_items.size());
    std::iota(perm.begin(), perm.end(), 0);
    std::sort(perm.begin(),
              perm.end(),
              [&probes, &keys, order](const std::uint32_t lhs, const std::uint32_t rhs) {
                  if (probes.enabled) {
                      ++probes.keyed_compares;
                  }
                  if (order == Qt::AscendingOrder) {
                      return keys[lhs] < keys[rhs];
                  } else {
                      return keys[rhs] < keys[lhs];
                  }
              });

    // Apply the permutation in place, walking each cycle once and marking
    // spent entries as self-loops (S3 review round 1): materializing
    // sorted copies would transiently duplicate the key vector — ~144 B
    // per item, ~144 MB extra at a one-million-item By-Item sort, right
    // where the resident-key budget is tightest — and the gauge does not
    // (and should not) count temporaries. The permutation itself is the
    // only transient (4 B per item).
    for (std::uint32_t start = 0; start < static_cast<std::uint32_t>(perm.size()); ++start) {
        if (perm[start] == start) {
            continue;
        }
        std::shared_ptr<Item> lifted_item = std::move(m_items[start]);
        ItemSortKey lifted_key = std::move(m_keys.keys[start]);
        std::uint32_t n = start;
        while (true) {
            const std::uint32_t from = perm[n];
            perm[n] = n;
            if (from == start) {
                m_items[n] = std::move(lifted_item);
                m_keys.keys[n] = std::move(lifted_key);
                break;
            }
            m_items[n] = std::move(m_items[from]);
            m_keys.keys[n] = std::move(m_keys.keys[from]);
            n = from;
        }
    }
    m_sorted = true;
}

void Bucket::RemoveRows(int first, int count)
{
    if ((first < 0) || (count <= 0) || (first + count > static_cast<int>(m_items.size()))) {
        FatalError(QString("Bucket::RemoveRows out of bounds: first %1 count %2 size %3")
                       .arg(QString::number(first),
                            QString::number(count),
                            QString::number(m_items.size())));
    }
    m_items.erase(m_items.begin() + first, m_items.begin() + first + count);
    if (m_keys.resident()) {
        std::int64_t delta = 0;
        for (int n = first; n < first + count; ++n) {
            delta -= keyBytes(m_keys.keys[static_cast<size_t>(n)]);
        }
        m_keys.keys.erase(m_keys.keys.begin() + first, m_keys.keys.begin() + first + count);
        m_keys.bytes += delta;
        ModelProbes::instance().live_key_bytes += delta;
    }
}

void Bucket::InsertRows(int first, const Items &items, const std::vector<ItemSortKey> *keys)
{
    if ((first < 0) || (first > static_cast<int>(m_items.size()))) {
        FatalError(QString("Bucket::InsertRows out of bounds: first %1 size %2")
                       .arg(QString::number(first), QString::number(m_items.size())));
    }
    m_items.insert(m_items.begin() + first, items.begin(), items.end());
    if (m_keys.resident()) {
        if (keys && (keys->size() == items.size())) {
            std::int64_t delta = 0;
            for (const auto &key : *keys) {
                delta += keyBytes(key);
            }
            m_keys.keys.insert(m_keys.keys.begin() + first, keys->begin(), keys->end());
            m_keys.bytes += delta;
            ModelProbes::instance().live_key_bytes += delta;
        } else {
            // No aligned keys for the arrivals: residency cannot be kept
            // honest, so evict; the next key-consuming event rehydrates.
            m_keys.Release();
        }
    }
}

Items Bucket::ReplaceSourceRows(const std::function<bool(const Item &)> &predicate,
                                const Items &arrivals,
                                const Column *column,
                                Qt::SortOrder order,
                                const std::function<void(int, int)> &begin_remove,
                                const std::function<void()> &end_remove,
                                const std::function<void(int, int)> &begin_insert,
                                const std::function<void()> &end_insert)
{
    auto &probes = ModelProbes::instance();
    const int old_count = static_cast<int>(m_items.size());

    ReplaceWindow win;
    win.old_items = &m_items;
    win.old_count = old_count;

    // 1. One scan finds the removal runs and the removed items.
    Items removed;
    for (int n = 0; n < old_count; ++n) {
        if (predicate(*m_items[static_cast<size_t>(n)])) {
            removed.push_back(m_items[static_cast<size_t>(n)]);
            if (!win.removals.empty()
                && (win.removals.back().first + win.removals.back().len == n)) {
                ++win.removals.back().len;
            } else {
                win.removals.push_back({n, 1});
            }
        }
    }
    const int removed_count = static_cast<int>(removed.size());
    const int retained_count = old_count - removed_count;
    win.retained_count = retained_count;
    win.removed_before.reserve(win.removals.size() + 1);
    win.retained_at.reserve(win.removals.size());
    int removed_so_far = 0;
    for (const auto &run : win.removals) {
        win.removed_before.push_back(removed_so_far);
        win.retained_at.push_back(run.first - removed_so_far);
        removed_so_far += run.len;
    }
    win.removed_before.push_back(removed_so_far);

    if (removed.empty() && arrivals.empty()) {
        return removed; // no visible change; nothing to notify or rebuild
    }

    // Retained coordinate -> old coordinate. Every caller below walks t
    // monotonically, so a run cursor makes this amortized O(1).
    size_t next_removal = 0;
    int removal_offset = 0;
    const auto retained_to_old = [&win, &next_removal, &removal_offset](int t) {
        while ((next_removal < win.removals.size()) && (win.retained_at[next_removal] <= t)) {
            removal_offset += win.removals[next_removal].len;
            ++next_removal;
        }
        return t + removal_offset;
    };

    // 2. Compact the resident keys in place, dropping the removed
    //    entries — retained keys end up index-aligned with the retained
    //    order, at no transient allocation (the S3 in-place discipline).
    //    An append (arrivals with no merge column) cannot keep alignment
    //    and evicts instead.
    const bool merge = (column != nullptr) && !arrivals.empty();
    if (merge) {
        HydrateKeys(*column); // R3-1: normally resident already (eager activation)
    }
    if (m_keys.resident()) {
        if (!arrivals.empty() && !merge) {
            m_keys.Release();
        } else if (removed_count > 0) {
            std::int64_t delta = 0;
            int dest = 0;
            size_t run = 0;
            for (int n = 0; n < old_count; ++n) {
                if ((run < win.removals.size()) && (n >= win.removals[run].first)
                    && (n < win.removals[run].first + win.removals[run].len)) {
                    delta -= keyBytes(m_keys.keys[static_cast<size_t>(n)]);
                    if (n == win.removals[run].first + win.removals[run].len - 1) {
                        ++run;
                    }
                    continue;
                }
                if (dest != n) {
                    m_keys.keys[static_cast<size_t>(dest)] = std::move(
                        m_keys.keys[static_cast<size_t>(n)]);
                }
                ++dest;
            }
            m_keys.keys.resize(static_cast<size_t>(retained_count));
            m_keys.bytes += delta;
            ModelProbes::instance().live_key_bytes += delta;
        }
    }

    // 3. Sort the arrivals by key and compute their insertion runs
    //    against the retained order (upper_bound over the compacted
    //    retained keys — positions nondecreasing since the arrivals are
    //    sorted), or one appended run when there is no merge column.
    std::vector<std::pair<ItemSortKey, std::shared_ptr<Item>>> sorted_arrivals;
    if (merge) {
        const Column &col = *column;
        const auto less = [&probes, order](const ItemSortKey &lhs, const ItemSortKey &rhs) {
            if (probes.enabled) {
                ++probes.keyed_compares;
            }
            return (order == Qt::AscendingOrder) ? (lhs < rhs) : (rhs < lhs);
        };
        if (probes.enabled) {
            // The merge is this bucket's order-refresh event
            // (staleOrderNeverSurvivesDelta).
            ++probes.bucket_sorts;
            ++probes.bucket_sorts_by_location[LocationInventory::KeyFor(m_location)];
        }
        sorted_arrivals.reserve(arrivals.size());
        for (const auto &item : arrivals) {
            sorted_arrivals.emplace_back(col.key(*item), item);
        }
        std::sort(sorted_arrivals.begin(),
                  sorted_arrivals.end(),
                  [&less](const auto &lhs, const auto &rhs) { return less(lhs.first, rhs.first); });

        const auto &retained_keys = m_keys.keys;
        int inserted_so_far = 0;
        int last_position = 0;
        for (size_t a = 0; a < sorted_arrivals.size();) {
            const int position = static_cast<int>(
                std::upper_bound(retained_keys.begin() + last_position,
                                 retained_keys.end(),
                                 sorted_arrivals[a].first,
                                 less)
                - retained_keys.begin());
            size_t run_end = a + 1;
            while ((run_end < sorted_arrivals.size())
                   && (static_cast<int>(std::upper_bound(retained_keys.begin() + position,
                                                         retained_keys.end(),
                                                         sorted_arrivals[run_end].first,
                                                         less)
                                        - retained_keys.begin())
                       == position)) {
                ++run_end;
            }
            win.insertions.push_back({position + inserted_so_far, static_cast<int>(run_end - a)});
            inserted_so_far += static_cast<int>(run_end - a);
            last_position = position;
            a = run_end;
        }
    } else if (!arrivals.empty()) {
        win.insertions.push_back({retained_count, static_cast<int>(arrivals.size())});
    }
    win.inserted_before.reserve(win.insertions.size() + 1);
    win.retained_insert_at.reserve(win.insertions.size());
    int inserted_total = 0;
    for (const auto &run : win.insertions) {
        win.inserted_before.push_back(inserted_total);
        win.retained_insert_at.push_back(run.first - inserted_total);
        inserted_total += run.len;
    }
    win.inserted_before.push_back(inserted_total);

    // 4. Removal notifications, back-to-front in current coordinates —
    //    the old vector stays intact and answers the queries.
    m_replace_window = &win;
    win.inserting = false;
    win.low_removed = static_cast<int>(win.removals.size());
    for (int j = static_cast<int>(win.removals.size()) - 1; j >= 0; --j) {
        const auto &run = win.removals[static_cast<size_t>(j)];
        begin_remove(run.first, run.first + run.len - 1);
        win.low_removed = j;
        end_remove();
    }

    // 5. Materialize the final order. Removal-only replacements compact
    //    the item vector in place; anything with arrivals builds the
    //    final vector by MOVING the retained rows out of the old one —
    //    the removal phase was the old order's last consumer, and moves
    //    keep the rebuild free of collection-scale refcount traffic.
    Items final_items;
    if (win.insertions.empty()) {
        int dest = 0;
        for (int t = 0; t < retained_count; ++t, ++dest) {
            const int src = retained_to_old(t);
            if (src != dest) {
                m_items[static_cast<size_t>(dest)] = std::move(m_items[static_cast<size_t>(src)]);
            }
        }
        m_items.resize(static_cast<size_t>(retained_count));
        m_replace_window = nullptr;
        return removed;
    }
    final_items.reserve(static_cast<size_t>(retained_count) + arrivals.size());
    {
        const int final_count = retained_count + inserted_total;
        size_t next_run = 0;
        size_t arrival_src = 0;
        int t = 0;
        while (static_cast<int>(final_items.size()) < final_count) {
            if ((next_run < win.insertions.size())
                && (static_cast<int>(final_items.size()) == win.insertions[next_run].first)) {
                const int len = win.insertions[next_run].len;
                for (int n = 0; n < len; ++n, ++arrival_src) {
                    final_items.push_back(merge ? sorted_arrivals[arrival_src].second
                                                : arrivals[arrival_src]);
                }
                ++next_run;
            } else {
                final_items.push_back(std::move(m_items[static_cast<size_t>(retained_to_old(t))]));
                ++t;
            }
        }
    }
    win.final_items = &final_items;

    // 6. Insertion notifications, forward — a pending run's current
    //    insertion point equals its final start, because every earlier
    //    run is already in; the final vector answers the queries.
    win.inserting = true;
    win.inserted_upto = 0;
    for (size_t i = 0; i < win.insertions.size(); ++i) {
        const auto &run = win.insertions[i];
        begin_insert(run.first, run.first + run.len - 1);
        win.inserted_upto = static_cast<int>(i) + 1;
        end_insert();
    }
    m_replace_window = nullptr;

    // 7. Commit. The final key vector is realized in place by a backward
    //    merge of the compacted retained keys with the arrival keys —
    //    replaying the recorded runs, no comparisons, and no second
    //    collection-sized key buffer (the A′ memory gate); growth beyond
    //    capacity is the one case that still reallocates.
    m_items = std::move(final_items);
    if (m_keys.resident() && merge) {
        std::int64_t delta = 0;
        for (const auto &arrival : sorted_arrivals) {
            delta += keyBytes(arrival.first);
        }
        m_keys.keys.resize(m_items.size());
        int dest = static_cast<int>(m_items.size()) - 1;
        int retained_src = retained_count - 1;
        int run = static_cast<int>(win.insertions.size()) - 1;
        size_t arrival_src = sorted_arrivals.size();
        while (dest >= 0) {
            if ((run >= 0)
                && (dest < win.insertions[static_cast<size_t>(run)].first
                               + win.insertions[static_cast<size_t>(run)].len)
                && (dest >= win.insertions[static_cast<size_t>(run)].first)) {
                m_keys.keys[static_cast<size_t>(dest)] = std::move(
                    sorted_arrivals[--arrival_src].first);
                if (dest == win.insertions[static_cast<size_t>(run)].first) {
                    --run;
                }
            } else {
                if (dest != retained_src) {
                    m_keys.keys[static_cast<size_t>(dest)] = std::move(
                        m_keys.keys[static_cast<size_t>(retained_src)]);
                }
                --retained_src;
            }
            --dest;
        }
        m_keys.bytes += delta;
        ModelProbes::instance().live_key_bytes += delta;
    }
    if (!arrivals.empty() && !merge) {
        // Appended arrival-ordered: the order defers to the next
        // key-consuming event (D2).
        InvalidateOrder();
    }
    return removed;
}

void Bucket::RebuildKeyEntries(const Column &column, const std::set<QString> &ids, bool everything)
{
    if (!m_keys.resident()) {
        return; // no keys, nothing stale (a collapsed bucket stays keyless)
    }
    if (m_keys.column != &column) {
        // Resident for another column: the entries cannot be patched into
        // validity — evict, and the next key-consuming event hydrates.
        m_keys.Release();
        return;
    }
    std::int64_t delta = 0;
    for (size_t n = 0; n < m_items.size(); ++n) {
        if (everything || (ids.count(m_items[n]->id()) > 0)) {
            delta -= keyBytes(m_keys.keys[n]);
            m_keys.keys[n] = column.key(*m_items[n]);
            delta += keyBytes(m_keys.keys[n]);
        }
    }
    m_keys.bytes += delta;
    ModelProbes::instance().live_key_bytes += delta;
}
