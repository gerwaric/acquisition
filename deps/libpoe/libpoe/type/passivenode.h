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

#include <json_struct/json_struct_qt.h>

#include <optional>
#include <vector>

namespace libpoe{
    
    // https://www.pathofexile.com/developer/docs/reference#type-PassiveNode

    struct PassiveNode {

        struct Mastery {
            unsigned effect; // uint effect hash
            std::vector<QString> stats; // array of string stat descriptions
            std::optional<std::vector<QString>> reminderText; // ? array of string
            JS_OBJ(effect, stats, reminderText);
        };

        struct ClusterJewel {
            std::optional<unsigned> size; // ? uint
            std::optional<unsigned> index; // ? uint
            std::optional<unsigned> proxy; // ? uint the proxy node identifier
            std::optional<unsigned> parent; // ? uint the parent node identifier
            JS_OBJ(size, index, proxy, parent);
        };

        std::optional<unsigned> skill; // ? uint skill hash
        std::optional<QString> name; // ? string
        std::optional<QString> icon; // ? string
        std::optional<bool> isKeystone; // ? bool always true if present
        std::optional<bool> isNotable; // ? bool always true if present
        std::optional<bool> isMastery; // ? bool always true if present
        std::optional<QString> inactiveIcon; // ? string inactive mastery image
        std::optional<QString> activeIcon; // ? string active mastery image
        std::optional<QString> activeEffectImage; // ? string active mastery or tattoo background image
        std::optional<std::vector<libpoe::PassiveNode::Mastery>> masteryEffects; // ? array of object
        std::optional<bool> isBlighted; // ? bool always true if present
        std::optional<bool> isTattoo; // ? bool always true if present
        std::optional<bool> isProxy; // ? bool always true if present
        std::optional<bool> isJewelSocket; // ? bool always true if present
        std::optional<libpoe::PassiveNode::ClusterJewel> expansionJewel; // ? object cluster jewel information
        std::optional<std::vector<QString>> recipe; // ? array of string components required for Blight crafting this node.
        // each string is one of ClearOil, SepiaOil, AmberOil, VerdantOil,
        // TealOil, AzureOil, IndigoOil, VioletOil, CrimsonOil,
        // BlackOil, OpalescentOil, SilverOil, GoldenOil, or PrismaticOil
        std::optional<unsigned> grantedStrength; // ? uint sum of stats on this node that grant strength
        std::optional<unsigned> grantedDexterity; // ? uint sum of stats on this node that grant dexterity
        std::optional<unsigned> grantedIntelligence; // ? uint sum of stats on this node that grant intelligence
        std::optional<QString> ascendancyName; // ? string
        std::optional<bool> isAscendancyStart; // ? bool always true if present
        std::optional<bool> isMultipleChoice; // ? bool always true if present
        std::optional<bool> isMultipleChoiceOption; // ? bool always true if present
        std::optional<unsigned> grantedPassivePoints; // ? uint
        std::optional<std::vector<QString>> stats; // ? array of string stat descriptions
        std::optional<std::vector<QString>> reminderText; // ? array of string
        std::optional<std::vector<QString>> flavourText; // ? array of string
        std::optional<unsigned> classStartIndex; // ? uint
        std::optional<QString> group; // ? string the key value to look up in the groups table
        std::optional<unsigned> orbit; // ? uint the orbit this node occupies within it's group
        std::optional<unsigned> orbitIndex; // ? uint the index of this node in the group's orbit
        std::vector<QString> out; // array of string node identifiers of nodes this one connects to
        std::vector<QString> in; // array of string node identifiers of nodes connected to this one

        JS_OBJ(skill, name, icon, isKeystone, isNotable, isMastery,
            inactiveIcon, activeIcon, activeEffectImage, masteryEffects,
            isBlighted, isTattoo, isProxy, isJewelSocket, expansionJewel, recipe,
            grantedStrength, grantedDexterity, grantedIntelligence, ascendancyName,
            isAscendancyStart, isMultipleChoice, isMultipleChoiceOption, grantedPassivePoints,
            stats, reminderText, flavourText, classStartIndex, group, orbit, orbitIndex, out, in);

    };

}
