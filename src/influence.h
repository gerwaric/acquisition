// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2020 testpushpleaseignore <elnino2k10@gmail.com>

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
constexpr const char *shaper_symbol_Link = ":/tooltip/symbols/ShaperSymbol.png";
constexpr const char *elder_symbol_Link = ":/tooltip/symbols/ElderSymbol.png";
constexpr const char *crusader_symbol_Link = ":/tooltip/symbols/CrusaderSymbol.png";
constexpr const char *hunter_symbol_Link = ":/tooltip/symbols/HunterSymbol.png";
constexpr const char *redeemer_symbol_Link = ":/tooltip/symbols/RedeemerSymbol.png";
constexpr const char *warlord_symbol_Link = ":/tooltip/symbols/WarlordSymbol.png";
constexpr const char *synthesised_symbol_Link = ":/tooltip/symbols/SynthesisedSymbol.png";
constexpr const char *fractured_symbol_Link = ":/tooltip/symbols/FracturedSymbol.png";
constexpr const char *searing_exarch_symbol_Link = ":/tooltip/symbols/SearingExarchSymbol.png";
constexpr const char *eater_of_worlds_symbol_Link = ":/tooltip/symbols/EaterOfWorldsSymbol.png";

QIcon combineInflunceIcons(const QIcon &leftIcon, const QIcon &rightIcon);
