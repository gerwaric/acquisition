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

#include <libpoe/type/account.h>

#include <optional>

namespace libpoe{

    // https://www.pathofexile.com/developer/docs/reference#type-PvPLadderTeamMember

    struct PvPLadderTeamMember {

        struct Character {
            QString id; // string a unique 64 digit hexadecimal string
            QString name; // string
            unsigned level; // uint
            QString class_; // string
            std::optional<QString> league; // ? string
            std::optional<unsigned> score; // ? uint count of league objective completions
            
            JS_OBJECT(
                JS_MEMBER(id),
                JS_MEMBER(name),
                JS_MEMBER(level),
                JS_MEMBER_WITH_NAME(class_, "class"),
                JS_MEMBER(league),
                JS_MEMBER(score));
        };

        libpoe::Account account; // Account
        libpoe::PvPLadderTeamMember::Character character; // object
        std::optional<bool> public_; // ? bool always true if present
        
        JS_OBJECT(
            JS_MEMBER(account),
            JS_MEMBER(character),
            JS_MEMBER_WITH_NAME(public_, "public"));
    };

}
