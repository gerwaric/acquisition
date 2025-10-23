/*
    Copyright (C) 2024-2025 Gerwaric

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

#include <poe/types/pvpladderteammember.h>

#include <QString>

#include <optional>
#include <vector>

namespace libpoe{

    // https://www.pathofexile.com/developer/docs/reference#type-PvPLadderTeamEntry

    struct PvPLadderTeamEntry {

        unsigned rank; // uint
        std::optional<unsigned> rating; // ? uint only present if the PvP Match uses Glicko ratings
        std::optional<unsigned> points; // ? uint
        std::optional<unsigned> games_played; // ? uint
        std::optional<unsigned> cumulative_opponent_points; // ? uint
        std::optional<QString> last_game_time; // ? string date time(ISO8601)
        std::vector<poe::PvPLadderTeamMember> members; // array of PvPLadderTeamMember

    };

}
