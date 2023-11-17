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

#include "poe_api/poe_account.h"

namespace PoE {

	// https://www.pathofexile.com/developer/docs/reference#type-PvPMatch
	struct PvPMatch {
		std::string                                 id;						// string	the match's name
		std::optional<std::string>					realm;					// ? string	pc, xbox, or sony
		std::optional<std::string>					startAt;				// ? string	date time(ISO8601)
		std::optional<std::string>					endAt;					// ? string	date time(ISO8601)
		std::optional<std::string>					url;					// ? string	a url link to a Path of Exile forum thread
		std::string                                 description;			// string
		bool                                        glickoRatings = false;	// bool
		bool                                        pvp = false;			// bool	always true
		std::string                                 style;					// string	Blitz, Swiss, or Arena
		std::optional<std::string>					registerAt;				// ? string	date time(ISO8601)
		std::optional<bool>                         complete;				// ? bool	always true if present
		std::optional<bool>                         upcoming;				// ? bool	always true if present
		std::optional<bool>                         inProgress;				// ? bool	always true if present
		JS_OBJ(id, realm, startAt, endAt, url, description, glickoRatings, pvp, style, registerAr, complete, upcoming, inProgress);
	};

	// Anonymous member of PvPLadderTeamMember
	struct PvPLadderCharacter {
		std::string                                 id;						// string	a unique 64 digit hexadecimal string
		std::string                                 name;					// string
		unsigned int                                level = 0;				// uint
		std::string                                 class_name;				// string
		std::optional<std::string>					league;					// ? string
		std::optional<unsigned int>                 score;					// ? uint	count of league objective completions
		JS_OBJECT(JS_MEMBER(id),
				  JS_MEMBER(name),
				  JS_MEMBER(level),
				  JS_MEMBER_WITH_NAME(class_name, "name"),
				  JS_MEMBER(league),
				  JS_MEMBER(score));
	};

	// https://www.pathofexile.com/developer/docs/reference#type-PvPLadderTeamMember
	struct PvPLadderTeamMember {
		PoE::Account                                account;				// Account
		PoE::PvPLadderCharacter                     character;				// object
		std::optional<bool>                         is_public;				// ? bool	always true if present
		JS_OBJECT(JS_MEMBER(account),
				  JS_MEMBER(character),
				  JS_MEMBER_WITH_NAME(is_public, "public"));
	};

	// https://www.pathofexile.com/developer/docs/reference#type-PvPLadderTeamEntry
	struct PvPLadderTeamEntry {
		unsigned int                                rank = 0;				// uint
		std::optional<unsigned int>                 rating;					// ? uint	only present if the PvP Match uses Glicko ratings
		std::optional<unsigned int>                 points;					// ? uint
		std::optional<unsigned int>                 games_played;			// ? uint
		std::optional<unsigned int>                 cumulative_opponent_points; // ? uint
		std::optional<std::string>					last_game_time;			// ? string	date time(ISO8601)
		PoE::PvPLadderTeamMember                    members;				// array of PvPLadderTeamMember
		JS_OBJ(rank, rating, points, games_played, cumulative_opponent_points, last_game_time, members);
	};

}
