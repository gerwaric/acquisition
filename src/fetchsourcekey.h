// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#pragma once

#include <QString>

#include <tuple>

#include "itemlocation.h"

// The key that M2's delta replacements are applied by, used identically on
// both sides of the presentation-lane signals (items-pipeline M2, D3/R1-3):
// the worker's per-reply erase and the published copy's erase share this one
// predicate, so they can never diverge on a cross-type id collision.
// ItemLocation equality deliberately ignores fetch_id, so location equality
// must never stand in for this key. Scope (R6-5): the typed key governs
// exactly the predicates M2 introduces — the replacement erases and the
// child reconciliation; legacy bare-id stores are recorded in F66.
struct FetchSourceKey
{
    ItemLocationType type;
    QString fetch_id;

    static FetchSourceKey ForLocation(const ItemLocation &location)
    {
        return {location.type(), location.fetch_id()};
    }

    bool operator==(const FetchSourceKey &other) const = default;
    bool operator<(const FetchSourceKey &other) const
    {
        return std::tie(type, fetch_id) < std::tie(other.type, other.fetch_id);
    }
};
