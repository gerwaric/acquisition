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
#include <vector>

#include <QString>

#include "poe/types/leaguerule.h"

namespace poe {

    // https://www.pathofexile.com/developer/docs/reference#type-League

    struct League
    {
        struct Category
        {
            QString id; // string the league category, e.g.Affliction
            std::optional<bool>
                current; // ? bool set for the active challenge leagues; always true if present
        };

        QString id;                                        // string the league's name
        std::optional<QString> realm;                      // ? string pc, xbox, or sony
        std::optional<QString> name;                       // ? string the league's name
        std::optional<QString> description;                // ? string
        std::optional<poe::League::Category> category;     // ? object
        std::optional<std::vector<poe::LeagueRule>> rules; // ? array of LeagueRule
        std::optional<QString> registerAt;                 // ? string date time(ISO8601)
        std::optional<bool> event;                         // ? bool always true if present
        std::optional<QString> url;        // ? string a url link to a Path of Exile forum thread
        std::optional<QString> startAt;    // ? string date time(ISO8601)
        std::optional<QString> endAt;      // ? string date time(ISO8601)
        std::optional<bool> timedEvent;    // ? bool always true if present
        std::optional<bool> scoreEvent;    // ? bool always true if present
        std::optional<bool> delveEvent;    // ? bool always true if present
        std::optional<bool> ancestorEvent; // ? bool always true if present
        std::optional<bool> leagueEvent;   // ? bool always true if present
    };

    using LeagueList = std::vector<std::unique_ptr<poe::League>>;

    struct LeagueListWrapper
    {
        std::unique_ptr<poe::LeagueList> leagues;
    };

} // namespace poe
