// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#pragma once

#include <algorithm>
#include <map>

#include "fetchsourcekey.h"
#include "item.h"

// Source-keyed item storage (items-pipeline M2, D3). The M2-M2 measurement
// fired the spec's storage conditional at both scales: four structurally
// identical O(all-items) erase passes per reply were ~99% of the measured
// path. Both sides of the presentation lane (worker and ItemsManager) hold
// their items in this store instead of a flat vector — buckets keyed by
// FetchSourceKey, plus a lazily rebuilt flat vector for whole-collection
// consumers. Per-reply operations (ReplaceSource, EraseSourcesIf) touch
// only the affected buckets or walk the bucket index, never all items;
// Flat() rebuilds run on snapshot/tick paths only, never per reply.
//
// Invariant: every bucket is non-empty and homogeneous — its items were all
// parsed under one fetch source, so they share type(), fetch_id(), and the
// stable id() (a child bucket's id() is its display parent's). An emptied
// source erases its bucket outright, which keeps "set of items" semantics
// identical to the flat representation this replaces. Predicates that used
// to test every item's location therefore test one representative per
// bucket.
class SourceKeyedItems
{
public:
    // Snapshot boundary: regroup everything. The given vector becomes the
    // flat cache as-is, so a ResetTo followed by Flat() preserves the
    // caller's ordering without a rebuild.
    void ResetTo(Items items)
    {
        m_buckets.clear();
        for (const auto &item : items) {
            m_buckets[FetchSourceKey::ForLocation(item->location())].push_back(item);
        }
        m_flat = std::move(items);
        m_flat_dirty = false;
        m_size = m_flat.size();
    }

    // The per-reply atomic replacement: everything previously fetched by
    // this source is dropped and the delta takes its place. O(old bucket +
    // delta). Returns the number of items replaced (for the callers' logs).
    size_t ReplaceSource(const FetchSourceKey &key, Items items)
    {
        size_t removed = 0;
        const auto it = m_buckets.find(key);
        if (it != m_buckets.end()) {
            removed = it->second.size();
            m_size -= removed;
            if (items.empty()) {
                m_buckets.erase(it);
            } else {
                m_size += items.size();
                it->second = std::move(items);
            }
            m_flat_dirty = true;
        } else if (!items.empty()) {
            m_size += items.size();
            m_buckets.emplace(key, std::move(items));
            m_flat_dirty = true;
        }
        return removed;
    }

    // Erase whole buckets matching pred(key, representative location) — the
    // representative stands for every item in the bucket (see the
    // homogeneity invariant above). O(sources) plus the erased items.
    // Returns the number of items erased.
    template<typename Pred>
    size_t EraseSourcesIf(Pred pred)
    {
        size_t erased = 0;
        for (auto it = m_buckets.begin(); it != m_buckets.end();) {
            if (pred(it->first, it->second.front()->location())) {
                erased += it->second.size();
                it = m_buckets.erase(it);
            } else {
                ++it;
            }
        }
        if (erased > 0) {
            m_size -= erased;
            m_flat_dirty = true;
        }
        return erased;
    }

    // All items, concatenated in bucket-key order (or the order the last
    // ResetTo/SortFlat established, while no mutation has intervened).
    // Rebuilt lazily after a mutation: snapshot and tick consumers only —
    // nothing on the per-reply path may call this.
    const Items &Flat() const
    {
        if (m_flat_dirty) {
            m_flat.clear();
            m_flat.reserve(m_size);
            for (const auto &[key, bucket] : m_buckets) {
                m_flat.insert(m_flat.end(), bucket.begin(), bucket.end());
            }
            m_flat_dirty = false;
        }
        return m_flat;
    }

    // In-place sort of the flat view (FinishUpdate's deterministic
    // presentation order). The cache stays valid: sorting permutes the same
    // set of items the buckets hold.
    template<typename Comp>
    void SortFlat(Comp comp)
    {
        Flat();
        std::sort(m_flat.begin(), m_flat.end(), comp);
    }

    // Bucket iteration for whole-collection maintenance (location rebasing
    // at the snapshot boundary). Items are shared_ptrs, so the pointees stay
    // mutable through the const view; the bucket structure does not.
    const std::map<FetchSourceKey, Items> &buckets() const { return m_buckets; }

    size_t size() const { return m_size; }

private:
    std::map<FetchSourceKey, Items> m_buckets;
    // The lazily rebuilt whole-collection view (D3): mutable so Flat() can
    // service const readers like ItemsManager::items().
    mutable Items m_flat;
    mutable bool m_flat_dirty{false};
    size_t m_size{0};
};
