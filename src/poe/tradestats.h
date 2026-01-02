// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2025 Tom Holz

#pragma once

#include <QString>

#include <optional>

namespace poe {

    struct TradeStatOption
    {
        int id;
        QString text;
    };

    struct TradeStatOptionsWrapper
    {
        std::vector<poe::TradeStatOption> options;
    };

    struct TradeStatEntry
    {
        QString id;
        QString text;
        QString type;
        std::optional<poe::TradeStatOptionsWrapper> option;
    };

    struct TradeStatSet
    {
        QString id;
        QString label;
        std::vector<poe::TradeStatEntry> entries;
    };

    struct TradeStatsWrapper
    {
        std::vector<poe::TradeStatSet> result;
    };

}
