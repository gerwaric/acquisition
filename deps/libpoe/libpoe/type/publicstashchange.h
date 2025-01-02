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

#include <libpoe/type/item.h>

#include <optional>
#include <vector>

namespace libpoe{

    // https://www.pathofexile.com/developer/docs/reference#type-PublicStashChange

    struct PublicStashChange {

        QString id; // string a unique 64 digit hexadecimal string
        bool public_; // bool if false then optional properties will be null
        std::optional<QString> accountName; // ? string
        std::optional<QString> stash; // ? string the name of the stash
        std::optional<QString> lastCharacterName; // ? string not included by default.Requires extra permissions
        QString stashType; // string
        std::optional<QString> league; // ? string the league's name
        std::vector<libpoe::Item> items; // array of Item

        JS_OBJECT(
            JS_MEMBER(id),
            JS_MEMBER_WITH_NAME(public_, "public"),
            JS_MEMBER(accountName),
            JS_MEMBER(stash),
            JS_MEMBER(lastCharacterName),
            JS_MEMBER(stashType),
            JS_MEMBER(league),
            JS_MEMBER(items));

    };

}
