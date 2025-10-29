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

#include <glaze/glaze.hpp>

#include "poe/types/item.h"

namespace poe {

    // https://www.pathofexile.com/developer/docs/reference#type-StashTab

    struct StashTab
    {
        struct Metadata
        {
            // WARNING: The colour field can sometimes be 2 or 4 characters (which means it needs to be zero-padded)
            std::optional<bool> public_;         // ?bool always true if present
            std::optional<bool> folder;          // ?bool always true if present
            std::optional<QString> colour;       // ? string 6 digit hex colour
            std::optional<glz::raw_json> layout; // TODO: UNDOCUMENTED
            std::optional<unsigned> items;       // TODO: UNDOCUMENTED
            std::optional<glz::raw_json> map;    // ?object various game specific properties
        };

        QString id;                                         // string a 10 digit hexadecimal string
        std::optional<QString> parent;                      // ?string a 10 digit hexadecimal string
        std::optional<QString> folder;                      // ?string a 10 digit hexadecimal string
        QString name;                                       // string
        QString type;                                       // string
        std::optional<unsigned> index;                      // ?uint
        poe::StashTab::Metadata metadata;                   // metadata object
        std::optional<std::vector<poe::StashTab>> children; // ?array of StashTab
        std::optional<std::vector<poe::Item>> items;        // ?array of Item

        inline bool operator<(const StashTab &other) const
        {
            const unsigned a = index.value_or(0);
            const unsigned b = other.index.value_or(0);
            return a < b;
        };
    };

    struct StashListWrapper
    {
        std::vector<poe::StashTab> stashes;
    };

    struct StashWrapper
    {
        std::optional<poe::StashTab> stash;
    };

}; // namespace poe

template<>
struct glz::meta<poe::StashTab::Metadata>
{
    using T = poe::StashTab::Metadata;
    static constexpr auto modify = glz::object("public", &T::public_);
};
