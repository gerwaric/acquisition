// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

#include "bucket.h"

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

bool Bucket::has_item(int row) const
{
    return (row >= 0) && (row < static_cast<int>(m_items.size()));
}

const std::shared_ptr<Item> &Bucket::item(int row) const
{
    const int item_count = static_cast<int>(m_items.size());
    if ((row < 0) || (row >= item_count)) {
        const QString message
            = QString("Bucket item row out of bounds: %1 item count: %2. Program will abort")
                  .arg(QString::number(row), QString::number(item_count));
        FatalError(message);
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
