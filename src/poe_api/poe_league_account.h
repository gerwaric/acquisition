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
#include <vector>

namespace PoE {

	// Anonymous member of LeagueAccount
	struct AtlasPassives {
		std::vector<unsigned int>                   hashes;					// array of uint
		JS_OBJ(hashes);
	};

	// https://www.pathofexile.com/developer/docs/reference#type-LeagueAccount
	struct LeagueAccount {
		std::optional<PoE::AtlasPassives>           atlas_passives;			// ? object
		JS_OBJ(atlas_passives);
	};

}