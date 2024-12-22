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

#include "buyoutmanager.h"

#include <regex>

#include <QsLog/QsLog.h>
#include <rapidjson/document.h>
#include <rapidjson/error/en.h>

#include "application.h"
#include "datastore.h"
#include "util.h"
#include "itemlocation.h"
#include "QVariant"

const std::map<std::string, BuyoutType> BuyoutManager::string_to_buyout_type_ = {
    {"~gb/o", BUYOUT_TYPE_BUYOUT},
    {"~b/o", BUYOUT_TYPE_BUYOUT},
    {"~c/o", BUYOUT_TYPE_CURRENT_OFFER},
    {"~price", BUYOUT_TYPE_FIXED},
};

BuyoutManager::BuyoutManager(DataStore& data)
    : data_(data)
    , save_needed_(false)
{
    Load();
}

BuyoutManager::~BuyoutManager() {
    Save();
}

void BuyoutManager::Set(const Item& item, const Buyout& buyout) {
    if (buyout.type == BUYOUT_TYPE_CURRENT_OFFER) {
        QLOG_WARN() << "BuyoutManager::Set() obsolete 'current offer' buyout detected for" << item.PrettyName() << ":" << buyout.AsText();
    };
    auto const& it = buyouts_.lower_bound(item.hash());
    if (it != buyouts_.end() && !(buyouts_.key_comp()(item.hash(), it->first))) {
        // Entry exists - we don't want to update if buyout is equal to existing
        if (buyout != it->second) {
            save_needed_ = true;
            it->second = buyout;
        };
    } else {
        save_needed_ = true;
        buyouts_.insert(it, { item.hash(), buyout });
    };
}

Buyout BuyoutManager::Get(const Item& item) const {
    auto const& it = buyouts_.find(item.hash());
    if (it != buyouts_.end()) {
        Buyout buyout = it->second;
        if (buyout.type == BUYOUT_TYPE_CURRENT_OFFER) {
            QLOG_WARN() << "BuyoutManager::Get() obsolete 'current offer' buyout detected for" << item.PrettyName() << ":" << buyout.AsText();
        };
        return buyout;
    };
    return Buyout();
}

Buyout BuyoutManager::GetTab(const std::string& tab) const {
    auto const& it = tab_buyouts_.find(tab);
    if (it != tab_buyouts_.end()) {
        Buyout buyout = it->second;
        if (buyout.type == BUYOUT_TYPE_CURRENT_OFFER) {
            QLOG_WARN() << "BuyoutManager::GetTab() obsolete 'current offer' buyout detected for" << tab << ":" << buyout.AsText();
        };
        return buyout;
    };
    return Buyout();
}

void BuyoutManager::SetTab(const std::string& tab, const Buyout& buyout) {
    if (buyout.type == BUYOUT_TYPE_CURRENT_OFFER) {
        QLOG_WARN() << "BuyoutManager::SetTab() obsolete 'current offer' buyout detected for" << tab << ":" << buyout.AsText();
    };
    auto const& it = tab_buyouts_.lower_bound(tab);
    if (it != tab_buyouts_.end() && !(tab_buyouts_.key_comp()(tab, it->first))) {
        // Entry exists - we don't want to update if buyout is equal to existing
        if (buyout != it->second) {
            save_needed_ = true;
            it->second = buyout;
        };
    } else {
        save_needed_ = true;
        tab_buyouts_.insert(it, { tab, buyout });
    };
}

void BuyoutManager::CompressTabBuyouts() {
    // When tabs are renamed we end up with stale tab buyouts that aren't deleted.
    // This function is to remove buyouts associated with tab names that don't
    // currently exist.
    std::set<std::string> tmp;
    for (auto const& loc : tabs_)
        tmp.insert(loc.GetUniqueHash());

    for (auto it = tab_buyouts_.begin(), ite = tab_buyouts_.end(); it != ite;) {
        if (tmp.count(it->first) == 0) {
            save_needed_ = true;
            it = tab_buyouts_.erase(it);
        } else {
            ++it;
        };
    };
}

