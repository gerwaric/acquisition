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

namespace libpoe {

    // https://www.pathofexile.com/developer/docs/reference#type-ItemFilter

    struct ItemFilter {

        struct Validity {
            bool valid; // bool
            std::optional<QString> version; // ? string game version
            std::optional<QString> validated; // ? string date time(ISO8601)
            JS_OBJ(valid, version, validated);
        };

        QString id; // string
        QString filter_name; // string
        QString realm; // string
        QString description; // string
        QString version; // string
        QString type; // string either Normal or Ruthless
        bool public_; // ? bool always true if present
        std::optional<QString> filter; // ? string not present when listing all filters
        std::optional<libpoe::ItemFilter::Validity> validation; // ? object not present when listing all filters
    
        JS_OBJECT(
            JS_MEMBER(id),
            JS_MEMBER(filter_name),
            JS_MEMBER(realm),
            JS_MEMBER(description),
            JS_MEMBER(version),
            JS_MEMBER(type),
            JS_MEMBER_WITH_NAME(public_, "public"),
            JS_MEMBER(filter),
            JS_MEMBER(validation));

    };

}
