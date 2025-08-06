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

#include <QObject>
#include <QString>

#include <map>

#include <util/spdlog_qt.h>

namespace ItemEnums {
    Q_NAMESPACE

    enum FrameType {
        FRAME_TYPE_NORMAL = 0,
        FRAME_TYPE_MAGIC = 1,
        FRAME_TYPE_RARE = 2,
        FRAME_TYPE_UNIQUE = 3,
        FRAME_TYPE_GEM = 4,
        FRAME_TYPE_CURRENCY = 5,
        FRAME_TYPE_DIVINATION_CARD = 6,
        FRAME_TYPE_QUEST_ITEM = 7,
        FRAME_TYPE_PROPHECY = 8,
        FRAME_TYPE_RELIC = 9
    };
    Q_ENUM_NS(FrameType)

    enum ElementalDamageType {
        ED_FIRE = 4,
        ED_COLD = 5,
        ED_LIGHTNING = 6,
    };
    Q_ENUM_NS(ElementalDamageType)

} // namespace ItemEnums

using FrameType = ItemEnums::FrameType;
template<>
struct fmt::formatter<FrameType, char> : QtEnumFormatter<FrameType>
{};

using ElementalDamageType = ItemEnums::ElementalDamageType;
template<>
struct fmt::formatter<ElementalDamageType, char> : QtEnumFormatter<ElementalDamageType>
{};

constexpr int PIXELS_PER_SLOT = 47;
constexpr int INVENTORY_SLOTS = 12;
constexpr int PIXELS_PER_MINIMAP_SLOT = 10;
constexpr int MINIMAP_SIZE = INVENTORY_SLOTS * PIXELS_PER_MINIMAP_SLOT;

struct position
{
    double x;
    double y;
};

const std::map<QString, position> &POS_MAP();
