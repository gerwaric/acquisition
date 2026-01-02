// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2025 Tom Holz

#pragma once

#include <QSqlDatabase>
#include <QString>

#include <unordered_map>
#include <vector>

#include <util/spdlog_qt.h>

#include "legacybuyout.h"
// IGNORE: #include "legacycurrency.h"
#include "legacycharacter.h"
#include "legacyitem.h"
#include "legacystash.h"

struct LegacyDataStore
{
    struct DataTable
    {
        QString db_version;
        QString version;
        std::unordered_map<QString, LegacyBuyout> buyouts;
        std::unordered_map<QString, LegacyBuyout> tab_buyouts;
        // TODO: IGNORE: std::unordered_set<QString> refresh_checked_state;
        // TODO: IGNORE: QString currency_last_value;
        // TODO: IGNORE: LegacyCurrencyMap currency_items;
    };

    struct TabsTable
    {
        std::vector<LegacyStash> stashes;
        std::vector<LegacyCharacter> characters;
    };

    using ItemsTable = std::unordered_map<QString, std::vector<LegacyItem>>;
    // IGNORE: using CurrencyTable = std::unordered_map<unsigned long long, QString>;

    LegacyDataStore(const QString &filename);
    bool exportJson(const QString &filename) const;
    bool exportTgz(const QString &filename) const;

    bool isValid() const { return m_valid; };
    qint64 itemCount() const { return m_item_count; };
    const LegacyDataStore::DataTable &data() const { return m_data; };
    const LegacyDataStore::TabsTable &tabs() const { return m_tabs; };
    const LegacyDataStore::ItemsTable &items() const { return m_items; };

private:
    LegacyDataStore::DataTable m_data;
    LegacyDataStore::TabsTable m_tabs;
    LegacyDataStore::ItemsTable m_items;
    // TODO: IGNORE: LegacyDataStore::CurrencyTable currency;

    bool m_valid{false};
    qint64 m_item_count{0};

    // Grant access to Glaze’s reflection for this type
    friend struct glz::meta<LegacyDataStore>;
};

template<>
struct glz::meta<LegacyDataStore>
{
    using T = LegacyDataStore;
    static constexpr auto value
        = glz::object("data", &T::m_data, "tabs", &T::m_tabs, "items", &T::m_items);
};
