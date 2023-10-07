#include <QString>

#include "util.h"
#include "QsLog.h"

#ifndef INFLUENCE_H
#define INFLUENCE_H

static const QString elder_1x1_Link = ":/backgrounds/ElderBackground_1x1.png";
static const QString elder_1x3_Link = ":/backgrounds/ElderBackground_1x3.png";
static const QString elder_1x4_Link = ":/backgrounds/ElderBackground_1x4.png";
static const QString elder_2x1_Link = ":/backgrounds/ElderBackground_2x1.png";
static const QString elder_2x2_Link = ":/backgrounds/ElderBackground_2x2.png";
static const QString elder_2x3_Link = ":/backgrounds/ElderBackground_2x3.png";
static const QString elder_2x4_Link = ":/backgrounds/ElderBackground_2x4.png";
static const QString shaper_1x1_Link = ":/backgrounds/ShaperBackground_1x1.png";
static const QString shaper_1x3_Link = ":/backgrounds/ShaperBackground_1x3.png";
static const QString shaper_1x4_Link = ":/backgrounds/ShaperBackground_1x4.png";
static const QString shaper_2x1_Link = ":/backgrounds/ShaperBackground_2x1.png";
static const QString shaper_2x2_Link = ":/backgrounds/ShaperBackground_2x2.png";
static const QString shaper_2x3_Link = ":/backgrounds/ShaperBackground_2x3.png";
static const QString shaper_2x4_Link = ":/backgrounds/ShaperBackground_2x4.png";
static const QString shaper_symbol_Link = ":/tooltip/ShaperItemSymbol.png";
static const QString elder_symbol_Link = ":/tooltip/ElderItemSymbol.png";
static const QString crusader_symbol_Link = ":/tooltip/Crusader-item-symbol.png";
static const QString hunter_symbol_Link = ":/tooltip/Hunter-item-symbol.png";
static const QString redeemer_symbol_Link = ":/tooltip/Redeemer-item-symbol.png";
static const QString warlord_symbol_Link = ":/tooltip/Warlord-item-symbol.png";
static const QString synthesised_symbol_Link = ":/tooltip/Synthesised-item-symbol.png";
static const QString fractured_symbol_Link = ":/tooltip/Fractured-item-symbol.png";
static const QString searing_exarch_symbol_Link = ":/tooltip/Searing-exarch-item-symbol.png";
static const QString eater_of_worlds_symbol_Link = ":/tooltip/Eater-of-worlds-item-symbol.png";

class Influence {


public:
	static QIcon combineInflunceIcons(const QIcon leftIcon, const QIcon rightIcon);

private:
	Influence() {}
};

#endif // INFLUENCE_H
