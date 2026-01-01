/*
    Copyright (C) 2014-2025 Acquisition Contributors

    This file is part of Acquisition.

    Acquisition is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Acquisition is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Acquisition.  If not, see <http://www.gnu.org/licenses/>.
*/

#pragma once

#include <QString>

#include <optional>
#include <tuple>
#include <vector>

#include <poe/types/item.h>

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
