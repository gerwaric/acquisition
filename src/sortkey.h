// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#pragma once

#include <QDateTime>
#include <QString>

#include <tuple>
#include <variant>

// The M3 sort key (items-pipeline-m3.md D1): the comparator's own tuple,
// materialized once per item by Column::key. The head is the column
// family's comparator tuple; the suffix is the (PrettyName, uid, hash)
// tie-break in D5's intended order. Keys order by plain tuple comparison,
// so a keyed sort reproduces the comparator's order without paying the
// regex/QVariant work per comparison (keyedOrderMatchesComparatorOrder is
// the equivalence pin).
struct ItemSortKey
{
    // One alternative per comparator family. Every key in a single sort is
    // built by the same column, so the variant index never decides an
    // ordering between items.
    using BaseHead = std::tuple<double, QString, double, QString>; // Column::multivalue
    using PriceHead = std::tuple<int, double>;                     // (currency rank, value)
    using DateHead = std::tuple<QDateTime>;                        // (last_update)
    using Suffix = std::tuple<QString, QString, QString>;          // (PrettyName, uid, hash)

    std::variant<BaseHead, PriceHead, DateHead> head;
    Suffix suffix;

    bool operator<(const ItemSortKey &rhs) const
    {
        return std::tie(head, suffix) < std::tie(rhs.head, rhs.suffix);
    }
};
