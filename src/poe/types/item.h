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

#include "util/glaze_qt.h"

#include <poe/types/cruciblenode.h>
#include <poe/types/frametype.h>
#include <poe/types/itemproperty.h>
#include <poe/types/itemsocket.h>

#include <QString>

#include <optional>
#include <unordered_map>
#include <vector>

namespace poe {

    // https://www.pathofexile.com/developer/docs/reference#type-Item

    struct Item {

        struct Influences {
            std::optional<bool> shaper;
            std::optional<bool> elder;
            std::optional<bool> crusader;
            std::optional<bool> redeemer;
            std::optional<bool> hunter;
            std::optional<bool> warlord;
        };

        struct Rewards {
            QString label; // string
            std::unordered_map<QString, QString> rewards; // dictionary of int the key is a string representing the type of reward.The value is the amount
        };

        struct LogbookFaction {
            QString id; // string Faction1, Faction2, Faction3, or Faction4
            QString name; // string
        };

        struct LogbookMods {
            QString name; // string area name
            poe::Item::LogbookFaction faction; // object
            std::vector<QString> mods; // array of string
        };

        struct UltimatumMods {
            QString type; // string text used to display ultimatum icons
            unsigned tier; // uint
        };

        struct IncubatingInfo {
            QString name; // string
            unsigned level; // uint monster level required to progress
            unsigned progress; // uint
            unsigned total; // uint
        };

        struct ScourgedInfo {
            unsigned tier; // uint 1 - 3 for items, 1 - 10 for maps
            std::optional<unsigned> level; // ? uint monster level required to progress
            std::optional<unsigned> progress; // ? uint
            std::optional<unsigned> total; // ? uint
        };

        struct CrucibleInfo {
            QString layout; // string URL to an image of the tree layout
            std::unordered_map<QString, CrucibleNode> nodes; // dictionary of CrucibleNode the key is the string value of the node index
        };

        struct HybridInfo {
            std::optional<bool> isVaalGem; // ? bool
            QString baseTypeName; // string
            std::optional<std::vector<poe::ItemProperty>> properties; // ? array of ItemProperty
            std::optional<std::vector<QString>> explicitMods; // ? array of string
            std::optional<QString> secDescrText; // ? string
        };

        struct ExtendedInfo {
            std::optional<QString> category; // ? string
            std::optional<std::vector<QString>> subcategories; // ? array of string
            std::optional<unsigned> prefixes; // ? uint
            std::optional<unsigned> suffixes; // ? uint
        };

