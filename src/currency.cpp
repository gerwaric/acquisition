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

#include "currency.h"

#include <util/spdlog_qt.h>

const QString Currency::m_currency_type_error;

const Currency::TypeStringMap Currency::m_currency_type_as_string = {
    {CURRENCY_NONE, ""},
    {CURRENCY_ORB_OF_ALTERATION, "Orb of Alteration"},
    {CURRENCY_ORB_OF_FUSING, "Orb of Fusing"},
    {CURRENCY_ORB_OF_ALCHEMY, "Orb of Alchemy"},
    {CURRENCY_CHAOS_ORB, "Chaos Orb"},
    {CURRENCY_GCP, "Gemcutter's Prism"},
    {CURRENCY_EXALTED_ORB, "Exalted Orb"},
    {CURRENCY_CHROMATIC_ORB, "Chromatic Orb"},
    {CURRENCY_JEWELLERS_ORB, "Jeweller's Orb"},
    {CURRENCY_ORB_OF_CHANCE, "Orb of Chance"},
    {CURRENCY_CARTOGRAPHERS_CHISEL, "Cartographer's Chisel"},
    {CURRENCY_ORB_OF_SCOURING, "Orb of Scouring"},
    {CURRENCY_BLESSED_ORB, "Blessed Orb"},
    {CURRENCY_ORB_OF_REGRET, "Orb of Regret"},
    {CURRENCY_REGAL_ORB, "Regal Orb"},
    {CURRENCY_DIVINE_ORB, "Divine Orb"},
    {CURRENCY_VAAL_ORB, "Vaal Orb"},
    {CURRENCY_PERANDUS_COIN, "Perandus Coin"},
    {CURRENCY_MIRROR_OF_KALANDRA, "Mirror of Kalandra"},
    {CURRENCY_SILVER_COIN, "Silver Coin"},
};

const Currency::TypeStringMap Currency::m_currency_type_as_tag = {
    {CURRENCY_NONE, ""},
    {CURRENCY_ORB_OF_ALTERATION, "alt"},
    {CURRENCY_ORB_OF_FUSING, "fuse"},
    {CURRENCY_ORB_OF_ALCHEMY, "alch"},
    {CURRENCY_CHAOS_ORB, "chaos"},
    {CURRENCY_GCP, "gcp"},
    {CURRENCY_EXALTED_ORB, "exa"},
    {CURRENCY_CHROMATIC_ORB, "chrom"},
    {CURRENCY_JEWELLERS_ORB, "jew"},
    {CURRENCY_ORB_OF_CHANCE, "chance"},
    {CURRENCY_CARTOGRAPHERS_CHISEL, "chisel"},
    {CURRENCY_ORB_OF_SCOURING, "scour"},
    {CURRENCY_BLESSED_ORB, "blessed"},
    {CURRENCY_ORB_OF_REGRET, "regret"},
    {CURRENCY_REGAL_ORB, "regal"},
    {CURRENCY_DIVINE_ORB, "divine"},
    {CURRENCY_VAAL_ORB, "vaal"},
    {CURRENCY_PERANDUS_COIN, "coin"},
    {CURRENCY_MIRROR_OF_KALANDRA, "mirror"},
    {CURRENCY_SILVER_COIN, "silver"},
};

const Currency::TypeIntMap Currency::m_currency_type_as_rank = {
    {CURRENCY_NONE, 0},
    {CURRENCY_CHROMATIC_ORB, 1},
    {CURRENCY_ORB_OF_ALTERATION, 2},
    {CURRENCY_JEWELLERS_ORB, 3},
    {CURRENCY_ORB_OF_CHANCE, 4},
    {CURRENCY_CARTOGRAPHERS_CHISEL, 5},
    {CURRENCY_PERANDUS_COIN, 6},
    {CURRENCY_ORB_OF_FUSING, 7},
    {CURRENCY_ORB_OF_ALCHEMY, 8},
    {CURRENCY_BLESSED_ORB, 9},
    {CURRENCY_ORB_OF_SCOURING, 10},
    {CURRENCY_CHAOS_ORB, 11},
    {CURRENCY_ORB_OF_REGRET, 12},
    {CURRENCY_REGAL_ORB, 13},
    {CURRENCY_VAAL_ORB, 14},
    {CURRENCY_GCP, 15},
    {CURRENCY_DIVINE_ORB, 16},
    {CURRENCY_EXALTED_ORB, 17},
    {CURRENCY_MIRROR_OF_KALANDRA, 18},
    {CURRENCY_SILVER_COIN, 19},
};

