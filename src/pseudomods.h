// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2025 Tom Holz

#pragma once

#include <QHashFunctions> // Needed for std::unordered_map<QString, ...>
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
