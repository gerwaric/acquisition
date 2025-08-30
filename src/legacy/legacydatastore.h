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

#include <QSqlDatabase>
#include <QString>

#include <unordered_map>
#include <vector>

#include <util/json_struct_qt.h>
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
        // IGNORE: std::unordered_set<QString> refresh_checked_state;
        // IGNORE: QString currency_last_value;
        // IGNORE: LegacyCurrencyMap currency_items;
        JS_OBJ(db_version, version, buyouts, tab_buyouts);
    };

    struct TabsTable
    {
        std::vector<LegacyStash> stashes;
        std::vector<LegacyCharacter> characters;
        JS_OBJ(stashes, characters);
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

    JS_OBJECT(JS_MEMBER_WITH_NAME(m_data, "data"),
              JS_MEMBER_WITH_NAME(m_tabs, "tabs"),
              JS_MEMBER_WITH_NAME(m_items, "items"));

private:
    LegacyDataStore::DataTable m_data;
    LegacyDataStore::TabsTable m_tabs;
    LegacyDataStore::ItemsTable m_items;
    // IGNORE: LegacyDataStore::CurrencyTable currency;

    bool m_valid{false};
    qint64 m_item_count{0};
};
