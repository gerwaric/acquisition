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
#include <vector>

#include <QString>

#include <glaze/glaze.hpp>

#include "poe/types/item.h"
#include "poe/types/itemjeweldata.h"
#include "poe/types/passivenode.h"

namespace poe {

    // https://www.pathofexile.com/developer/docs/reference#type-Character

    struct Character
    {
        struct Passives
        {
            std::vector<unsigned> hashes;    // array of uint
            std::vector<unsigned> hashes_ex; // array of uint
            std::optional<std::unordered_map<QString, int>>
                mastery_effects; // dictionary of int; PoE1 only; the key is the string value of the mastery node skill hash and the value is the selected effect hash
            std::optional<std::unordered_map<QString, std::vector<int>>>
                specializations; // dictionary of array of int; PoE2 only; the keys are set1, set2, and set3
            std::unordered_map<QString, poe::PassiveNode>
                skill_overrides; // dictionary of PassiveNode; the key is the string value of the node identifier being replaced
            std::optional<QString>
                bandit_choice; // ?string; PoE1 only; one of Kraityn, Alira, Oak, or Eramir
            std::optional<QString>
                pantheon_major; // ?string; PoE1 only; one of TheBrineKing, Arakaali, Solaris, or Lunaris
            std::optional<QString>
                pantheon_minor; // ?string; PoE1 only; one of Abberath, Gruthkul, Yugul, Shakari, Tukohama, Ralakesh, Garukhan, or Ryslatha
            std::unordered_map<QString, poe::ItemJewelData>
                jewel_data; // dictionary of ItemJewelData the key is the string value of the x property of an item from the jewels array in this request
            std::optional<QString>
                alternate_ascendancy; // ?string; PoE1 only; Warden, Warlock, or Primalist (deprecated)
        };

        struct Metadata
        {
            std::optional<QString> version; // ?string game; version for the character's realm
        };

        QString id;                    // string; a unique 64 digit hexadecimal string
        QString name;                  // string
        QString realm;                 // string; pc, xbox, or sony
        QString class_;                // string
        std::optional<QString> league; // ?string
        unsigned level;                // uint
        unsigned experience;           // uint
        std::optional<bool> ruthless;  // ?bool; always true if present; PoE1 only
        std::optional<bool> expired;   // ?bool; always true if present
        std::optional<bool> deleted;   // ?bool; always true if present
        std::optional<bool> current;   // ?bool; always true if present
        std::optional<std::vector<poe::Item>> equipment;  // ?array of Item
        std::optional<std::vector<poe::Item>> skills;     // ?array of Item; PoE2 only
        std::optional<std::vector<poe::Item>> inventory;  // ?array of Item
        std::optional<std::vector<poe::Item>> rucksack;   // ?array of Item
        std::optional<std::vector<poe::Item>> jewels;     // ?array of Item
        std::optional<poe::Character::Passives> passives; // ?object
        std::optional<poe::Character::Metadata> metadata; // ?object

        inline bool operator<(const Character &other) const { return name < other.name; };
    };

    struct CharacterListWrapper
    {
        std::vector<poe::Character> characters;
    };

    struct CharacterWrapper
    {
        std::optional<poe::Character> character;
    };

} // namespace poe

template<>
struct glz::meta<poe::Character>
{
    using T = poe::Character;
    static constexpr auto modify = glz::object("class", &T::class_);
};
