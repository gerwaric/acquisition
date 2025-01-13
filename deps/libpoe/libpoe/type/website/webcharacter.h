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

#include <unordered_map>
#include <vector>

namespace libpoe {

    struct WebCharacter {
        QString name;
        QString realm;
        QString class_;
        QString league;
        unsigned level;
        bool pinnable;
        JS_OBJECT(
            JS_MEMBER(name),
            JS_MEMBER(realm),
            JS_MEMBER_WITH_NAME(class_, "class"),
            JS_MEMBER(league),
            JS_MEMBER(level),
            JS_MEMBER(pinnable));
    };

    struct LegacyCharacterList {
        std::vector<libpoe::WebCharacter> characters;
        JS_OBJ(characters);
    };

    struct LegacyCharacterStash {
        std::vector<libpoe::Item> items;
        JS_OBJ(items);
    };

}
