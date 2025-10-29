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

#include <vector>

#include <QString>

#include <glaze/glaze.hpp>

#include "poe/types/item.h"

namespace poe {

    struct WebCharacter {
        QString name;
        QString realm;
        QString class_;
        QString league;
        unsigned level;
        bool pinnable;
    };

    struct WebCharacterList {
        std::vector<poe::WebCharacter> characters;
    };

    struct WebCharacterStash {
        std::vector<poe::Item> items;
    };

}

template<>
struct glz::meta<poe::WebCharacter>
{
    using T = poe::WebCharacter;
    static constexpr auto modify = glz::object("class", &T::class_);
};
