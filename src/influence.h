/*
    Copyright (C) 2014-2025 Acquisition Contributors

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

#include <QIcon>
#include <QString>

constexpr const char *elder_1x1_Link = ":/backgrounds/ElderBackground_1x1.png";
constexpr const char *elder_1x3_Link = ":/backgrounds/ElderBackground_1x3.png";
constexpr const char *elder_1x4_Link = ":/backgrounds/ElderBackground_1x4.png";
constexpr const char *elder_2x1_Link = ":/backgrounds/ElderBackground_2x1.png";
constexpr const char *elder_2x2_Link = ":/backgrounds/ElderBackground_2x2.png";
constexpr const char *elder_2x3_Link = ":/backgrounds/ElderBackground_2x3.png";
constexpr const char *elder_2x4_Link = ":/backgrounds/ElderBackground_2x4.png";
constexpr const char *shaper_1x1_Link = ":/backgrounds/ShaperBackground_1x1.png";
constexpr const char *shaper_1x3_Link = ":/backgrounds/ShaperBackground_1x3.png";
constexpr const char *shaper_1x4_Link = ":/backgrounds/ShaperBackground_1x4.png";
constexpr const char *shaper_2x1_Link = ":/backgrounds/ShaperBackground_2x1.png";
constexpr const char *shaper_2x2_Link = ":/backgrounds/ShaperBackground_2x2.png";
constexpr const char *shaper_2x3_Link = ":/backgrounds/ShaperBackground_2x3.png";
constexpr const char *shaper_2x4_Link = ":/backgrounds/ShaperBackground_2x4.png";
constexpr const char *shaper_symbol_Link = ":/tooltip/ShaperItemSymbol.png";
constexpr const char *elder_symbol_Link = ":/tooltip/ElderItemSymbol.png";
constexpr const char *crusader_symbol_Link = ":/tooltip/Crusader-item-symbol.png";
constexpr const char *hunter_symbol_Link = ":/tooltip/Hunter-item-symbol.png";
constexpr const char *redeemer_symbol_Link = ":/tooltip/Redeemer-item-symbol.png";
constexpr const char *warlord_symbol_Link = ":/tooltip/Warlord-item-symbol.png";
constexpr const char *synthesised_symbol_Link = ":/tooltip/Synthesised-item-symbol.png";
constexpr const char *fractured_symbol_Link = ":/tooltip/Fractured-item-symbol.png";
constexpr const char *searing_exarch_symbol_Link = ":/tooltip/Searing-exarch-item-symbol.png";
constexpr const char *eater_of_worlds_symbol_Link = ":/tooltip/Eater-of-worlds-item-symbol.png";

QIcon combineInflunceIcons(const QIcon &leftIcon, const QIcon &rightIcon);
