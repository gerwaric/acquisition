// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2024 Tom Holz

#pragma once

#include <vector>

#include <QString>

namespace poe {

    // https://www.pathofexile.com/developer/docs/reference#type-LeagueAccount

    struct LeagueAccount
    {
        struct AtlasPassiveTree
        {
            QString name;                 // string
            std::vector<unsigned> hashes; // array of uint
        };

        std::vector<poe::LeagueAccount::AtlasPassiveTree> atlas_passive_trees; // array of object
    };

} // namespace poe
