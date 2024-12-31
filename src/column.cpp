/*
    Copyright (C) 2014-2024 Acquisition Contributors

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

#include "column.h"

#include <cmath>
#include <QVector>
#include <QRegularExpression>
#include <QApplication>
#include <QPalette>
#include <QIcon>

#include "util/util.h"

#include "buyoutmanager.h"
#include "influence.h"
#include "itemconstants.h"

const double EPS = 1e-6;
const char* SORT_DOUBLE_MATCH = "^\\+?([\\d.]+)%?$";
const char* SORT_TWO_VALUES = "^(\\d+)([-/])(\\d+)$";

QColor Column::color(const Item& /* item */) const {
    return QApplication::palette().color(QPalette::WindowText);
}

Column::sort_tuple Column::multivalue(const Item* item) const {

    // Transform values into something optimal for sorting
    // Possibilities: 12, 12.12, 10%, 10.13%, +16%, 12-14, 10/20
    static const QRegularExpression sort_double_match(SORT_DOUBLE_MATCH);
    static const QRegularExpression sort_two_values(SORT_TWO_VALUES);

    double first_double = 0.0;
    double second_double = 0.0;
    std::string first_string = "";
    std::string second_string = "";

    QString str = value(*item).toString();
    QRegularExpressionMatch match;

    if (str.contains(sort_double_match, &match)) {
        first_double = match.captured(1).toDouble();
    } else if (str.contains(sort_two_values, &match)) {
        if (match.captured(2).startsWith("-")) {
            first_double = 0.5 * (match.captured(1).toDouble() + match.captured(3).toDouble());
        } else {
            first_string = item->PrettyName();
            second_double = match.captured(1).toDouble();
        }
    } else {
        first_string = str.toStdString();
        second_string = item->PrettyName().c_str();
    }

    return std::forward_as_tuple(
        first_double,
        first_string,
        second_double,
        second_string, *item);
}

bool Column::lt(const Item* lhs, const Item* rhs) const {
    return multivalue(lhs) < multivalue(rhs);
}

std::string NameColumn::name() const {
    return "Name";
}

QVariant NameColumn::value(const Item& item) const {
    return item.PrettyName().c_str();
}

QColor NameColumn::color(const Item& item) const {
    switch (item.frameType()) {
    case FRAME_TYPE_MAGIC:
        return QColor(0x00, 0x66, 0x99);
    case FRAME_TYPE_RARE:
        return QColor(Qt::darkYellow);
    case FRAME_TYPE_UNIQUE:
        return QColor(234, 117, 0);
    case FRAME_TYPE_GEM:
        return QColor(0x1b, 0xa2, 0x9b);
    case FRAME_TYPE_CURRENCY:
        return QColor(0x77, 0x6e, 0x59);
    case FRAME_TYPE_DIVINATION_CARD:
        return QColor(QRgb(0x01bcba));
    case FRAME_TYPE_QUEST_ITEM:
        return QColor(QRgb(0x4ae63a));
    case FRAME_TYPE_PROPHECY:
        return QColor(181, 75, 255);
    case FRAME_TYPE_RELIC:
        return QColor(QRgb(0x82ad6a));
    default:
        return QApplication::palette().color(QPalette::WindowText);
    }
}

std::string CorruptedColumn::name() const {
    return "Corr";
}

QVariant CorruptedColumn::value(const Item& item) const {
    if (item.corrupted())
        return "C";
    return QVariant();
}

std::string CraftedColumn::name() const {
    return "Mast";
}

QVariant CraftedColumn::value(const Item& item) const {
    if (item.crafted())
        return "M";
    return QVariant();
}

std::string EnchantedColumn::name() const {
    return "Ench";
}

QVariant EnchantedColumn::value(const Item& item) const {
    if (item.enchanted())
        return "En";
    return QVariant();
}

std::string InfluncedColumn::name() const {
    return "Inf";
}

