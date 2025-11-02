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

#include "poe/types/item.h"

namespace poe {

    struct WebStashTab
    {
        struct Colour
        {
            unsigned r;
            unsigned g;
            unsigned b;
        };

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

    struct WebStashListWrapper
    {
        std::optional<glz::raw_json> blightLayout;
        std::optional<glz::raw_json> currencyLayout;
        std::optional<glz::raw_json> deliriumLayout;
        std::optional<glz::raw_json> delveLayout;
        std::optional<glz::raw_json> divinationLayout;
        std::optional<glz::raw_json> essenceLayout;
        std::optional<glz::raw_json> flaskLayout;
        std::optional<glz::raw_json> fragmentLayout;
        std::optional<glz::raw_json> gemLayout;
        std::optional<glz::raw_json> mapLayout;
        std::optional<glz::raw_json> ultimatumLayout;
        std::optional<glz::raw_json> uniqueLayout;

        std::vector<poe::Item> items;
        unsigned numTabs;

        std::optional<std::vector<poe::WebStashTab>> tabs;
        std::optional<bool> quadLayout;
    };

} // namespace poe
