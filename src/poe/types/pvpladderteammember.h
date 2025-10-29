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

#include <QString>

#include <glaze/glaze.hpp>

#include "poe/types/account.h"

namespace poe {

    // https://www.pathofexile.com/developer/docs/reference#type-PvPLadderTeamMember

    struct PvPLadderTeamMember
    {
        struct Character
        {
            QString id;                    // string a unique 64 digit hexadecimal string
            QString name;                  // string
            unsigned level;                // uint
            QString class_;                // string
            std::optional<QString> league; // ? string
            std::optional<unsigned> score; // ? uint count of league objective completions
        };

        poe::Account account;                          // Account
        poe::PvPLadderTeamMember::Character character; // object
        std::optional<bool> public_;                   // ? bool always true if present
    };

} // namespace poe

template<>
struct glz::meta<poe::PvPLadderTeamMember::Character>
{
    using T = poe::PvPLadderTeamMember::Character;
    static constexpr auto modify = glz::object("class", &T::class_);
};

template<>
struct glz::meta<poe::PvPLadderTeamMember>
{
    using T = poe::PvPLadderTeamMember;
    static constexpr auto modify = glz::object("public", &T::public_);
};
