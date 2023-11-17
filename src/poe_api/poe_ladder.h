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
#include "poe_api/poe_account.h"

#include <optional>
#include <string>

namespace PoE {

	// Anonymous member of LadderEntryCharacter
	struct LadderEntryDepth {
		unsigned int                                default_ = 0;
		unsigned int                                solo = 0;
		JS_OBJ(default_, solo);
	};

	// Anonymous member of LadderEntry
	struct LadderEntryCharacter {
		std::string                                 id;						// string	a unique 64 digit hexadecimal string
		std::string                                 name;					// string
		unsigned int                                level = 0;				// uint
		std::string                                 class_;					// string
		std::optional<unsigned int>                 time;					// ? uint	time taken to complete the league objective in seconds
		std::optional<unsigned int>                 score;					// ? uint	count of league objective completions
		std::optional<JS::JsonObject>               progress;				// ? object	the values of this depend on the league objective
		std::optional<unsigned int>                 experience;				// ? uint
		std::optional<PoE::LadderEntryDepth>        depth;					// ? object	deepest Delve depth completed
		JS_OBJ(id, name, level, class_, time, score, progress, experience, depth);
	};

	// https://www.pathofexile.com/developer/docs/reference#type-LadderEntry
	struct LadderEntry {
		unsigned int                                rank = 0;				// uint
		std::optional<bool>                         dead;					// ? bool
		std::optional<bool>                         retired;				// ? bool
		std::optional<bool>                         ineligible;				// ? bool
		std::optional<bool>                         public_;				// ? bool
		PoE::LadderEntryCharacter                   character;				// object
		std::optional<PoE::Account>                 account;				// ? Account
		JS_OBJ(rank, dead, retired, ineligible, public_, character, account);
	};

	// Anonymous member of EventLadderEntry
	struct PrivateLeague {
		std::string                                 name;					// string
		std::string                                 url;					// string	a url link to a Path of Exile Private League
		JS_OBJ(name, url);
	};

	// https://www.pathofexile.com/developer/docs/reference#type-EventLadderEntry
	struct EventLadderEntry {
		unsigned int                                rank = 0;				// uint
		std::optional<bool>                         ineligible;				// ? bool
		std::optional<unsigned int>                 time;					// ? uint	time taken to complete the league objective in seconds
		PoE::PrivateLeague                          private_league;			// object
		JS_OBJ(rank, ineligible, time, private_league);
	};

}
