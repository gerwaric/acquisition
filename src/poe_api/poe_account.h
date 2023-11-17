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

	// Anonymous member of Account
	struct AccountChallenges {
		std::string                                 set;					// string	the challenge set
		unsigned int                                completed = 0;			// uint
		unsigned int                                max = 0;				// uint
		JS_OBJ(set, completed, max);
	};

	// Anonymous member of AccountTwitch
	struct AccountTwitchStream {
		std::string                                 name;					// string
		std::string                                 image;					// string
		std::string                                 status;					// string
		JS_OBJ(name, image, status);
	};

	// Anonymous member of Account
	struct AccountTwitch {
		std::string                                 name;					// string
		std::optional<PoE::AccountTwitchStream>     stream;					// ? object
		JS_OBJ(name, stream);
	};

	// https://www.pathofexile.com/developer/docs/reference#type-Guild
	struct Guild {
		unsigned int                                id = 0;					// uint
		std::string                                 name;					// string
		std::string                                 tag;					// string
		std::optional<unsigned int>                 points;					// ? uint
		std::optional<std::string>					statusMessage;			// ? string
		std::string                                 createdAt;				// string	date time(ISO8601)
		JS_OBJ(id, name, tag, points, statusMessage, createdAt);
	};

	// https://www.pathofexile.com/developer/docs/reference#type-Account
	struct Account {
		std::string                                 name;					// string
		std::optional<std::string>					realm;					// ? string	pc, xbox, or sony
		std::optional<PoE::Guild>                   guild;					// ? Guild
		std::optional<PoE::AccountChallenges>       challenges;				// ? object
		std::optional<PoE::AccountTwitch>           twitch;					// ? object
		JS_OBJ(name, realm, guild, challenges, twitch);
	};

}