QVariant InfluncedColumn::value(const Item& item) const {
    QString influencesString = "";

    if (item.hasInfluence(Item::SHAPER))
        influencesString += "S";
    if (item.hasInfluence(Item::ELDER))
        influencesString += "E";
    if (item.hasInfluence(Item::HUNTER))
        influencesString += "H";
    if (item.hasInfluence(Item::WARLORD))
        influencesString += "W";
    if (item.hasInfluence(Item::CRUSADER))
        influencesString += "C";
    if (item.hasInfluence(Item::REDEEMER))
        influencesString += "R";

    //TODO maybe add an option to toggle whether to send an icon or text for an influence?
    return "";
}

QVariant InfluncedColumn::icon(const Item& item) const {
    QIcon leftIcon, rightIcon;

    Item::INFLUENCE_TYPES leftInf = item.influenceLeft();
    Item::INFLUENCE_TYPES rightInf = item.influenceRight();

    int numInfluences = rightInf != Item::NONE && rightInf != leftInf ? 2 : item.influenceLeft() != Item::NONE ? 1 : 0;

    if (numInfluences > 0) {
        switch (item.influenceLeft()) {
        case Item::ELDER: leftIcon.addFile(elder_symbol_Link); break;
        case Item::SHAPER: leftIcon.addFile(shaper_symbol_Link); break;
        case Item::CRUSADER: leftIcon.addFile(crusader_symbol_Link); break;
        case Item::HUNTER: leftIcon.addFile(hunter_symbol_Link); break;
        case Item::REDEEMER: leftIcon.addFile(redeemer_symbol_Link); break;
        case Item::WARLORD: leftIcon.addFile(warlord_symbol_Link); break;
        case Item::SYNTHESISED: leftIcon.addFile(synthesised_symbol_Link); break;
        case Item::FRACTURED: leftIcon.addFile(fractured_symbol_Link); break;
        case Item::SEARING_EXARCH: leftIcon.addFile(searing_exarch_symbol_Link); break;
        case Item::EATER_OF_WORLDS: leftIcon.addFile(eater_of_worlds_symbol_Link); break;
        case Item::NONE: break;
        }
        if (numInfluences == 2) {
            switch (item.influenceRight()) {
            case Item::ELDER: rightIcon.addFile(elder_symbol_Link); break;
            case Item::SHAPER: rightIcon.addFile(shaper_symbol_Link); break;
            case Item::CRUSADER: rightIcon.addFile(crusader_symbol_Link); break;
            case Item::HUNTER: rightIcon.addFile(hunter_symbol_Link); break;
            case Item::REDEEMER: rightIcon.addFile(redeemer_symbol_Link); break;
            case Item::WARLORD: rightIcon.addFile(warlord_symbol_Link); break;
            case Item::SYNTHESISED: rightIcon.addFile(synthesised_symbol_Link); break;
            case Item::FRACTURED: rightIcon.addFile(fractured_symbol_Link); break;
            case Item::SEARING_EXARCH: rightIcon.addFile(searing_exarch_symbol_Link); break;
            case Item::EATER_OF_WORLDS: rightIcon.addFile(eater_of_worlds_symbol_Link); break;
            case Item::NONE: break;
            }
            return combineInflunceIcons(leftIcon, rightIcon);
        }
        return leftIcon;
    }
    return QVariant::fromValue(NULL);
}



PropertyColumn::PropertyColumn(const std::string& name) :
    m_name(name),
    m_property(name)
{}

PropertyColumn::PropertyColumn(const std::string& name, const std::string& property) :
    m_name(name),
    m_property(property)
{}

std::string PropertyColumn::name() const {
    return m_name;
}

QVariant PropertyColumn::value(const Item& item) const {
    if (item.properties().count(m_property))
        return item.properties().find(m_property)->second.c_str();
    return QVariant();
}

std::string DPSColumn::name() const {
    return "DPS";
}

QVariant DPSColumn::value(const Item& item) const {
    double dps = item.DPS();
    if (fabs(dps) < EPS)
        return QVariant();
    return dps;
}

std::string pDPSColumn::name() const {
    return "pDPS";
}

QVariant pDPSColumn::value(const Item& item) const {
    double pdps = item.pDPS();
    if (fabs(pdps) < EPS)
        return QVariant();
    return pdps;
}

std::string eDPSColumn::name() const {
    return "eDPS";
}