        bool verified; // bool
        unsigned w; // uint
        unsigned h; // uint
        QString icon; // string
        std::optional<bool> support; // ? bool always true if present
        std::optional<int> stackSize; // ? int
        std::optional<int> maxStackSize; // ? int
        std::optional<QString> stackSizeText; // ? string
        std::optional<QString> league; // ? string
        std::optional<QString> id; // ? string a unique 64 digit hexadecimal string
        std::optional<poe::Item::Influences> influences; // ? object
        std::optional<bool> elder; // ? bool always true if present
        std::optional<bool> shaper; // ? bool always true if present
        std::optional<bool> searing; // ? bool always true if present
        std::optional<bool> tangled; // ? bool always true if present
        std::optional<bool> abyssJewel; // ? bool always true if present
        std::optional<bool> delve; // ? bool always true if present
        std::optional<bool> fractured; // ? bool always true if present
        std::optional<bool> synthesised; // ? bool always true if present
        std::optional<std::vector<poe::ItemSocket>> sockets; // ? array of ItemSocket
        std::optional<std::vector<poe::Item>> socketedItems; // ? array of Item
        QString name; // string
        QString typeLine; // string
        QString baseType; // string
        std::optional<QString> rarity; // ? string Normal, Magic, Rare, or Unique
        bool identified; // bool
        std::optional<int> itemLevel; // ? int used for items that always display their item level
        int ilvl; // int
        std::optional<QString> note; // ? string user - generated text
        std::optional<QString> forum_note; // ? string user - generated text
        std::optional<bool> lockedToCharacter; // ? bool always true if present
        std::optional<bool> lockedToAccount; // ? bool always true if present
        std::optional<bool> duplicated; // ? bool always true if present
        std::optional<bool> split; // ? bool always true if present
        std::optional<bool> corrupted; // ? bool always true if present
        std::optional<bool> unmodifiable; // ? bool always true if present
        std::optional<bool> cisRaceReward; // ? bool always true if present
        std::optional<bool> seaRaceReward; // ? bool always true if present
        std::optional<bool> thRaceReward; // ? bool always true if present
        std::optional<std::vector<poe::ItemProperty>> properties; // ? array of ItemProperty
        std::optional<std::vector<poe::ItemProperty>> notableProperties; // ? array of ItemProperty
        std::optional<std::vector<poe::ItemProperty>> requirements; // ? array of ItemProperty
        std::optional<std::vector<poe::ItemProperty>> additionalProperties; // ? array of ItemProperty
        std::optional<std::vector<poe::ItemProperty>> nextLevelRequirements; // ? array of ItemProperty
        std::optional<int> talismanTier; // ? int
        std::optional<std::vector<poe::Item::Rewards>> rewards; // ? array of object
        std::optional<QString> secDescrText; // ? string
        std::optional<std::vector<QString>> utilityMods; // ? array of string
        std::optional<std::vector<poe::Item::LogbookMods>> logbookMods; // ? array of object
        std::optional<std::vector<QString>> enchantMods; // ? array of string
        std::optional<std::vector<QString>> scourgeMods; // ? array of string
        std::optional<std::vector<QString>> implicitMods; // ? array of string
        std::optional<std::vector<poe::Item::UltimatumMods>> ultimatumMods; // ? array of object
        std::optional<std::vector<QString>> explicitMods; // ? array of string
        std::optional<std::vector<QString>> craftedMods; // ? array of string
        std::optional<std::vector<QString>> fracturedMods; // ? array of string
        std::optional<std::vector<QString>> crucibleMods; // ? array of string only allocated mods are included
        std::optional<std::vector<QString>> cosmeticMods; // ? array of string
        std::optional<std::vector<QString>> veiledMods; // ? array of string random video identifier
        std::optional<bool> veiled; // ? bool always true if present
        std::optional<QString> descrText; // ? string
        std::optional<std::vector<QString>> flavourText; // ? array of
        // WARNING: flavourTextParsed might sometimes be an object
        //std::optional<std::vector<QString>> flavourTextParsed; // ? array of string or object
        std::optional<std::vector<QString>> flavourTextNote; // ? string user - generated text
        std::optional<std::vector<QString>> prophecyText; // ? string
        std::optional<bool> isRelic; // ? bool always true if present
        std::optional<int> foilVariation; // ? int
        std::optional<bool> replica; // ? bool always true if present
        std::optional<bool> foreseeing; // ? bool always true if present
        std::optional<poe::Item::IncubatingInfo> incubatedItem; // ? object
        std::optional<poe::Item::ScourgedInfo> scourged; // ? object
        std::optional<poe::Item::CrucibleInfo> crucible; // ? object
        std::optional<bool> ruthless; // ? bool always true if present
        poe::FrameType frameType; // ? uint as FrameType
        std::optional<QString> artFilename; // ? string
        std::optional<poe::Item::HybridInfo> hybrid; // ? object
        std::optional<poe::Item::ExtendedInfo> extended; // ? object only present in the Public Stash API
        std::optional<unsigned> x; // ? uint
        std::optional<unsigned> y; // ? uint
        std::optional<QString> inventoryId; // ? string
        std::optional<unsigned> socket; // ? uint
        std::optional<QString> colour; // ? string S, D, I, or G

        inline bool operator<(const Item& other) const {
            const QString a = name.isEmpty() ? typeLine : name + " " + typeLine;
            const QString b = other.name.isEmpty() ? other.typeLine : other.name + " " + other.typeLine;
            return a < b;
        };
    };

}
