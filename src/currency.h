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

#pragma once

#include <map>
#include <string>
#include <vector>

enum CurrencyType {
    CURRENCY_NONE,
    CURRENCY_ORB_OF_ALTERATION,
    CURRENCY_ORB_OF_FUSING,
    CURRENCY_ORB_OF_ALCHEMY,
    CURRENCY_CHAOS_ORB,
    CURRENCY_GCP,
    CURRENCY_EXALTED_ORB,
    CURRENCY_CHROMATIC_ORB,
    CURRENCY_JEWELLERS_ORB,
    CURRENCY_ORB_OF_CHANCE,
    CURRENCY_CARTOGRAPHERS_CHISEL,
    CURRENCY_ORB_OF_SCOURING,
    CURRENCY_BLESSED_ORB,
    CURRENCY_ORB_OF_REGRET,
    CURRENCY_REGAL_ORB,
    CURRENCY_DIVINE_ORB,
    CURRENCY_VAAL_ORB,
    CURRENCY_PERANDUS_COIN,
    CURRENCY_MIRROR_OF_KALANDRA,
    CURRENCY_SILVER_COIN,
};

struct Currency {
    typedef std::map<CurrencyType, std::string> TypeStringMap;
    typedef std::map<CurrencyType, int> TypeIntMap;
    typedef std::map<std::string, CurrencyType> StringTypeMap;

    Currency() = default;
    Currency(CurrencyType in_type) : type(in_type) { };

    CurrencyType type{ CURRENCY_NONE };

    static std::vector<CurrencyType> Types();
    static Currency FromTag(const std::string& tag);
    static Currency FromIndex(int i);
    static Currency FromString(const std::string& currency);

    bool operator==(const Currency& rhs) const { return type == rhs.type; }
    bool operator!=(const Currency& rhs) const { return type != rhs.type; }
    bool operator<(const Currency& rhs) const { return type < rhs.type; }

    const std::string& AsString() const;
    const std::string& AsTag() const;
    const int& AsRank() const;

private:
    static const std::string m_currency_type_error;
    static const Currency::TypeStringMap m_currency_type_as_string;
    static const Currency::TypeStringMap m_currency_type_as_tag;
    static const Currency::TypeIntMap m_currency_type_as_rank;
    static const Currency::StringTypeMap m_string_to_currency_type;
};
