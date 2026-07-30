// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

#include "bucket.h"

#include "locationinventory.h"
#include "modelprobes.h"
#include "util/fatalerror.h"

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

void Bucket::Sort(const Column &column, Qt::SortOrder order)
{
    auto &probes = ModelProbes::instance();
    if (probes.enabled) {
        ++probes.bucket_sorts;
        ++probes.bucket_sorts_by_location[LocationInventory::KeyFor(m_location)];
    }

    // The M3 keyed sort (items-pipeline-m3.md D1/D5): materialize the
    // comparator's tuple once per item, order (key, item) pairs by plain
    // tuple comparison, adopt the item order. S1 semantics: keys are
    // built at each sort and discarded — residency/caching lands in S3,
    // so a key can never outlive the sort that built it.
    std::vector<std::pair<ItemSortKey, std::shared_ptr<Item>>> keyed;
    keyed.reserve(m_items.size());
    for (const auto &item : m_items) {
        keyed.emplace_back(column.key(*item), item);
    }
    if (probes.enabled) {
        ++probes.key_builds;
        ++probes.key_builds_by_location[LocationInventory::KeyFor(m_location)];
    }

    std::sort(keyed.begin(), keyed.end(), [&probes, order](const auto &lhs, const auto &rhs) {
        if (probes.enabled) {
            ++probes.keyed_compares;
        }
        if (order == Qt::AscendingOrder) {
            return lhs.first < rhs.first;
        } else {
            return rhs.first < lhs.first;
        }
    });

    for (size_t n = 0; n < keyed.size(); ++n) {
        m_items[n] = std::move(keyed[n].second);
    }
}
