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

#include <json_struct/json_struct_qt.h>

#include <optional>

namespace libpoe{

    // https://www.pathofexile.com/developer/docs/reference#type-EventLadderEntry

    struct EventLadderEntry {

        struct PrivateLeague {
            QString name; // string
            QString url; // string a url link to a Path of Exile Private League
            JS_OBJ(name, url);
        };

        unsigned rank; // uint
        std::optional<bool> ineligible; // ? bool
        std::optional<unsigned> time; // ? uint time taken to complete the league objective in seconds
        libpoe::EventLadderEntry::PrivateLeague private_league; // object
        JS_OBJ(rank, ineligible, time, private_league);
    };

}
