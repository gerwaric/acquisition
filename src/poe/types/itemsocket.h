// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2024 Tom Holz

#pragma once

#include <optional>

#include <QString>

namespace poe {

    // https://www.pathofexile.com/developer/docs/reference#type-ItemSocket

    struct ItemSocket
    {
        unsigned group;                 // uint
        std::optional<QString> attr;    // ?string; PoE1 only; S, D, I, G, A, or DV
        std::optional<QString> sColour; // ?string; PoE1 only; R, G, B, W, A, or DV
        std::optional<QString> type;    // ?string; PoE2 only; gem, jewel, or rune
        std::optional<QString>
            item; // ?string; PoE2 only; emerald, sapphire, ruby, rune, soulcore, primaltalisman, vividtalisman, wildtalisman, sacredtalisman, activegem, or supportgem
    };

} // namespace poe
