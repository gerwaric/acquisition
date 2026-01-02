// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

#pragma once

#include <QObject>
#include <QString>

#include <map>
#include <vector>

#include <util/spdlog_qt.h>

struct Currency
{
    Q_GADGET;

public:
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
    Q_ENUM(CurrencyType)

    typedef std::map<CurrencyType, QString> TypeStringMap;
    typedef std::map<CurrencyType, int> TypeIntMap;
    typedef std::map<QString, CurrencyType> StringTypeMap;

    Currency() = default;
    Currency(CurrencyType in_type)
        : type(in_type) {};

    CurrencyType type{CurrencyType::CURRENCY_NONE};

    static std::vector<CurrencyType> Types();
    static Currency FromTag(const QString &tag);
    static Currency FromIndex(int i);
    static Currency FromString(const QString &currency);

    bool operator==(const Currency &rhs) const { return type == rhs.type; }
    bool operator!=(const Currency &rhs) const { return type != rhs.type; }
    bool operator<(const Currency &rhs) const { return type < rhs.type; }

    const QString &AsString() const;
    const QString &AsTag() const;
    const int &AsRank() const;

private:
    static const QString m_currency_type_error;
    static const Currency::TypeStringMap m_currency_type_as_string;
    static const Currency::TypeStringMap m_currency_type_as_tag;
    static const Currency::TypeIntMap m_currency_type_as_rank;
    static const Currency::StringTypeMap m_string_to_currency_type;
};

using CurrencyType = Currency::CurrencyType;
template<>
struct fmt::formatter<CurrencyType, char> : QtEnumFormatter<CurrencyType>
{};