void BuyoutManager::CompressItemBuyouts(const Items& items) {
    // When items are moved between tabs or deleted their buyouts entries remain
    // This function looks at buyouts and makes sure there is an associated item
    // that exists
    std::set<std::string> tmp;
    for (auto const& item_sp : items) {
        const Item& item = *item_sp;
        tmp.insert(item.hash());
    };

    for (auto it = buyouts_.cbegin(); it != buyouts_.cend();) {
        if (tmp.count(it->first) == 0) {
            buyouts_.erase(it++);
        } else {
            ++it;
        };
    };
}

void BuyoutManager::SetRefreshChecked(const ItemLocation& loc, bool value) {
    save_needed_ = true;
    refresh_checked_[loc.GetUniqueHash()] = value;
}

bool BuyoutManager::GetRefreshChecked(const ItemLocation& loc) const {
    auto it = refresh_checked_.find(loc.GetUniqueHash());
    bool refresh_checked = (it != refresh_checked_.end()) ? it->second : true;
    return (refresh_checked || GetRefreshLocked(loc));
}

bool BuyoutManager::GetRefreshLocked(const ItemLocation& loc) const {
    return refresh_locked_.count(loc.GetUniqueHash());
}

void BuyoutManager::SetRefreshLocked(const ItemLocation& loc) {
    refresh_locked_.insert(loc.GetUniqueHash());
}

void BuyoutManager::ClearRefreshLocks() {
    refresh_locked_.clear();
}

void BuyoutManager::Clear() {
    save_needed_ = true;
    buyouts_.clear();
    tab_buyouts_.clear();
    refresh_locked_.clear();
    refresh_checked_.clear();
    tabs_.clear();
}

std::string BuyoutManager::Serialize(const std::map<std::string, Buyout>& buyouts) {
    rapidjson::Document doc;
    doc.SetObject();
    auto& alloc = doc.GetAllocator();

    for (auto& bo : buyouts) {
        const Buyout& buyout = bo.second;
        if (!buyout.IsSavable()) {
            continue;
        };
        rapidjson::Value item(rapidjson::kObjectType);
        item.AddMember("value", buyout.value, alloc);

        // If last_update is null, set as the actual time
        const auto last_update = buyout.last_update.isNull()
            ? QDateTime::currentDateTime().toSecsSinceEpoch()
            : buyout.last_update.toSecsSinceEpoch();
        rapidjson::Value value(rapidjson::kNumberType);
        value.SetInt64(last_update);
        item.AddMember("last_update", value, alloc);

        Util::RapidjsonAddConstString(&item, "type", buyout.BuyoutTypeAsTag(), alloc);
        Util::RapidjsonAddConstString(&item, "currency", buyout.CurrencyAsTag(), alloc);
        Util::RapidjsonAddConstString(&item, "source", buyout.BuyoutSourceAsTag(), alloc);

        item.AddMember("inherited", buyout.inherited, alloc);

        rapidjson::Value name(bo.first.c_str(), alloc);
        doc.AddMember(name, item, alloc);
    };

    return Util::RapidjsonSerialize(doc);
}

void BuyoutManager::Deserialize(const std::string& data, std::map<std::string, Buyout>* buyouts) {
    buyouts->clear();

    // if data is empty (on first use) we shouldn't make user panic by showing ERROR messages
    if (data.empty())
        return;

    rapidjson::Document doc;
    if (doc.Parse(data.c_str()).HasParseError()) {
        QLOG_ERROR() << "Error while parsing buyouts.";
        QLOG_ERROR() << rapidjson::GetParseError_En(doc.GetParseError());
        return;
    };
    if (!doc.IsObject()) {
        return;
    };
    for (auto itr = doc.MemberBegin(); itr != doc.MemberEnd(); ++itr) {
        auto& object = itr->value;
        const std::string& name = itr->name.GetString();
        Buyout bo;

        bo.currency = Currency::FromTag(object["currency"].GetString());
        bo.type = Buyout::TagAsBuyoutType(object["type"].GetString());
        bo.value = object["value"].GetDouble();
        if (object.HasMember("last_update")) {
            bo.last_update = QDateTime::fromSecsSinceEpoch(object["last_update"].GetInt64());
        };
        if (object.HasMember("source")) {
            bo.source = Buyout::TagAsBuyoutSource(object["source"].GetString());
        };
        bo.inherited = false;
        if (object.HasMember("inherited")) {
            bo.inherited = object["inherited"].GetBool();
        };
        if (bo.type == BUYOUT_TYPE_CURRENT_OFFER) {
            QLOG_WARN() << "BuyoutManager::Deserialize() obsolete 'current offer' buyout detected:" << name;
        };
        (*buyouts)[name] = bo;
    };
}


