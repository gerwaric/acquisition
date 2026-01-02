// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2025 Tom Holz

#pragma once

#include <QString>

#include <optional>
#include <tuple>
#include <vector>

struct LegacyItem
{
    struct Socket
    {
        unsigned group;
        std::optional<QString> attr;
    };

    struct Property
    {
        QString name;
        std::vector<std::tuple<QString, int>> values;
    };

    struct HybridInfo
    {
        std::optional<bool> isVaalGem;
        QString baseTypeName;
    };

    // This is just enough of the item to calculate the legacy buyout hash.

    QString id;
    std::optional<std::vector<LegacyItem::Socket>> sockets;
    QString name;
    QString typeLine;
    std::optional<std::vector<LegacyItem::Property>> properties;
    std::optional<std::vector<LegacyItem::Property>> additionalProperties;
    std::optional<std::vector<QString>> implicitMods;
    std::optional<std::vector<QString>> explicitMods;
    std::optional<LegacyItem::HybridInfo> hybrid;
    std::optional<QString> _character;
    std::optional<QString> _tab_label;

    // Duplicate the way acquisition handles typeLine for vaal gems.
    QString effectiveTypeLine() const;

    // Duplicate acquisition's old item hashing function.
    QString hash() const;
};
