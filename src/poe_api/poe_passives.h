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

#include <limits>
#include <optional>
#include <string>
#include <vector>

namespace PoE {

	// Anonymous member of PassiveNode
	struct PassiveExpansionJewel {
		std::optional<unsigned int>                 size;					// ? uint
		std::optional<unsigned int>                 index;					// ? uint
		std::optional<unsigned int>                 proxy;					// ? uint	the proxy node identifier
		std::optional<unsigned int>                 parent;					// ? uint	the parent node identifier
		JS_OBJ(size, index, proxy, parent);
	};

	// Anonymous member of PassiveNode
	struct PassiveMasteryEffect {
		unsigned int                                effect = 0;					// uint	effect hash
		std::vector<std::string>                    stats;					// array of string	stat descriptions
		std::optional<std::vector<std::string>>     reminderText;			// ? array of string
		JS_OBJ(effect, stats, reminderText);
	};

	// https://www.pathofexile.com/developer/docs/reference#type-PassiveGroup
	struct PassiveGroup {
		float                                       x = std::numeric_limits<float>::signaling_NaN(); // float
		float                                       y = std::numeric_limits<float>::signaling_NaN(); // float
		std::vector<unsigned int>                   orbits;					// array of uint
		std::optional<bool>                         isProxy;				// ? bool	always true if present
		std::optional<std::string>					proxy;					// ? string	identifier of the placeholder node
		std::vector<std::string>                    nodes;					// array of string	the node identifiers associated with this group
		JS_OBJ(x, y, orbits, isProxy, proxy, nodes);
	};

	// https://www.pathofexile.com/developer/docs/reference#type-PassiveNode
	struct PassiveNode {
		std::optional<unsigned int>                 skill;					// ? uint	skill hash
		std::optional<std::string>					name;					// ? string
		std::optional<std::string>					icon;					// ? string
		std::optional<bool>                         isKeystone;				// ? bool	always true if present
		std::optional<bool>                         isNotable;				// ? bool	always true if present
		std::optional<bool>                         isMastery;				// ? bool	always true if present
		std::optional<std::string>					inactiveIcon;			// ? string	inactive mastery image
		std::optional<std::string>					activeIcon;				// ? string	active mastery image
		std::optional<std::string>					activeEffectImage;		// ? string	active mastery or tattoo background image
		std::optional<std::vector<PoE::PassiveMasteryEffect>>	masteryEffects;	// ? array of object
		std::optional<bool>                         isBlighted;				// ? bool	always true if present
		std::optional<bool>                         isTattoo;				// ? bool	always true if present
		std::optional<bool>                         isProxy;                // ? bool	always true if present
		std::optional<bool>                         isJewelSocket;			// ? bool	always true if present
		std::optional<PoE::PassiveExpansionJewel>   expansionJewel;			// ? object	cluster jewel information
		std::optional<std::vector<std::string>>     recipe;					// ? array of string	components required for Blight crafting this node.
		//                                                                  //     each string is one of ClearOil, SepiaOil, AmberOil, VerdantOil,
		//                                                                  //     TealOil, AzureOil, IndigoOil, VioletOil, CrimsonOil,
		//                                                                  //     BlackOil, OpalescentOil, SilverOil, or GoldenOil
		std::optional<unsigned int>                 grantedStrength;		// ? uint	sum of stats on this node that grant strength
		std::optional<unsigned int>                 grantedDexterity;		// ? uint	sum of stats on this node that grant dexterity
		std::optional<unsigned int>                 grantedIntelligence;	// ? uint	sum of stats on this node that grant intelligence
		std::optional<std::string>					ascendancyName;			// ? string
		std::optional<bool>                         isAscendancyStart;		// ? bool	always true if present
		std::optional<bool>                         isMultipleChoice;		// ? bool	always true if present
		std::optional<bool>                         isMultipleChoiceOption;	// ? bool	always true if present
		std::optional<unsigned int>                 grantedPassivePoints;	// ? uint
		std::optional<std::vector<std::string>>     stats;					// ? array of string	stat descriptions
		std::optional<std::vector<std::string>>     reminderText;			// ? array of string
		std::optional<std::vector<std::string>>     flavourText;			// ? array of string
		std::optional<unsigned int>                 classStartIndex;		// ? uint
		std::optional<unsigned int>                 group;					// ? uint	index into the groups table
		std::optional<unsigned int>                 orbit;					// ? uint	the orbit this node occupies within it's group
		std::optional<unsigned int>                 orbitIndex;				// ? uint	the index of this node in the group's orbit
		std::vector<std::string>                    out;					// array of string	node identifiers of nodes this one connects to
		std::vector<std::string>                    in;						// array of string	node identifiers of nodes connected to this one
		JS_OBJ(skill, name, icon, isKeystone, isNotable, isMastery, inactiveIcon, activeIcon, activeEffectImage, masteryEffects,
			   isBlighted, isTattoo, isProxy, isJewelSocket, expansionJewel, recipe, grantedStrength, grantedDexterity,
			   grantedIntelligence, ascendancyName, isAscendancyStart, isMultipleChoice, isMultipleChoiceOption,
			   grantedPassivePoints, stats, reminderText, flavourText, classStartIndex, group, orbit, orbitIndex, out, in);
	};

}

