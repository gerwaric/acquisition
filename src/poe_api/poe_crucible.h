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

#include <optional>
#include <string>
#include <vector>

namespace PoE {

	// https://www.pathofexile.com/developer/docs/reference#type-CrucibleNode
	struct CrucibleNode {
		std::optional<unsigned int>                 skill;					// ? uint	mod hash
		std::optional<unsigned int>                 tier;					// ? uint	mod tier
		std::optional<std::string>					icon;					// ? string
		std::optional<bool>                         allocate;				// ? bool	always true if present
		std::optional<bool>                         isNotable;				// ? bool	always true if present
		std::optional<bool>                         isReward;				// ? bool	always true if present
		std::optional<std::vector<std::string>>     stats;					// ? array of string	stat descriptions
		std::optional<std::vector<std::string>>     reminderText;			// ? array of string;
		std::optional<unsigned int>                 orbit;					// ? uint	the column this not occupies
		std::optional<unsigned int>                 orbitIndex;				// ? uint	the node's position within the column
		std::vector<std::string>                    out;					// array of string	node identifiers of nodes this one connects to
		std::vector<std::string>                    in;						// array of string	node identifiers of nodes connected to this one
		JS_OBJ(skill, tier, icon, allocate, isNotable, isReward, stats, reminderText, orbit, orbitIndex, out, in);
	};

}
