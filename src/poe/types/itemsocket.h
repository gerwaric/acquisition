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
