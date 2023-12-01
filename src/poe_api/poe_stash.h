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
		StashTab() {};
		StashTab(const std::string& json);
		std::string                                 id;						// string	a 10 digit hexadecimal string
		std::optional<std::string>					parent;					// ? string	a 10 digit hexadecimal string
		std::string                                 name;					// string
		std::string                                 type;					// string
		std::optional<unsigned int>                 index;					// ? uint
		PoE::StashTabMetadata                       metadata;				// object
		std::optional<std::vector<PoE::StashTab>>   children;               // ? array of StashTab
		std::optional<std::vector<PoE::Item>>       items;                  // ? array of Item
		JS_OBJ(id, parent, name, type, index, metadata, children, items);
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

	using ListStashesCallback = std::function<void(const std::vector<PoE::StashTab>&)>;
	using GetStashCallback = std::function<void(const PoE::StashTab&)>;

	void ListStashes(QObject* object, PoE::ListStashesCallback callback,
		const PoE::LeagueName& league);

	void GetStash(QObject* object, PoE::GetStashCallback,
		const PoE::LeagueName& league,
		const PoE::StashId& stash_id,
		const PoE::StashId& substash_id = PoE::StashId());

}
