// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2015 Ilya Zhuravlev

#pragma once

#include <map>

#include "datastore.h"

class MemoryDataStore : public DataStore
{
public:
    void Set(const QString &key, const QString &value);
    void SetTabs(const ItemLocationType type, const Locations &tabs);
    void SetItems(const ItemLocation &loc, const Items &items);
    QString Get(const QString &key, const QString &default_value = "");
    Locations GetTabs(const ItemLocationType type);
    Items GetItems(const ItemLocation &loc);
    void InsertCurrencyUpdate(const CurrencyUpdate &update);
    std::vector<CurrencyUpdate> GetAllCurrency();

private:
    std::map<QString, QString> m_data;
    std::map<ItemLocationType, Locations> m_tabs;
    std::map<QString, Items> m_items;
    std::vector<CurrencyUpdate> m_currency_updates;
};
