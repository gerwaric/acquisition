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
#include <string>

namespace libpoe{

    // https://www.pathofexile.com/developer/docs/reference#type-LadderEntry

    struct LadderEntry {

        struct DelveInfo {
            std::optional<unsigned> default_; // ? uint
            std::optional<unsigned> solo; // ? uint
            JS_OBJECT(
                JS_MEMBER_WITH_NAME(default_, "default"),
                JS_MEMBER(solo));
        };

        struct Character {
            QString id; // string a unique 64 digit hexadecimal string
            QString name; // string
            unsigned level; // uint
            QString class_; // string
            std::optional<unsigned> time; // ? uint time taken to complete the league objective in seconds
            std::optional<unsigned> score; // ? uint count of league objective completions
            // TBD progress; // ? object the values of this depend on the league objective
            std::optional<unsigned> experience; // ? uint
            std::optional<libpoe::LadderEntry::DelveInfo> depth; // ? object deepest Delve depth completed
            JS_OBJECT(
                JS_MEBMER(id),
                JS_MEMBER(name),
                JS_MEMBER(level),
                JS_MEMBER_WITH_NAME(class_, "class"),
                JS_MEMBER(time),
                JS_MEMBER(score),
                /* JS_MEMBER(progress), */
                JS_MEMBER(experience),
                JS_MEMBER(depth));
        };

        unsigned rank; // uint
        std::optional<bool> dead; // ? bool
        std::optional<bool> retired; // ? bool
        std::optional<bool> ineligible; // ? bool
        std::optional<bool> public_; // ? bool
        libpoe::LadderEntry::Character character; // object
        std::optional<libpoe::Account> account; // ? Account
        JS_OBJECT(
            JS_MEMBER(rank),
            JS_MEMBER(dead),
            JS_MEMBER(retired),
            JS_MEMBER(ineligible),
            JS_MEMBER_WITH_NAME(public_, "public"),
            JS_MEMBER(character),
            JS_MEMBER(account));

    };

}