std::string BuyoutManager::Serialize(const std::map<std::string, bool>& obj) {
    rapidjson::Document doc;
    doc.SetObject();
    auto& alloc = doc.GetAllocator();

    for (auto& pair : obj) {
        rapidjson::Value key(pair.first.c_str(), alloc);
        rapidjson::Value val(pair.second);
        doc.AddMember(key, val, alloc);
    };
    return Util::RapidjsonSerialize(doc);
}

void BuyoutManager::Deserialize(const std::string& data, std::map<std::string, bool>& obj) {
    // if data is empty (on first use) we shouldn't make user panic by showing ERROR messages
    if (data.empty()) {
        return;
    };

    rapidjson::Document doc;
    if (doc.Parse(data.c_str()).HasParseError()) {
        QLOG_ERROR() << rapidjson::GetParseError_En(doc.GetParseError());
        return;
    };

    if (!doc.IsObject()) {
        return;
    };

    for (auto itr = doc.MemberBegin(); itr != doc.MemberEnd(); ++itr) {
        const auto& val = itr->value.GetBool();
        const auto& name = itr->name.GetString();
        obj[name] = val;
    };
}

void BuyoutManager::Save() {
    if (!save_needed_) {
        return;
    };
    save_needed_ = false;
    data_.Set("buyouts", Serialize(buyouts_));
    data_.Set("tab_buyouts", Serialize(tab_buyouts_));
    data_.Set("refresh_checked_state", Serialize(refresh_checked_));
}

void BuyoutManager::Load() {
    Deserialize(data_.Get("buyouts"), &buyouts_);
    Deserialize(data_.Get("tab_buyouts"), &tab_buyouts_);
    Deserialize(data_.Get("refresh_checked_state"), refresh_checked_);
}
void BuyoutManager::SetStashTabLocations(const std::vector<ItemLocation>& tabs) {
    tabs_ = tabs;
}

const std::vector<ItemLocation> BuyoutManager::GetStashTabLocations() const {
    return tabs_;
}

BuyoutType BuyoutManager::StringToBuyoutType(std::string bo_str) const {
    auto const& it = string_to_buyout_type_.find(bo_str);
    if (it != string_to_buyout_type_.end()) {
        return it->second;
    };
    return BUYOUT_TYPE_INHERIT;
}

Buyout BuyoutManager::StringToBuyout(std::string format) {
    // Parse format string and initialize buyout object, if string does not match any known format
    // then the buyout object will not be valid (IsValid will return false).
    std::regex exp("(~\\S+)\\s+(\\d+\\.?\\d*)\\s+(\\w+)");

    std::smatch sm;

    Buyout tmp;
    // regex_search allows for stuff before ~ and after currency type.  We only want to honor the formats
    // that POE trade also accept so this may need to change if it's too generous
    if (std::regex_search(format, sm, exp)) {
        tmp.type = StringToBuyoutType(sm[1]);
        tmp.value = QVariant(sm[2].str().c_str()).toDouble();
        tmp.currency = Currency::FromString(sm[3]);
        tmp.source = BUYOUT_SOURCE_GAME;
        tmp.last_update = QDateTime::currentDateTime();
    };
    return tmp;
}

void BuyoutManager::MigrateItem(const Item& item) {
    std::string old_hash = item.old_hash();
    std::string hash = item.hash();
    auto it = buyouts_.find(old_hash);
    auto new_it = buyouts_.find(hash);
    if (it != buyouts_.end() && (new_it == buyouts_.end() || new_it->second.source != BUYOUT_SOURCE_MANUAL)) {
        buyouts_[hash] = it->second;
        buyouts_.erase(it);
        save_needed_ = true;
    };
}