QVariant eDPSColumn::value(const Item& item) const {
    double edps = item.eDPS();
    if (fabs(edps) < EPS)
        return QVariant();
    return edps;
}

ElementalDamageColumn::ElementalDamageColumn(int index) :
    m_index(index)
{}

std::string ElementalDamageColumn::name() const {
    if (m_index == 0)
        return "ED";
    return "";
}

QVariant ElementalDamageColumn::value(const Item& item) const {
    if (item.elemental_damage().size() > m_index) {
        auto& ed = item.elemental_damage().at(m_index);
        return ed.first.c_str();
    }
    return QVariant();
}

QColor ElementalDamageColumn::color(const Item& item) const {
    if (item.elemental_damage().size() > m_index) {
        auto& ed = item.elemental_damage().at(m_index);
        switch (ed.second) {
        case ED_FIRE:
            return QColor(0xc5, 0x13, 0x13);
        case ED_COLD:
            return QColor(0x36, 0x64, 0x92);
        case ED_LIGHTNING:
            return QColor(0xb9, 0x9c, 0x00);
        }
    }
    return QColor();
}

std::string ChaosDamageColumn::name() const {
    return "CD";
}

QVariant ChaosDamageColumn::value(const Item& item) const {
    if (item.properties().count("Chaos Damage"))
        return item.properties().find("Chaos Damage")->second.c_str();
    return QVariant();
}

QColor ChaosDamageColumn::color(const Item& item) const {
    Q_UNUSED(item);
    return QColor(0xd0, 0x20, 0x90);
}

std::string cDPSColumn::name() const {
    return "cDPS";
}

QVariant cDPSColumn::value(const Item& item) const {
    double cdps = item.cDPS();
    if (fabs(cdps) < EPS)
        return QVariant();
    return cdps;
}

PriceColumn::PriceColumn(const BuyoutManager& bo_manager) :
    m_bo_manager(bo_manager)
{}

std::string PriceColumn::name() const {
    return "Price";
}

QVariant PriceColumn::value(const Item& item) const {
    const Buyout& bo = m_bo_manager.Get(item);
    return bo.AsText().c_str();
}

QColor PriceColumn::color(const Item& item) const {
    const Buyout& bo = m_bo_manager.Get(item);
    return bo.IsInherited() ? QColor(0xaa, 0xaa, 0xaa) : QApplication::palette().color(QPalette::WindowText);
}

std::tuple<int, double, const Item&> PriceColumn::multivalue(const Item* item) const {
    const Buyout& bo = m_bo_manager.Get(*item);
    // forward_as_tuple used to forward item reference properly and avoid ref to temporary
    // that will be destroyed.  We want item reference because we want to sort based on item
    // object itself and not the pointer.  I'm not entirely sure I fully understand
    // this mechanism, but basically want to avoid a ton of Item copies during sorting
    // so trying to avoid pass by value (ericsium).
    return std::forward_as_tuple(bo.currency.AsRank(), bo.value, *item);
}

bool PriceColumn::lt(const Item* lhs, const Item* rhs) const {
    return multivalue(lhs) < multivalue(rhs);
}

DateColumn::DateColumn(const BuyoutManager& bo_manager) :
    m_bo_manager(bo_manager)
{}

std::string DateColumn::name() const {
    return "Last Update";
}

QVariant DateColumn::value(const Item& item) const {
    const Buyout& bo = m_bo_manager.Get(item);
    return bo.IsActive() ? Util::TimeAgoInWords(bo.last_update).c_str() : QVariant();
}

bool DateColumn::lt(const Item* lhs, const Item* rhs) const {
    const QDateTime lhs_update_time = m_bo_manager.Get(*lhs).last_update;
    const QDateTime rhs_update_time = m_bo_manager.Get(*rhs).last_update;
    return (std::tie(lhs_update_time, *lhs) <
        std::tie(rhs_update_time, *rhs));
}

std::string ItemlevelColumn::name() const {
    return "ilvl";
}

QVariant ItemlevelColumn::value(const Item& item) const {
    if (item.ilvl() > 0)
        return item.ilvl();
    return QVariant();
}
