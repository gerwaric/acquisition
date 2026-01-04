// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2024 Tom Holz

#pragma once

#include <QString>

#include <optional>
#include <vector>

#include <glaze/glaze.hpp>

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
