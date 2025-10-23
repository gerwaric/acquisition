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

#include <QString>

#include <optional>
#include <vector>

namespace poe {
    
    // https://www.pathofexile.com/developer/docs/reference#type-PassiveGroup

    struct PassiveGroup {

        float x; // float
        float y; // float
        std::vector<unsigned> orbits; //	array of uint
        std::optional<bool> isProxy; // ? bool always true if present
        std::optional<QString> proxy; // ? string identifier of the placeholder node
        std::vector<QString> nodes; // array of string the node identifiers associated with this group;
    };

}
