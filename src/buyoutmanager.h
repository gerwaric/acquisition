// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

#pragma once

#include "item.h"

#include <QDateTime>
#include <QObject>
#include <QString>

#include <set>
#include <unordered_map>

#include "buyout.h"

class Item;
class ItemLocation;
class DataStore;
class BuyoutRepo;

class BuyoutManager : public QObject
{
    Q_OBJECT
public:
    explicit BuyoutManager(DataStore &data, BuyoutRepo &repo);
    ~BuyoutManager();

    void Set(const Item &item, const Buyout &buyout);
    void SetTab(const ItemLocation &location, const Buyout &buyout);

    Buyout Get(const Item &item) const;
    Buyout GetTab(const ItemLocation &location) const;

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
    void ImportBuyouts(const QString &filename);

signals:
    bool SetItemBuyout(const Buyout &buyout, const Item &item);
    bool SetLocationBuyout(const Buyout &buyout, const ItemLocation &location);

private:
    BuyoutType StringToBuyoutType(QString bo_str) const;

    QString Serialize(const std::unordered_map<QString, Buyout> &buyouts);
    void Deserialize(const QString &data, std::unordered_map<QString, Buyout> &buyouts);

    QString Serialize(const std::unordered_map<QString, bool> &obj);
    void Deserialize(const QString &data, std::unordered_map<QString, bool> &obj);

    DataStore &m_data;
    BuyoutRepo &m_repo;

    std::unordered_map<QString, Buyout> m_buyouts;
    std::unordered_map<QString, Buyout> m_tab_buyouts;
    std::unordered_map<QString, bool> m_refresh_checked;
    std::set<QString> m_refresh_locked;
    bool m_save_needed;
    std::vector<ItemLocation> m_tabs;
    static const std::unordered_map<QString, BuyoutType> m_string_to_buyout_type;
};
