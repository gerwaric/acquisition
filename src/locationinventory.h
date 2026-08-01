// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#pragma once

#include <QString>

#include <map>
#include <utility>
#include <vector>

#include "itemlocation.h"

// The canonical-location inventory behind M2's stable-identity bucket rule
// (items-pipeline M2, D6/R5-1/R6-1): the freshest tab metadata the UI has
// seen per stable (type, id) display key. Every delta's location anchor is
// ingested — empty deltas included — a delta's location supersedes embedded
// item metadata, and the final snapshot supersedes everything (ResetTo,
// which is also where tab deletions take effect; deletion never streams).
class LocationInventory
{
public:
    using Key = std::pair<ItemLocationType, QString>;

    static Key KeyFor(const ItemLocation &location) { return {location.type(), location.id()}; }

    void Ingest(const ItemLocation &location) { m_locations[KeyFor(location)] = location; }

    void ResetTo(const std::vector<ItemLocation> &tabs)
    {
        m_locations.clear();
        for (const auto &tab : tabs) {
            m_locations[KeyFor(tab)] = tab;
        }
    }

    // The freshest location seen for the given location's stable display
    // key; falls back to the location itself for keys never seen.
    const ItemLocation &Canonical(const ItemLocation &embedded) const
    {
        const auto it = m_locations.find(KeyFor(embedded));
        return (it != m_locations.end()) ? it->second : embedded;
    }

    // Every known location, for the unfiltered empty-bucket source list.
    const std::map<Key, ItemLocation> &entries() const { return m_locations; }

private:
    std::map<Key, ItemLocation> m_locations;
};
