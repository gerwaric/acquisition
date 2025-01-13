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

    // https://www.pathofexile.com/developer/docs/reference#type-PvPMatch

    struct PvPMatch {

        QString id; // string the match's name
        std::optional<QString> realm; // ? string pc, xbox, or sony
        std::optional<QString> startAt; // ? string date time(ISO8601)
        std::optional<QString> endAt; // ? string date time(ISO8601)
        std::optional<QString> url; // ? string a url link to a Path of Exile forum thread
        QString description; // string
        bool glickoRatings; // bool
        bool pvp; // bool always true
        QString style; // string Blitz, Swiss, or Arena
        std::optional<QString> registerAt; // ? string date time(ISO8601)
        std::optional<bool>complete; // ? bool always true if present
        std::optional<bool>upcoming; // ? bool always true if present
        std::optional<bool> inProgress; // ? bool always true if present

        JS_OBJ(id, realm, startAt, endAt, url, description, glickoRatings, pvp, style, registerAt, complete, upcoming, inProgress);

    };

}
