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

namespace PoE {

	struct ItemFilterValidation {
		bool                                        valid = false;			// bool
		std::optional<std::string>                  version;				// ? string	game version
		std::optional<std::string>					validated;				// ? string	date time (ISO8601)
		JS_OBJ(valid, version, validated);
	};

	// https://www.pathofexile.com/developer/docs/reference#type-ItemFilter
	struct ItemFilter {
		std::string                                 id;						// string
		std::string                                 filter_name;			// string
		std::string                                 realm;					// string
		std::string                                 description;			// string
		std::string                                 version;				// string
		std::string                                 type;					// string	either Normal or Ruthless
		std::optional<bool>                         is_public;				// ? bool	always true if present
		std::optional<std::string>					filter;					// ? string	not present when listing all filters
		std::optional<PoE::ItemFilterValidation>    validation;				// ? object	not present when listing all filters
		JS_OBJECT(
			JS_MEMBER(id),
			JS_MEMBER(filter_name),
			JS_MEMBER(realm),
			JS_MEMBER(description),
			JS_MEMBER(version),
			JS_MEMBER(type),
			JS_MEMBER_WITH_NAME(is_public, "public"),
			JS_MEMBER(filter),
			JS_MEMBER(validation));
	};

}
