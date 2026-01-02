// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

#pragma once

#include "item.h"

#include <QDateTime>
#include <QString>

#include <set>

#include "buyout.h"

class ItemLocation;
class DataStore;

class BuyoutManager
{
public:
    explicit BuyoutManager(DataStore &data);
    ~BuyoutManager();
    void Set(const Item &item, const Buyout &buyout);
    Buyout Get(const Item &item) const;

    void SetTab(const QString &tab, const Buyout &buyout);
    Buyout GetTab(const QString &tab) const;
    void CompressTabBuyouts();
    void CompressItemBuyouts(const Items &items);

    void SetRefreshChecked(const ItemLocation &tab, bool value);
    bool GetRefreshChecked(const ItemLocation &tab) const;

    bool GetRefreshLocked(const ItemLocation &tab) const;
    void SetRefreshLocked(const ItemLocation &tab);
    void ClearRefreshLocks();

    void SetStashTabLocations(const std::vector<ItemLocation> &tabs);
    const std::vector<ItemLocation> &GetStashTabLocations() const;
    void Clear();

    Buyout StringToBuyout(QString);

    void Save();
    void Load();

    void MigrateItem(const QString &old_hash, const QString &new_hash);

private:
    BuyoutType StringToBuyoutType(QString bo_str) const;

    QString Serialize(const std::map<QString, Buyout> &buyouts);
    void Deserialize(const QString &data, std::map<QString, Buyout> &buyouts);

    QString Serialize(const std::map<QString, bool> &obj);
    void Deserialize(const QString &data, std::map<QString, bool> &obj);

    DataStore &m_data;
    std::map<QString, Buyout> m_buyouts;
    std::map<QString, Buyout> m_tab_buyouts;
    std::map<QString, bool> m_refresh_checked;
    std::set<QString> m_refresh_locked;
    bool m_save_needed;
    std::vector<ItemLocation> m_tabs;
    static const std::map<QString, BuyoutType> m_string_to_buyout_type;
};
