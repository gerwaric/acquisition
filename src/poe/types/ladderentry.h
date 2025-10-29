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

#include <optional>

#include <glaze/glaze.hpp>

#include "poe/types/account.h"

namespace poe {

    // https://www.pathofexile.com/developer/docs/reference#type-LadderEntry

    struct LadderEntry
    {
        struct DelveInfo
        {
            std::optional<unsigned> default_; // ? uint
            std::optional<unsigned> solo;     // ? uint
        };

        struct Character
        {
            QString id;     // string a unique 64 digit hexadecimal string
            QString name;   // string
            unsigned level; // uint
            QString class_; // string
            std::optional<unsigned>
                time; // ? uint time taken to complete the league objective in seconds
            std::optional<unsigned> score; // ? uint count of league objective completions
            // TBD progress; // ? object the values of this depend on the league objective
            std::optional<unsigned> experience;               // ? uint
            std::optional<poe::LadderEntry::DelveInfo> depth; // ? object deepest Delve depth completed
        };

        unsigned rank;                         // uint
        std::optional<bool> dead;              // ? bool
        std::optional<bool> retired;           // ? bool
        std::optional<bool> ineligible;        // ? bool
        std::optional<bool> public_;           // ? bool
        poe::LadderEntry::Character character; // object
        std::optional<poe::Account> account;   // ? Account
    };

} // namespace poe

template<>
struct glz::meta<poe::LadderEntry::DelveInfo>
{
    using T = poe::LadderEntry::DelveInfo;
    static constexpr auto modify = glz::object("default", &T::default_);
};

template<>
struct glz::meta<poe::LadderEntry::Character>
{
    using T = poe::LadderEntry::Character;
    static constexpr auto modify = glz::object("class", &T::class_);
};

template<>
struct glz::meta<poe::LadderEntry>
{
    using T = poe::LadderEntry;
    static constexpr auto modify = glz::object("public", &T::public_);
};
