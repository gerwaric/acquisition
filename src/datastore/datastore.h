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

#include <QString>

#include <vector>

#include "currencymanager.h"
#include "item.h"
#include "itemlocation.h"

class DataStore {
public:
    virtual ~DataStore() {};
    virtual void Set(const QString& key, const QString& value) = 0;
    virtual void SetTabs(const ItemLocationType& type, const Locations& tabs) = 0;
    virtual void SetItems(const ItemLocation& loc, const Items& items) = 0;
    virtual QString Get(const QString& key, const QString& default_value = "") = 0;
    virtual Locations GetTabs(const ItemLocationType& type) = 0;
    virtual Items GetItems(const ItemLocation& loc) = 0;
    virtual void InsertCurrencyUpdate(const CurrencyUpdate& update) = 0;
    virtual std::vector<CurrencyUpdate> GetAllCurrency() = 0;
    void SetInt(const QString& key, int value);
    int GetInt(const QString& key, int default_value = 0);
protected:
    QString Serialize(const Locations& tabs);
    QString Serialize(const Items& items);
    Locations DeserializeTabs(const QString& json);
    Items DeserializeItems(const QString& json, const ItemLocation& tab);
};
