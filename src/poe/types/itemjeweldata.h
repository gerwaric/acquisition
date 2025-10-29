/*
    Copyright (C) 2024-2025 Gerwaric

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

#include <optional>
#include <unordered_map>

#include <QString>

#include "poe/types/passivegroup.h"
#include "poe/types/passivenode.h"
#include "util/glaze_qt.h"

static_assert(ACQUISITION_USE_GLAZE);

namespace poe {

    // https://www.pathofexile.com/developer/docs/reference#type-ItemJewelData

    struct ItemJewelData
    {
        struct Subgraph
        {
            std::unordered_map<QString, poe::PassiveGroup>
                groups; // dictionary of PassiveGroup the key is the string value of the group id
            std::unordered_map<QString, poe::PassiveNode>
                nodes; // dictionary of PassiveNode the key is the string value of the node identifier
        };

        QString type;                                         // string
        std::optional<unsigned> radius;                       // ? uint
        std::optional<unsigned> radiusMin;                    // ? uint
        std::optional<QString> radiusVisual;                  // ? string
        std::optional<poe::ItemJewelData::Subgraph> subgraph; // ? object only present on cluster jewels
    };

} // namespace poe
