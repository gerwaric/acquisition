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

#include "item.h"
#include <QDateTime>
#include <set>

#include "buyout.h"

class ItemLocation;
class DataStore;

class BuyoutManager {
public:
    explicit BuyoutManager(DataStore& data);
    ~BuyoutManager();
    void Set(const Item& item, const Buyout& buyout);
    Buyout Get(const Item& item) const;

    void SetTab(const std::string& tab, const Buyout& buyout);
    Buyout GetTab(const std::string& tab) const;
    void CompressTabBuyouts();
    void CompressItemBuyouts(const Items& items);

    void SetRefreshChecked(const ItemLocation& tab, bool value);
    bool GetRefreshChecked(const ItemLocation& tab) const;

    bool GetRefreshLocked(const ItemLocation& tab) const;
    void SetRefreshLocked(const ItemLocation& tab);
    void ClearRefreshLocks();

    void SetStashTabLocations(const std::vector<ItemLocation>& tabs);
    const std::vector<ItemLocation> GetStashTabLocations() const;
    void Clear();

    Buyout StringToBuyout(std::string);

    void Save();
    void Load();

    void MigrateItem(const Item& item);
private:
    BuyoutType StringToBuyoutType(std::string bo_str) const;

    std::string Serialize(const std::map<std::string, Buyout>& buyouts);
    void Deserialize(const std::string& data, std::map<std::string, Buyout>* buyouts);

    std::string Serialize(const std::map<std::string, bool>& obj);
    void Deserialize(const std::string& data, std::map<std::string, bool>& obj);

    DataStore& m_data;
    std::map<std::string, Buyout> m_buyouts;
    std::map<std::string, Buyout> m_tab_buyouts;
    std::map<std::string, bool> m_refresh_checked;
    std::set<std::string> m_refresh_locked;
    bool m_save_needed;
    std::vector<ItemLocation> m_tabs;
    static const std::map<std::string, BuyoutType> m_string_to_buyout_type;
};

