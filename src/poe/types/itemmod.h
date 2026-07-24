// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2024 Tom Holz

#pragma once

#include <optional>

#include <QString>

namespace poe {

    // https://www.pathofexile.com/developer/docs/reference#type-ItemMod

    struct ItemMod
    {
        struct Flags
        {
            std::optional<bool> fractured;  // ? bool always true if present
            std::optional<bool> mutated;    // ? bool always true if present
            std::optional<bool> crafted;    // ? bool always true if present
            std::optional<bool> desecrated; // ? bool PoE2 only always true if present
            std::optional<bool> vestigial;  // ? bool PoE1 only; always true if present
        };

        QString description;                 // string
        std::optional<ItemMod::Flags> flags; // ? object
    };

} // namespace poe
