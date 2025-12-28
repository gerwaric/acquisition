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

#include "pseudomods.h"

// These are just summed, and the mod named as the first element of a vector is generated with value equaling the sum.
// Both implicit and explicit fields are considered.
// This is pretty much the same list as poe.trade uses

// clang-format off
const PseudoModMap PseudoModManager::SUMMING_MODS = {
    {
        "+#% total to Cold Resistance",
        {
            "+#% to Cold Resistance",
            "+#% to Fire and Cold Resistances",
            "+#% to Cold and Lightning Resistances",
            "+#% to Cold and Chaos Resistances",
            "+#% to all Elemental Resistances"
        }
    },
    {
        "+#% total to Fire Resistance",
        {
            "+#% to Fire Resistance",
            "+#% to Fire and Cold Resistances",
            "+#% to Fire and Lightning Resistances",
            "+#% to Fire and Chaos Resistances",
            "+#% to all Elemental Resistances"
        }
    },
    {
        "+#% total to Lightning Resistance",
        {
            "+#% to Lightning Resistance",
            "+#% to Fire and Lightning Resistances",
            "+#% to Cold and Lightning Resistances",
            "+#% to Lightning and Chaos Resistances",
            "+#% to all Elemental Resistances"
        }
    },
    {
        "+#% total Elemental Resistance",
        {
            // "+#% total to Fire Resistance":
            "+#% to Fire Resistance",
            "+#% to Fire and Cold Resistances",
            "+#% to Fire and Lightning Resistances",
            "+#% to Fire and Chaos Resistances",
            "+#% to all Elemental Resistances",
            // "+#% total to Cold Resistance":
            "+#% to Cold Resistance",
            "+#% to Fire and Cold Resistances",
            "+#% to Cold and Lightning Resistances",
            "+#% to Cold and Chaos Resistances",
            "+#% to all Elemental Resistances",
            // "+#% total to Lightning Resistance":
            "+#% to Lightning Resistance",
            "+#% to Fire and Lightning Resistances",
            "+#% to Cold and Lightning Resistances",
            "+#% to Lightning and Chaos Resistances",
            "+#% to all Elemental Resistances"
        }
    },
    {
        "+#% total to Chaos Resistance",
        {
            "+#% to Chaos Resistance",
            "+#% to Fire and Chaos Resistances",
            "+#% to Cold and Chaos Resistances",
            "+#% to Lightning and Chaos Resistances"
        }
    },
    {
        "+#% total Resistance",
        {
            // "+#% total to Fire Resistance":
            "+#% to Fire Resistance",
            "+#% to Fire and Cold Resistances",
            "+#% to Fire and Lightning Resistances",
            "+#% to Fire and Chaos Resistances",
            "+#% to all Elemental Resistances",
            // "+#% total to Cold Resistance":
            "+#% to Cold Resistance",
            "+#% to Fire and Cold Resistances",
            "+#% to Cold and Lightning Resistances",
            "+#% to Cold and Chaos Resistances",
            "+#% to all Elemental Resistances",
            // "+#% total to Lightning Resistance":
            "+#% to Lightning Resistance",
            "+#% to Fire and Lightning Resistances",
            "+#% to Cold and Lightning Resistances",
            "+#% to Lightning and Chaos Resistances",
            "+#% to all Elemental Resistances",
            //"+#% total to Chaos Resistance":
            "+#% to Chaos Resistance",
            "+#% to Fire and Chaos Resistances",
            "+#% to Cold and Chaos Resistances",
            "+#% to Lightning and Chaos Resistances"
        }
    },
    {
        "+# total to Strength",
        {
            "+# to Strength",
            "+# to Strength and Dexterity",
            "+# to Strength and Intelligence",
            "+# to all Attributes"
        }
    },
    {
        "+# total to Dexterity",
        {
            "+# to Dexterity",
            "+# to Strength and Dexterity",
            "+# to Dexterity and Intelligence",
            "+# to all Attributes"
        }
    },
    {
        "+# total to Intelligence",
        {
            "+# to Intelligence",
            "+# to Strength and Intelligence",
            "+# to Dexterity and Intelligence",
            "+# to all Attributes"
        }
    },
    {
        "+#% total Attack Speed",
        {
            "#% increased Attack Speed"
        }
    },
    {
        "+#% total Cast Speed",
        {
            "#% increased Cast Speed"
        }
    },
    {
        "+#% total increased Physical Damage",
        {
            "#% increased Physical Damage",
            "#% increased Global Physical Damage"
        }
    },
    {
        "+#% total Critical Strike Chance for Spells",
        {
            "#% increased Global Critical Strike Chance",
            "#% increased Spell Critical Strike Chance"
        }
    },
    {
        "+# total to Level of Socketed Gems",
        {
            "+# to Level of Socketed Gems"
        }
    },
    {
        "+# total to Level of Socketed Elemental Gems",
        {
            "+# to Level of Socketed Elemental Gems",
            "+# to Level of Socketed Gems"
        }
    },
    {
        "+# total to Level of Socketed Fire Gems",
        {
            "+# to Level of Socketed Fire Gems",
            "+# to Level of Socketed Elemental Gems",
            "+# to Level of Socketed Gems"
        }
    },
    {
        "+# total to Level of Socketed Cold Gems",
        {
            "+# to Level of Socketed Cold Gems",
            "+# to Level of Socketed Elemental Gems",
            "+# to Level of Socketed Gems"
        }
    },
    {
        "+# total to Level of Socketed Lightning Gems",
        {
            "+# to Level of Socketed Lightning Gems",
            "+# to Level of Socketed Elemental Gems",
            "+# to Level of Socketed Gems"
        }
    },
    {
        "+# total to Level of Socketed Chaos Gems",
        {
            "+# to Level of Socketed Chaos Gems",
            "+# to Level of Socketed Gems"
        }
    },
    {
        "+# total to Level of Socketed Spell Gems",
        {
            "+# to Level of Socketed Spell Gems",
            "+# to Level of Socketed Gems"
        }
    },
    {
        "+# total to Level of Socketed Projectile Gems",
        {
            "+# to Level of Socketed Projectile Gems",
            "+# to Level of Socketed Gems"
        }
    },
    {
        "+# total to Level of Socketed Bow Gems",
        {
            "+# to Level of Socketed Bow Gems",
            "+# to Level of Socketed Gems"
        }
    },
    {
        "+# total to Level of Socketed Melee Gems",
        {
            "+# to Level of Socketed Melee Gems",
            "+# to Level of Socketed Gems"
        }
    },
    {
        "+# total to Level of Socketed Minion Gems",
        {
            "+# to Level of Socketed Minion Gems",
            "+# to Level of Socketed Gems"
        }
    },
    {
        "+# total to Level of Socketed Strength Gems",
        {
            "+# to Level of Socketed Strength Gems",
            "+# to Level of Socketed Gems"
        }
    },
    {
        "+# total to Level of Socketed Deterity Gems",
        {
            "+# to Level of Socketed Deterity Gems",
            "+# to Level of Socketed Gems"
        }
    },
    {
        "+# total to Level of Socketed Intelligence Gems",
        {
            "+# to Level of Socketed Intelligence Gems",
            "+# to Level of Socketed Gems"
        }
    },
    {
        "+# total to Level of Socketed Aura Gems",
        {
            "+# to Level of Socketed Aura Gems",
            "+# to Level of Socketed Gems"
        }
        },
    {
        "+# total to Level of Socketed Movement Gems",
        {
            "+# to Level of Socketed Movement Gems",
            "+# to Level of Socketed Gems"
        }
    },
    {
        "+# total to Level of Socketed Curse Gems", {
            "+# to Level of Socketed Curse Gems",
            "+# to Level of Socketed Gems"
        }
    },
    {
        "+# total to Level of Socketed Vaal Gems",
        {
            "+# to Level of Socketed Vaal Gems",
            "+# to Level of Socketed Gems"
        }
    },
    {
        "+# total to Level of Socketed Support Gems",
        {
            "+# to Level of Socketed Support Gems",
            "+# to Level of Socketed Gems"
        }
    },
    {
        "+# total to Level of Socketed Skill Gems",
        {
            "+# to Level of Socketed Skill Gems",
            "+# to Level of Socketed Gems"
        }
    },
    {
        "+# total to Level of Socketed Warcry Gems",
        {
            "+# to Level of Socketed Warcry Gems",
            "+# to Level of Socketed Gems"
        }
    },
    {
        "+# total to Level of Socketed Golem Gems",
        {
            "+# to Level of Socketed Golem Gems",
            "+# to Level of Socketed Gems"
        }
    }
};
// clang-format on

PseudoModMap PseudoModManager::reverseMap(const PseudoModMap &map)
{
    PseudoModMap reverse_map;
    for (const auto &[pseudo_mod, real_mods] : map) {
        for (const auto &real_mod : real_mods) {
            reverse_map[real_mod].push_back(pseudo_mod);
        }
    }
    return reverse_map;
}

const PseudoModMap PseudoModManager::SUMMING_MODS_LOOKUP = PseudoModManager::reverseMap(
    PseudoModManager::SUMMING_MODS);
