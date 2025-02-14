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

#include "buyout.h"

#include <QsLog/QsLog.h>

const QString Buyout::m_buyout_type_error;

const Buyout::BuyoutTypeMap Buyout::m_buyout_type_as_tag = {
    {BUYOUT_TYPE_IGNORE, "[ignore]"},
    {BUYOUT_TYPE_BUYOUT, "b/o"},
    {BUYOUT_TYPE_FIXED, "price"},
    {BUYOUT_TYPE_NO_PRICE, "no price"},
    {BUYOUT_TYPE_CURRENT_OFFER, "c/o"},
    {BUYOUT_TYPE_INHERIT, ""}
};

const Buyout::BuyoutTypeMap Buyout::m_buyout_type_as_prefix = {
    {BUYOUT_TYPE_IGNORE, ""},
    {BUYOUT_TYPE_BUYOUT, " ~b/o "},
    {BUYOUT_TYPE_FIXED, " ~price "},
    {BUYOUT_TYPE_CURRENT_OFFER, " ~c/o "},
    {BUYOUT_TYPE_NO_PRICE, ""},
    {BUYOUT_TYPE_INHERIT, ""}
};

const Buyout::BuyoutSourceMap Buyout::m_buyout_source_as_tag = {
    {BUYOUT_SOURCE_NONE, ""},
    {BUYOUT_SOURCE_MANUAL, "manual"},
    {BUYOUT_SOURCE_GAME, "game"},
    {BUYOUT_SOURCE_AUTO, "auto"}
};

bool Buyout::IsValid() const {
    switch (type) {
    case BUYOUT_TYPE_IGNORE:
    case BUYOUT_TYPE_INHERIT:
    case BUYOUT_TYPE_NO_PRICE:
        return true;
    case BUYOUT_TYPE_BUYOUT:
    case BUYOUT_TYPE_FIXED:
    case BUYOUT_TYPE_CURRENT_OFFER:
        return (currency != CURRENCY_NONE) && (source != BUYOUT_SOURCE_NONE);
    default:
        QLOG_ERROR() << "Invalid buyout type:" << type;
        return false;
    };
}

bool Buyout::IsActive() const {
    return IsValid() && type != BUYOUT_TYPE_INHERIT;
}

bool Buyout::IsPostable() const {
    return ((source != BUYOUT_SOURCE_GAME) && (IsPriced() || (type == BUYOUT_TYPE_NO_PRICE)));
}

bool Buyout::IsPriced() const {
    return (type == BUYOUT_TYPE_BUYOUT || type == BUYOUT_TYPE_FIXED || type == BUYOUT_TYPE_CURRENT_OFFER);
}

bool Buyout::IsGameSet() const {
    return (source == BUYOUT_SOURCE_GAME);
}

bool Buyout::RequiresRefresh() const
{
    return !(type == BUYOUT_TYPE_IGNORE || type == BUYOUT_TYPE_INHERIT);
}

BuyoutSource Buyout::TagAsBuyoutSource(QString tag) {
    auto& m = m_buyout_source_as_tag;
    auto const& it = std::find_if(m.begin(), m.end(), [&](BuyoutSourceMap::value_type const& x) { return x.second == tag; });
    return (it != m.end()) ? it->first : BUYOUT_SOURCE_NONE;
}

BuyoutType Buyout::TagAsBuyoutType(QString tag) {
    auto& m = m_buyout_type_as_tag;
    auto const& it = std::find_if(m.begin(), m.end(), [&](BuyoutTypeMap::value_type const& x) { return x.second == tag; });
    return (it != m.end()) ? it->first : BUYOUT_TYPE_INHERIT;
}

BuyoutType Buyout::IndexAsBuyoutType(int index) {
    if (index >= 0) {
        size_t index_t = (size_t)index;
        if (index_t < m_buyout_type_as_tag.size()) {
            return static_cast<BuyoutType>(index_t);
        };
    };

    QLOG_WARN() << "Buyout type index out of bounds: " << index << ". This should never happen - please report.";
    return BUYOUT_TYPE_INHERIT;
}

QString Buyout::AsText() const {
    if (IsPriced()) {
        return BuyoutTypeAsTag() + " " + QString::number(value) + " " + CurrencyAsTag();
    } else {
        return BuyoutTypeAsTag();
    };
}

const QString& Buyout::BuyoutTypeAsTag() const {
    auto const& it = m_buyout_type_as_tag.find(type);
    if (it != m_buyout_type_as_tag.end()) {
        return it->second;
    } else {
        QLOG_WARN() << "No mapping from buyout type: " << type << " to tag. This should never happen - please report.";
        return m_buyout_type_error;
    };
}

const QString& Buyout::BuyoutTypeAsPrefix() const {
    auto const& it = m_buyout_type_as_prefix.find(type);
    if (it != m_buyout_type_as_prefix.end()) {
        return it->second;
    } else {
        QLOG_WARN() << "No mapping from buyout type: " << type << " to prefix. This should never happen - please report.";
        return m_buyout_type_error;
    };
}

const QString& Buyout::BuyoutSourceAsTag() const {
    auto const& it = m_buyout_source_as_tag.find(source);
    if (it != m_buyout_source_as_tag.end()) {
        return it->second;
    } else {
        QLOG_WARN() << "No mapping from buyout source: " << source << " to tag. This should never happen - please report.";
        return m_buyout_type_error;
    };
}

const QString& Buyout::CurrencyAsTag() const
{
    return currency.AsTag();
}

bool Buyout::operator==(const Buyout& o) const {
    static const double eps = 1e-6;
    return std::fabs(o.value - value) < eps
        && o.type == type
        && o.currency == currency
        && o.inherited == inherited
        && o.source == source;
}

bool Buyout::operator!=(const Buyout& o) const {
    return !(o == *this);
}
