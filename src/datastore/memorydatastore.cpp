// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2015 Ilya Zhuravlev

#include "datastore/memorydatastore.h"

#include "currencymanager.h"

QString MemoryDataStore::Get(const QString &key, const QString &default_value)
{
    auto i = m_data.find(key);
    if (i == m_data.end()) {
        return default_value;
    }
    return i->second;
}

Locations MemoryDataStore::GetTabs(const ItemLocationType type)
{
    auto i = m_tabs.find(type);
    if (i == m_tabs.end()) {
        return {};
    }
    return i->second;
}

Items MemoryDataStore::GetItems(const ItemLocation &loc)
{
    auto i = m_items.find(loc.get_tab_uniq_id());
    if (i == m_items.end()) {
        return {};
    }
    return i->second;
}

void MemoryDataStore::Set(const QString &key, const QString &value)
{
    m_data[key] = value;
}

void MemoryDataStore::SetTabs(const ItemLocationType type, const Locations &tabs)
{
    m_tabs[type] = tabs;
}

void MemoryDataStore::SetItems(const ItemLocation &loc, const Items &items)
{
    m_items[loc.get_tab_uniq_id()] = items;
}

void MemoryDataStore::InsertCurrencyUpdate(const CurrencyUpdate &update)
{
    m_currency_updates.push_back(update);
}

std::vector<CurrencyUpdate> MemoryDataStore::GetAllCurrency()
{
    return m_currency_updates;
}
