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

    // https://www.pathofexile.com/developer/docs/reference#type-CrucibleNode

    struct CrucibleNode {
        std::optional<unsigned> skill; // ? uint	mod hash
        std::optional<unsigned> tier; // ? uint	mod tier
        std::optional<QString> icon; // ? string
        std::optional<bool> allocated; // ? bool	always true if present
        std::optional<bool> isNotable; // ? bool	always true if present
        std::optional<bool> isReward; // ? bool	always true if present
        std::optional<std::vector<QString>> stats; // ? array of string	stat descriptions
        std::optional<std::vector<QString>> reminderText; // ? array of string
        std::optional<unsigned> orbit; // ? uint	the column this node occupies
        std::optional<unsigned> orbitIndex; // ? uint	the node's position within the column
        std::vector<QString> out; //	array of string	node identifiers of nodes this one connects to
        std::vector<QString> in; //	array of string	node identifiers of nodes connected to this one
    };

}
