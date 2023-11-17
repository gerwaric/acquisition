/*
	Copyright 2023 Gerwaric

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

#include "json_struct/json_struct.h"
#include "poe_api/poe_crucible.h"
#include "poe_api/poe_passives.h"

#include <optional>
#include <string>
#include <tuple>
#include <unordered_map>
#include <vector>

namespace PoE {

	// https://www.pathofexile.com/developer/docs/reference#type-ItemSocket
	struct ItemSocket {
		unsigned int                                group = 0;				// uint
		std::optional<std::string>					attr;					// ? string	S, D, I, G, A, or DV
		std::optional<std::string>					sColour;             	// ? string R, G, B, W, A, or DV
		JS_OBJ(group, attr, sColour);
	};

	// https://www.pathofexile.com/developer/docs/reference#type-ItemProperty
	struct ItemProperty {
		std::string                                 name;					// string
		std::vector<std::tuple<std::string, unsigned int>> values;          // array of array
		unsigned int                                displayMode;			// uint
		std::optional<double>                       progress;				// ? double	rounded to 2 decimal places
		std::optional<unsigned int>                 type;					// ? uint
		std::optional<std::string>					suffix;					// ? string
		JS_OBJ(name, values, displayMode, progress, type, suffix);
	};

	// Anonymous member of Item
	struct ItemReward {
		std::string                                 label;					// string
		std::unordered_map<std::string, int>        rewards;				// dictionary of int - the key is a string representing the type of reward. The value is the amount
		JS_OBJ(label, rewards);
	};

	// Anonymous member of LogbookMod
	struct LogbookFaction {
		std::string                                 id;						// string	Faction1, Faction2, Faction3, or Faction4
		std::string                                 name;					// string
		JS_OBJ(id, name);
	};

	// Anonymous member of Item
	struct LogbookMod {
		std::string                                 name;					// string	area name
		PoE::LogbookFaction                         faction;				// object
		std::vector<std::string>                    mods;					// array of string
		JS_OBJ(name, faction, mods);
	};

	// Anonymous member of Item
	struct UltimatumMod {
		std::string                                 type;					// string	text used to display ultimatum icons
		unsigned int                                tier = 0;				// uint
		JS_OBJ(type, tier);
	};

	// Anonymous member of Item
	struct IncubatedItemInfo {
		std::string                             	name;					// string
		unsigned int                                level = 0;				// uint		monster level required to progress
		unsigned int                                progress = 0;			// uint
		unsigned int                                total = 0;				// uint
		JS_OBJ(name, level, progress, total);
	};

	// Anonymous member of Item
	struct ScourgedItemInfo {
		unsigned int                                tier = 0;				// uint		1 - 3 for items, 1 - 10 for maps
		std::optional<unsigned int>                 level;					// ? uint	monster level required to progress
		std::optional<unsigned int>                 progress;				// ? uint
		std::optional<unsigned int>                 total;					// ? uint
		JS_OBJ(tier, level, progress, total);
	};

	// Anonymous member of Item
	struct CrucibleItemInfo {
		std::string                                 layout;					// string	URL to an image of the tree layout
		std::unordered_map<std::string, PoE::CrucibleNode> nodes;                // dictionary of CrucibleNode	the key is the string value of the node index
		JS_OBJ(layout, nodes);
	};

	// Anonymous member of Item
	struct HybridItemInfo {
		std::optional<bool>                         isVaalGem;				// ? bool
		std::string                                 baseTypeName;			// std::string
		std::optional<std::vector<PoE::ItemProperty>>	properties;				// ? array of ItemProperty
		std::optional<std::vector<std::string>>     explicitMods;			// ? array of string
		std::optional<std::string>                  secDescrText;			// ? string
		JS_OBJ(isVaalGem, baseTypeName, properties, explicitMods, secDescrText);
	};

	// Anonymous member of Item
	struct ExtendedItemInfo {
		std::optional<std::string>					category;				// ? string - (only present in the Public Stash API)
		std::optional<std::vector<std::string>>     subcategories;			// ? array of string - (only present in the Public Stash API)
		std::optional<unsigned int>                 prefixes;				// ? uint (only present in the Public Stash API)
		std::optional<unsigned int>                 suffixes;				// ? uint (only present in the Public Stash API)
		JS_OBJ(category, subcategories, prefixes, suffixes);
	};

	// https://www.pathofexile.com/developer/docs/reference#type-FrameType
	enum class FrameType {
		Normal = 0,
		Magic = 1,
		Rare = 2,
		Unique = 3,
		Gem = 4,
		Currency = 5,
		DivinationCard = 6,
		Quest = 7,
		Prophecy = 8,
		Foil = 9,
		SupporterFoil = 10
	};
}

// This macro won't work in the PoE namespace.
JS_ENUM_NAMESPACE_DECLARE_VALUE_PARSER(PoE, FrameType);

namespace PoE {

	// https://www.pathofexile.com/developer/docs/reference#type-Item
	struct Item {
		bool                                        verified = false;		// bool
		unsigned int                                w = 0;					// uint
		unsigned int                                h = 0;					// uint
		std::string                                 icon;					// string
		std::optional<bool>                         support;				// ? always true if present
		std::optional<int>                          stackSize;				// ?
		std::optional<int>                      	maxStackSize;			// ?
		std::optional<std::string>					stackSizeText;			// ?
		std::optional<std::string>					league;					// ?
		std::optional<std::string>					id;						// ? a unique 64 digit hexadecimal string
		std::optional<JS::JsonObject>               influences;				// ? object       /***** WARNING *****/
		std::optional<bool>                         elder;					// ? always true if present
		std::optional<bool>                         shaper;					// ? always true if present
		std::optional<bool>                         searing;				// ? always true if present
		std::optional<bool>                         tangled;				// ? always true if present
		std::optional<bool>                         abyssJewel;				// ? always true if present
		std::optional<bool>                         delve;					// ? always true if present
		std::optional<bool>                         fractured;				// ? always true if present
		std::optional<bool>                         synthesised;			// ? always true if present
		std::optional<std::vector<PoE::ItemSocket>>	sockets;				// ? array of ItemSocket
		std::optional<std::vector<PoE::Item>>		socketedItems;			// ? array of Item
		std::string                                 name;					// string
		std::string                                 typeLine;				// string
		std::string                                 baseType;				// string
		bool                                        identified = false;		// bool
		std::optional<int>                          itemLevel;				// ? int
		int                                         ilvl = 0;				// deprecated
		std::optional<std::string>					note;					// ? user-generated text
		std::optional<std::string>					forum_note;				// ? user-generated text
		std::optional<bool>                         lockedToCharacter;		// ? always true if present
		std::optional<bool>                         lockedToAccount;		// ? always true if present
		std::optional<bool>                         duplicated;				// ? always true if present
		std::optional<bool>                         split;					// ? always true if present
		std::optional<bool>                         corrupted;				// ? always true if present
		std::optional<bool>                         unmodifiable;			// ? always true if present
		std::optional<bool>                         cisRaceReward;			// ? always true if present
		std::optional<bool>                         seaRaceReward;			// ? always true if present
		std::optional<bool>                         thRaceReward;			// ? always true if present
		std::optional<std::vector<PoE::ItemProperty>>	properties;				// ? array of ItemProperty
		std::optional<std::vector<PoE::ItemProperty>>	notableProperties;		// ? array of ItemProperty
		std::optional<std::vector<PoE::ItemProperty>>	requirements;			// ? array of ItemProperty
		std::optional<std::vector<PoE::ItemProperty>>	additionalProperties;	// ? array of ItemProperty
		std::optional<std::vector<PoE::ItemProperty>>	nextLevelRequirements;	// ? array of ItemProperty
		std::optional<int>                          talismanTier;			// ? int
		std::optional<std::vector<PoE::ItemReward>>		rewards;				// ? array of object
		std::optional<std::vector<std::string>>     secDescrText;			// ? string
		std::optional<std::vector<std::string>>     utilityMods;			// ? array of string
		std::optional<std::vector<PoE::LogbookMod>>		logbookMods;			// ? array of object
		std::optional<std::vector<std::string>>     enchantMods;			// ? array of string
		std::optional<std::vector<std::string>>     scourgeMods;			// ? array of string
		std::optional<std::vector<std::string>>     implicitMods;			// ? array of string
		std::optional<std::vector<PoE::UltimatumMod>>	ultimatumMods;			// ? array of object
		std::optional<std::vector<std::string>>     explicitMods;			// ? array of string
		std::optional<std::vector<std::string>>     craftedMods;			// ? array of string
		std::optional<std::vector<std::string>>     fracturedMods;			// ? array of string
		std::optional<std::vector<std::string>>     crucibleMods;			// ? array of string	only allocated mods are included
		std::optional<std::vector<std::string>>     cosmeticMods;			// ? array of string
		std::optional<std::vector<std::string>>     veiledMods;				// ? array of string	random video identifier
		std::optional<bool>                         veiled;					// ? bool	always true if present
		std::optional<std::string>					descrText;				// ? string
		std::optional<std::vector<std::string>>     flavourText;			// ? array of string
		std::optional<JS::JsonObject>		    	flavourTextParsed;		// ? array of string or object  /****** WARNING ******/
		std::optional<std::string>					flavourTextNote;		// ? string	user-generated text
		std::optional<std::string>					prophecyText;			// ? string
		std::optional<bool>                         isRelic;				// ? bool	always true if present
		std::optional<int>                          foilVariation;			// ? int
		std::optional<bool>                         replica;				// ? bool	always true if present
		std::optional<bool>                         foreseeing;				// ? bool	always true if present
		std::optional<PoE::IncubatedItemInfo>       incubatedItem;          // ? object
		std::optional<PoE::ScourgedItemInfo>        scourged;				// ? object
		std::optional<PoE::CrucibleItemInfo>        crucible;				// ? object
		std::optional<bool>                         ruthless;				// ? bool	always true if present
		std::optional<PoE::FrameType>               frameType;				// ? uint as FrameType	often used to determine an item's rarity
		std::optional<std::string>					artFilename;			// ? string
		std::optional<PoE::HybridItemInfo>          hybrid;					// ? object
		std::optional<PoE::ExtendedItemInfo>        extended;				// ? object	only present in the Public Stash API
		std::optional<unsigned int>                 x;						// ? uint
		std::optional<unsigned int>                 y;						// ? uint
		std::optional<std::string>				 	inventoryId;			// ? string
		std::optional<unsigned int>                 socket;					// ? uint
		std::optional<std::string>					colour;					// ? string	S, D, I, or G
		JS_OBJ(verified, w, h, icon, support, stackSize, maxStackSize, stackSizeText,
			   league, id, influences, elder, shaper, searing, tangled, abyssJewel, delve,
			   fractured, synthesised, sockets, socketedItems, name, typeLine, baseType, identified,
			   itemLevel, ilvl, note, forum_note, lockedToCharacter, lockedToAccount, duplicated, split,
			   corrupted, unmodifiable, cisRaceReward, seaRaceReward, thRaceReward, properties, notableProperties,
			   requirements, additionalProperties, nextLevelRequirements, talismanTier, rewards, secDescrText,
			   utilityMods, logbookMods, enchantMods, scourgeMods, implicitMods, ultimatumMods, explicitMods,
			   craftedMods, fracturedMods, crucibleMods, cosmeticMods, veiledMods, veiled, descrText, flavourText,
			   flavourTextParsed, flavourTextNote, prophecyText, isRelic, foilVariation, replica, foreseeing,
			   incubatedItem, scourged, crucible, ruthless, frameType, artFilename, hybrid, extended, x, y,
			   inventoryId, socket, colour);
	};

	// Anonymous member of ItemJewelData
	struct ItemJewelSubgraph {
		std::unordered_map<std::string, PoE::PassiveGroup> groups;               // dictionary of PassiveGroup	the key is the string value of the group id
		std::unordered_map<std::string, PoE::PassiveNode> nodes;                 // dictionary of PassiveNode	the key is the string value of the node identifier
		JS_OBJ(groups, nodes);
	};

	// https://www.pathofexile.com/developer/docs/reference#type-ItemJewelData
	struct ItemJewelData {
		std::string                                 type;					// string
		std::optional<unsigned int>                 radius;					// ? uint
		std::optional<unsigned int>                 radiusMin;				// ? uint
		std::optional<std::string>					radiusVisual;			// ? string
		std::optional<PoE::ItemJewelSubgraph>       subgraph;				// ? object	only present on cluster jewels
		JS_OBJ(type, radius, radiusMin, radiusVisual, subgraph);
	};

}
