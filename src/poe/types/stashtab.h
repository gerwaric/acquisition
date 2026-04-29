// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2024 Tom Holz

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
        }
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