const Currency::StringTypeMap Currency::m_string_to_currency_type = {
    {"alch", CURRENCY_ORB_OF_ALCHEMY},
    {"alchs", CURRENCY_ORB_OF_ALCHEMY},
    {"alchemy", CURRENCY_ORB_OF_ALCHEMY},
    {"alt", CURRENCY_ORB_OF_ALTERATION},
    {"alts", CURRENCY_ORB_OF_ALTERATION},
    {"alteration", CURRENCY_ORB_OF_ALTERATION},
    {"alterations", CURRENCY_ORB_OF_ALTERATION},
    {"blessed", CURRENCY_BLESSED_ORB},
    {"cartographer", CURRENCY_CARTOGRAPHERS_CHISEL},
    {"cartographers", CURRENCY_CARTOGRAPHERS_CHISEL},
    {"chance", CURRENCY_ORB_OF_CHANCE},
    {"chaos", CURRENCY_CHAOS_ORB},
    {"chisel", CURRENCY_CARTOGRAPHERS_CHISEL},
    {"chisels", CURRENCY_CARTOGRAPHERS_CHISEL},
    {"chrom", CURRENCY_CHROMATIC_ORB},
    {"chrome", CURRENCY_CHROMATIC_ORB},
    {"chromes", CURRENCY_CHROMATIC_ORB},
    {"chromatic", CURRENCY_CHROMATIC_ORB},
    {"chromatics", CURRENCY_CHROMATIC_ORB},
    {"coin", CURRENCY_PERANDUS_COIN},
    {"coins", CURRENCY_PERANDUS_COIN},
    {"divine", CURRENCY_DIVINE_ORB},
    {"divines", CURRENCY_DIVINE_ORB},
    {"exa", CURRENCY_EXALTED_ORB},
    {"exalted", CURRENCY_EXALTED_ORB},
    {"fuse", CURRENCY_ORB_OF_FUSING},
    {"fuses", CURRENCY_ORB_OF_FUSING},
    {"fusing", CURRENCY_ORB_OF_FUSING},
    {"fusings", CURRENCY_ORB_OF_FUSING},
    {"gcp", CURRENCY_GCP},
    {"gcps", CURRENCY_GCP},
    {"gemcutter", CURRENCY_GCP},
    {"gemcutters", CURRENCY_GCP},
    {"jew", CURRENCY_JEWELLERS_ORB},
    {"jewel", CURRENCY_JEWELLERS_ORB},
    {"jewels", CURRENCY_JEWELLERS_ORB},
    {"jeweler", CURRENCY_JEWELLERS_ORB},
    {"jewelers", CURRENCY_JEWELLERS_ORB},
    {"mir", CURRENCY_MIRROR_OF_KALANDRA},
    {"mirror", CURRENCY_MIRROR_OF_KALANDRA},
    {"p", CURRENCY_PERANDUS_COIN},
    {"perandus", CURRENCY_PERANDUS_COIN},
    {"regal", CURRENCY_REGAL_ORB},
    {"regals", CURRENCY_REGAL_ORB},
    {"regret", CURRENCY_ORB_OF_REGRET},
    {"regrets", CURRENCY_ORB_OF_REGRET},
    {"scour", CURRENCY_ORB_OF_SCOURING},
    {"scours", CURRENCY_ORB_OF_SCOURING},
    {"scouring", CURRENCY_ORB_OF_SCOURING},
    {"shekel", CURRENCY_PERANDUS_COIN},
    {"vaal", CURRENCY_VAAL_ORB},
    {"silver", CURRENCY_SILVER_COIN},
};

std::vector<Currency::CurrencyType> Currency::Types()
{
    std::vector<Currency::CurrencyType> tmp;
    for (unsigned int i = 0; i < m_currency_type_as_tag.size(); i++) {
        tmp.push_back(static_cast<CurrencyType>(i));
    }
    return tmp;
}

Currency Currency::FromTag(const QString &tag)
{
    auto &m = m_currency_type_as_tag;
    auto const &it = std::find_if(m.begin(),
                                  m.end(),
                                  [&](Currency::TypeStringMap::value_type const &x) {
                                      return x.second == tag;
                                  });
    return Currency((it != m.end()) ? it->first : CURRENCY_NONE);
}

Currency Currency::FromIndex(int index)
{
    if (static_cast<unsigned int>(index) >= m_currency_type_as_tag.size()) {
        spdlog::warn(
            "Currency type index out of bounds: {}. This should never happen - please report.",
            index);
        return CURRENCY_NONE;
    } else {
        return Currency(static_cast<CurrencyType>(index));
    }
}

Currency Currency::FromString(const QString &currency)
{
    auto const &it = m_string_to_currency_type.find(currency);
    if (it != m_string_to_currency_type.end()) {
        return it->second;
    }
    return CURRENCY_NONE;
}

const QString &Currency::AsString() const
{
    auto const &it = m_currency_type_as_string.find(type);
    if (it != m_currency_type_as_string.end()) {
        return it->second;
    } else {
        spdlog::warn("No mapping from currency type: {} to string. This should never happen - "
                     "please report.",
                     type);
        return m_currency_type_error;
    }
}

const QString &Currency::AsTag() const
{
    auto const &it = m_currency_type_as_tag.find(type);
    if (it != m_currency_type_as_tag.end()) {
        return it->second;
    } else {
        spdlog::warn(
            "No mapping from currency type: {} to tag. This should never happen - please report.",
            type);
        return m_currency_type_error;
    }
}

const int &Currency::AsRank() const
{
    return m_currency_type_as_rank.at(type);
}
