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
#include "poe_api/poe_item.h"

#include <optional>
#include <string>
#include <vector>

class QObject;

namespace PoE {

	// Anonymous member of StashTab
	struct StashTabMetadata {
		std::optional<bool>                         is_public;				// ? bool	always true if present
		std::optional<bool>                         folder;					// ? bool	always true if present
		std::optional<std::string>					colour;					// ? string	6 digit hex colour
		JS_OBJECT(JS_MEMBER_WITH_NAME(is_public, "public"),
				  JS_MEMBER(folder),
				  JS_MEMBER(colour));
	};

	// https://www.pathofexile.com/developer/docs/reference#type-StashTab
	struct StashTab {
		std::string                                 id;						// string	a 10 digit hexadecimal string
		std::optional<std::string>					parent;					// ? string	a 10 digit hexadecimal string
		std::string                                 name;					// string
		std::string                                 type;					// string
		std::optional<unsigned int>                 index;					// ? uint
		PoE::StashTabMetadata                       metadata;				// object
		std::optional<std::vector<PoE::StashTab>>   children;               // ? array of StashTab
		std::optional<std::vector<PoE::Item>>       items;                  // ? array of Item
		JS_OBJ(id, parent, name, type, index, metadata);
	};

	// https://www.pathofexile.com/developer/docs/reference#type-PublicStashChange
	struct PublicStashChange {
		std::string                                 id;						// string	a unique 64 digit hexadecimal string
		bool                                        public_ = false;		// bool	if false then optional properties will be null
		std::optional<std::string>					accountName;			// ? string
		std::optional<std::string>					stash;					// ? string	the name of the stash
		std::optional<std::string>					lastCharacterName;		// ? string	not included by default.Requires extra permissions
		std::string                                 stashType;				// string
		std::optional<std::string>					league;					// ? string	the league's name
		std::vector<PoE::Item>                      items;					// array of Item
		JS_OBJ(id, public_, accountName, stash, lastCharacterName, stashType, league, items);
	};

	struct LegacyStashTabColour {
		unsigned int r;
		unsigned int g;
		unsigned int b;
		JS_OBJ(r, g, b);
	};

	struct LegacyStashTab {
		LegacyStashTab(const StashTab& stash);
		std::string n;
		unsigned int i;
		std::string id;
		std::string type;
		bool        selected;
		LegacyStashTabColour colour;
		std::string srcL;
		std::string srcC;
		std::string srcR;
		JS_OBJ(n, i, id, type, selected, colour, srcL, srcC, srcR);
	};

	struct ListStashesResult {
		std::vector<StashTab> stashes;
		JS_OBJ(stashes);
	};
	struct GetStashResult {
		StashTab stash;
		JS_OBJ(stash);
	};

	typedef std::function<void(const ListStashesResult&)> ListStashesCallback;
	typedef std::function<void(const GetStashResult&)> GetStashCallback;

	void ListStashes(QObject* object, ListStashesCallback callback,
		const std::string& league);

	void GetStash(QObject* object, GetStashCallback,
		const std::string& league,
		const std::string& stash_id,
		const std::string& substash_id = "");

}
