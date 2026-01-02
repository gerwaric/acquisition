// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

#pragma once

#include <QString>

#include <vector>

#include "currencymanager.h"
#include "item.h"
#include "itemlocation.h"

class DataStore
{
public:
    virtual ~DataStore() {};
    virtual void Set(const QString &key, const QString &value) = 0;
    virtual void SetTabs(const ItemLocationType type, const Locations &tabs) = 0;
    virtual void SetItems(const ItemLocation &loc, const Items &items) = 0;
    virtual QString Get(const QString &key, const QString &default_value = "") = 0;
    virtual Locations GetTabs(const ItemLocationType type) = 0;
    virtual Items GetItems(const ItemLocation &loc) = 0;
    virtual void InsertCurrencyUpdate(const CurrencyUpdate &update) = 0;
    virtual std::vector<CurrencyUpdate> GetAllCurrency() = 0;
    void SetInt(const QString &key, int value);
    int GetInt(const QString &key, int default_value = 0);

protected:
    QString Serialize(const Locations &tabs);
    QString Serialize(const Items &items);
    Locations DeserializeTabs(const QString &json, ItemLocationType type);
    Items DeserializeItems(const QString &json, const ItemLocation &tab);

private:
    Locations DeserializeStashTabs(std::string_view json);
    Locations DeserializeCharacterTabs(std::string_view json);
};
