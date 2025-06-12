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

#include "datastore.h"

class Application;
struct CurrencyUpdate;

class SqliteDataStore : public DataStore {
public:
    SqliteDataStore(const QString& m_filename);
    ~SqliteDataStore();
    void Set(const QString& key, const QString& value);
    void SetTabs(const ItemLocationType type, const Locations& value);
    void SetItems(const ItemLocation& loc, const Items& value);
    QString Get(const QString& key, const QString& default_value = "");
    Locations GetTabs(const ItemLocationType type);
    Items GetItems(const ItemLocation& loc);
    void InsertCurrencyUpdate(const CurrencyUpdate& update);
    std::vector<CurrencyUpdate> GetAllCurrency();
    static QString MakeFilename(const QString& name, const QString& league);
private:
    void CreateTable(const QString& username, const QString& fields);
    void CleanItemsTable();

    QString m_filename;
};
