// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

#pragma once

#include <cstdint>
#include <set>
#include <vector>

#include "column.h"
#include "item.h"
#include "itemlocation.h"
#include "sortkey.h"

// The bucket's resident sort keys (items-pipeline-m3.md D1): index-aligned
// with the bucket's items, derived from exactly one column. `column` doubles
// as the residency marker — null means evicted. The store owns its share of
// the live-key-bytes gauge (ModelProbes::live_key_bytes) and releases it on
// eviction, move, and destruction, so the gauge balances however a bucket
// dies (refilter rebuild, search deletion, mode switch).
struct ResidentKeyStore
{
    std::vector<ItemSortKey> keys;
    const Column *column{nullptr};
    std::int64_t bytes{0};

    ResidentKeyStore() = default;
    ResidentKeyStore(const ResidentKeyStore &) = delete;
    ResidentKeyStore &operator=(const ResidentKeyStore &) = delete;
    ResidentKeyStore(ResidentKeyStore &&other) noexcept
        : keys(std::move(other.keys))
        , column(other.column)
        , bytes(other.bytes)
    {
        other.keys.clear();
        other.column = nullptr;
        other.bytes = 0;
    }
    ResidentKeyStore &operator=(ResidentKeyStore &&other) noexcept
    {
        if (this != &other) {
            Release();
            keys = std::move(other.keys);
            column = other.column;
            bytes = other.bytes;
            other.keys.clear();
            other.column = nullptr;
            other.bytes = 0;
        }
        return *this;
    }
    ~ResidentKeyStore() { Release(); }

    bool resident() const { return column != nullptr; }
    // Evicts the keys and returns their bytes to the gauge.
    void Release();
};

// A bucket holds set of filtered items.
// Items are "bucketed" by their location: stash tab / character.
class Bucket
{
public:
    Bucket() = default;
    explicit Bucket(const ItemLocation &location);

    // Non-copyable
    Bucket(const Bucket &) = delete;
    Bucket &operator=(const Bucket &) = delete;

    // Moveable
    Bucket(Bucket &&) = default;
    Bucket &operator=(Bucket &&) = default;

    void AddItem(const std::shared_ptr<Item> &item);
    void AddItems(const Items &items);
    const Items &items() const { return m_items; }
    bool has_item(int row) const;
    const std::shared_ptr<Item> &item(int row) const;
    const ItemLocation &location() const { return m_location; }

    // Sorts unconditionally on this column's keys — hydrating the resident
    // vector if evicted (R3-1), reusing it otherwise (R2-3) — and sets the
    // sorted flag. The D2 skip-when-valid lives in the callers (Search,
    // ItemsModel), which own flag validity against the model's current
    // (column, order).
    void Sort(const Column &column, Qt::SortOrder order);

    // D2's per-bucket sorted-validity flag. Sorted-order validity and key
    // residency are independent axes (R3-1): sorted-but-keyless is a
    // legitimate state, entered by expanding with a valid flag or by
    // collapse/deactivation eviction.
    bool sorted() const { return m_sorted; }
    void InvalidateOrder() { m_sorted = false; }

    // The live materialization mark (D1: a materialized bucket is an
    // expanded By-Tab bucket; the By-Item flat bucket is materialized by
    // definition and never consults this). Maintained by the view's
    // expand/collapse signals and by FilterItems for default-expanded
    // searches.
    bool expanded() const { return m_expanded; }
    void SetExpanded(bool expanded) { m_expanded = expanded; }

    // Collapse/deactivation eviction (D1/R2-3/R2-4): keys leave, order and
    // flag persist.
    void EvictKeys() { m_keys.Release(); }
    bool hasResidentKeys() const { return m_keys.resident(); }

    // The buyout key effect (R3-2): rebuild the resident entries of the
    // affected items — every entry when `everything` — so a batch's
    // re-sort never runs on stale resident keys. No-op when evicted (a
    // collapsed bucket's keys stay absent); keys resident for another
    // column are evicted instead of patched.
    void RebuildKeyEntries(const Column &column, const std::set<QString> &ids, bool everything);

private:
    void HydrateKeys(const Column &column);

    Items m_items;
    ItemLocation m_location;
    ResidentKeyStore m_keys;
    bool m_sorted{false};
    bool m_expanded{false};
};
