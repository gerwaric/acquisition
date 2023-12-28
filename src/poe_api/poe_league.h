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

#include "poe_api/poe_typedefs.h"

#include <optional>
#include <string>
#include <vector>

class QObject;

namespace PoE {

	// https://www.pathofexile.com/developer/docs/reference#type-LeagueRule
	struct LeagueRule {
		std::string                                 id;                     // string examples : Hardcore, NoParties(SSF)
		std::string                                 name;                   // string
		std::optional<std::string>                  description;            // ? string
		JS_OBJ(id, name, description);
	};

	// Anonymous member of League
	struct LeagueCategory {
		std::string                                 id;                     // string	the league category, e.g.Affliction
		std::optional<bool>                         active;                 // ? bool	always true if present
	};

	// https://www.pathofexile.com/developer/docs/reference#type-League
	struct League {
		std::string                                 id;						// string   the league's name
		std::optional<std::string>                  realm;					// ? string pc, xbox, or sony
		std::optional<std::string>                  description;			// ? string
		std::optional<PoE::LeagueCategory>          category;
		std::optional<std::vector<PoE::LeagueRule>> rules;					// ? array of LeagueRule
		std::optional<std::string>                  registerAt;				// ? string date time(ISO8601)
		std::optional<bool>                         event;					// ? bool   always true if present
		std::optional<std::string>                  url;					// ? string a url link to a Path of Exile forum thread
		std::optional<std::string>                  startAt;                // ? string date time(ISO8601)
		std::optional<std::string>                  endAt;                  // ? string date time(ISO8601)
		std::optional<bool>                         timedEvent;             // ? bool   always true if present
		std::optional<bool>                         scoreEvent;             // ? bool   always true if present
		std::optional<bool>                         delveEvent;             // ? bool   always true if present
		std::optional<bool>                         ancestorEvent;          // ? bool   always true if present
		std::optional<bool>                         leagueEvent;            // ? bool   always true if present
		JS_OBJ(id, realm, description, category, rules, registerAt, event, url, startAt, endAt, timedEvent, scoreEvent, delveEvent, ancestorEvent, leagueEvent);
	};

	using GetLeaguesCallback = std::function<void(const std::vector<PoE::League>&)>;

	void GetLeagues(QObject* object, PoE::GetLeaguesCallback callback);
}
