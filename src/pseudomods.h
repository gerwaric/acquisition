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

#include <QHashFunctions> // Hash functions seem to be needed for std::unordered_map<QString,*>
#include <QString>

#include <unordered_map>
#include <vector>

using PseudoModMap = std::unordered_map<QString, std::vector<QString>>;

class PseudoModManager
{
public:
    // These are just summed, and the mod named as the first element of a vector is generated with value equaling the sum.
    // Both implicit and explicit fields are considered.
    // This is pretty much the same list as poe.trade uses
    static const PseudoModMap SUMMING_MODS;
    static const PseudoModMap SUMMING_MODS_LOOKUP;

private:
    static PseudoModMap reverseMap(const PseudoModMap &map);
};
