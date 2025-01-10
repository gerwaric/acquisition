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

#include <QSqlDatabase>

#include <set>

#include <json_struct/json_struct_qt.h>
#include <QsLog/QsLog.h>

#include "legacybuyout.h"
#include "legacycurrency.h"
#include "legacycharacter.h"
#include "legacyitem.h"
#include "legacystash.h"

class LegacyDataStore {

public:

    enum class ValidationStatus {
        Valid,
        Invalid
    };

    struct DataTable {
        QString db_version;
        QString version;
        std::unordered_map<QString, LegacyBuyout> buyouts;
        std::unordered_map<QString, LegacyBuyout> tab_buyouts;
        // IGNORE: std::unordered_set<QString> refresh_checked_state;
        // IGNORE: QString currency_last_value;
        // IGNORE: LegacyCurrencyMap currency_items;
        JS_OBJ(db_version, version, buyouts, tab_buyouts);
    };

    struct TabsTable {
        std::vector<LegacyStash> stashes;
        std::vector<LegacyCharacter> characters;
        JS_OBJ(stashes, characters);
    };

    using ItemsTable = std::unordered_map<QString, std::vector<LegacyItem>>;
    // IGNORE: using CurrencyTable = std::unordered_map<unsigned long long, QString>;

    struct Imported {
        Imported(const QString& filename);
        bool ok{ true };
        LegacyDataStore::DataTable data;
        LegacyDataStore::TabsTable tabs;
        LegacyDataStore::ItemsTable items;
        // IGNORE: LegacyDataStore::CurrencyTable currency;
        JS_OBJ(data, tabs, items);
    };

    LegacyDataStore(const QString& filename);
    LegacyDataStore::ValidationStatus validate();
    void exportJson(const QString& filename);

private:
    void validateTabBuyouts();
    void validateItemBuyouts();

    const Imported m_datastore;

};
