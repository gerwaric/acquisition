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

#include <poe/types/item.h>

#include <QString>

#include <optional>
#include <vector>

namespace poe {

    struct WebStashTab {

        struct Colour {
            unsigned r;
            unsigned g;
            unsigned b;
        };

        struct Tab {
            QString n;
            unsigned i;
            QString id;
            QString type;
            bool selected;
            poe::WebStashTab::Colour colour;
            QString srcL;
            QString srcC;
            QString srcR;
        };

        unsigned numTabs;
        std::vector<poe::WebStashTab::Tab> tabs;
        std::optional<bool> quadLayout;
        std::vector<poe::Item> items;
    };

}
