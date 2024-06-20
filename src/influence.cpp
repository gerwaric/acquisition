#include "influence.h"
#include <QImage>
#include <QIcon>
#include <QPainter>

const char* elder_1x1_Link = ":/backgrounds/ElderBackground_1x1.png";
const char* elder_1x3_Link = ":/backgrounds/ElderBackground_1x3.png";
const char* elder_1x4_Link = ":/backgrounds/ElderBackground_1x4.png";
const char* elder_2x1_Link = ":/backgrounds/ElderBackground_2x1.png";
const char* elder_2x2_Link = ":/backgrounds/ElderBackground_2x2.png";
const char* elder_2x3_Link = ":/backgrounds/ElderBackground_2x3.png";
const char* elder_2x4_Link = ":/backgrounds/ElderBackground_2x4.png";
const char* shaper_1x1_Link = ":/backgrounds/ShaperBackground_1x1.png";
const char* shaper_1x3_Link = ":/backgrounds/ShaperBackground_1x3.png";
const char* shaper_1x4_Link = ":/backgrounds/ShaperBackground_1x4.png";
const char* shaper_2x1_Link = ":/backgrounds/ShaperBackground_2x1.png";
const char* shaper_2x2_Link = ":/backgrounds/ShaperBackground_2x2.png";
const char* shaper_2x3_Link = ":/backgrounds/ShaperBackground_2x3.png";
const char* shaper_2x4_Link = ":/backgrounds/ShaperBackground_2x4.png";
const char* shaper_symbol_Link = ":/tooltip/ShaperItemSymbol.png";
const char* elder_symbol_Link = ":/tooltip/ElderItemSymbol.png";
const char* crusader_symbol_Link = ":/tooltip/Crusader-item-symbol.png";
const char* hunter_symbol_Link = ":/tooltip/Hunter-item-symbol.png";
const char* redeemer_symbol_Link = ":/tooltip/Redeemer-item-symbol.png";
const char* warlord_symbol_Link = ":/tooltip/Warlord-item-symbol.png";
const char* synthesised_symbol_Link = ":/tooltip/Synthesised-item-symbol.png";
const char* fractured_symbol_Link = ":/tooltip/Fractured-item-symbol.png";
const char* searing_exarch_symbol_Link = ":/tooltip/Searing-exarch-item-symbol.png";
const char* eater_of_worlds_symbol_Link = ":/tooltip/Eater-of-worlds-item-symbol.png";

QIcon combineInflunceIcons(const QIcon leftIcon, const QIcon rightIcon) {
	const int width = 54;
	const int height = 27;

	QPixmap layered(width, height);
	layered.fill(Qt::transparent);
	QPainter layered_painter(&layered);

	layered_painter.drawPixmap(0, 0, leftIcon.pixmap(27, 27));
	layered_painter.drawPixmap(27, 0, rightIcon.pixmap(27, 27));

	return QIcon(layered);
}
