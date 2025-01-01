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

#include <QRegularExpression>
#include <QVariant>

#include <QsLog/QsLog.h>
#include <rapidjson/document.h>
#include <rapidjson/error/en.h>

#include "datastore/datastore.h"
#include "util/util.h"

#include "application.h"
#include "itemlocation.h"

const std::map<QString, BuyoutType> BuyoutManager::m_string_to_buyout_type = {
    {"~gb/o", BUYOUT_TYPE_BUYOUT},
    {"~b/o", BUYOUT_TYPE_BUYOUT},
    {"~c/o", BUYOUT_TYPE_CURRENT_OFFER},
    {"~price", BUYOUT_TYPE_FIXED},
};

BuyoutManager::BuyoutManager(DataStore& data)
    : m_data(data)
    , m_save_needed(false)
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
    auto const& it = m_buyouts.lower_bound(item.hash());
    if (it != m_buyouts.end() && !(m_buyouts.key_comp()(item.hash(), it->first))) {
        // Entry exists - we don't want to update if buyout is equal to existing
        if (buyout != it->second) {
            m_save_needed = true;
            it->second = buyout;
        };
    } else {
        m_save_needed = true;
        m_buyouts.insert(it, { item.hash(), buyout });
    };
}

Buyout BuyoutManager::Get(const Item& item) const {
    auto const& it = m_buyouts.find(item.hash());
    if (it != m_buyouts.end()) {
        Buyout buyout = it->second;
        if (buyout.type == BUYOUT_TYPE_CURRENT_OFFER) {
            QLOG_WARN() << "BuyoutManager::Get() obsolete 'current offer' buyout detected for" << item.PrettyName() << ":" << buyout.AsText();
        };
        return buyout;
    };
    return Buyout();
}

Buyout BuyoutManager::GetTab(const QString& tab) const {
    auto const& it = m_tab_buyouts.find(tab);
    if (it != m_tab_buyouts.end()) {
        Buyout buyout = it->second;
        if (buyout.type == BUYOUT_TYPE_CURRENT_OFFER) {
            QLOG_WARN() << "BuyoutManager::GetTab() obsolete 'current offer' buyout detected for" << tab << ":" << buyout.AsText();
        };
        return buyout;
    };
    return Buyout();
}

void BuyoutManager::SetTab(const QString& tab, const Buyout& buyout) {
    if (buyout.type == BUYOUT_TYPE_CURRENT_OFFER) {
        QLOG_WARN() << "BuyoutManager::SetTab() obsolete 'current offer' buyout detected for" << tab << ":" << buyout.AsText();
    };
    auto const& it = m_tab_buyouts.lower_bound(tab);
    if (it != m_tab_buyouts.end() && !(m_tab_buyouts.key_comp()(tab, it->first))) {
        // Entry exists - we don't want to update if buyout is equal to existing
        if (buyout != it->second) {
            m_save_needed = true;
            it->second = buyout;
        };
    } else {
        m_save_needed = true;
        m_tab_buyouts.insert(it, { tab, buyout });
    };
}

void BuyoutManager::CompressTabBuyouts() {
    // When tabs are renamed we end up with stale tab buyouts that aren't deleted.
    // This function is to remove buyouts associated with tab names that don't
    // currently exist.
    std::set<QString> tmp;
    for (auto const& loc : m_tabs)
        tmp.insert(loc.GetUniqueHash());

    for (auto it = m_tab_buyouts.begin(), ite = m_tab_buyouts.end(); it != ite;) {
        if (tmp.count(it->first) == 0) {
            m_save_needed = true;
            it = m_tab_buyouts.erase(it);
        } else {
            ++it;
        };
    };
}

void BuyoutManager::CompressItemBuyouts(const Items& items) {
    // When items are moved between tabs or deleted their buyouts entries remain
    // This function looks at buyouts and makes sure there is an associated item
    // that exists
    std::set<QString> tmp;
    for (auto const& item_sp : items) {
        const Item& item = *item_sp;
        tmp.insert(item.hash());
    };

    for (auto it = m_buyouts.cbegin(); it != m_buyouts.cend();) {
        if (tmp.count(it->first) == 0) {
            m_buyouts.erase(it++);
        } else {
            ++it;
        };
    };
}

void BuyoutManager::SetRefreshChecked(const ItemLocation& loc, bool value) {
    m_save_needed = true;
    m_refresh_checked[loc.GetUniqueHash()] = value;
}

bool BuyoutManager::GetRefreshChecked(const ItemLocation& loc) const {
    auto it = m_refresh_checked.find(loc.GetUniqueHash());
    bool refresh_checked = (it != m_refresh_checked.end()) ? it->second : true;
    return (refresh_checked || GetRefreshLocked(loc));
}

bool BuyoutManager::GetRefreshLocked(const ItemLocation& loc) const {
    return m_refresh_locked.count(loc.GetUniqueHash());
}

void BuyoutManager::SetRefreshLocked(const ItemLocation& loc) {
    m_refresh_locked.insert(loc.GetUniqueHash());
}

void BuyoutManager::ClearRefreshLocks() {
    m_refresh_locked.clear();
}

void BuyoutManager::Clear() {
    m_save_needed = true;
    m_buyouts.clear();
    m_tab_buyouts.clear();
    m_refresh_locked.clear();
    m_refresh_checked.clear();
    m_tabs.clear();
}

QString BuyoutManager::Serialize(const std::map<QString, Buyout>& buyouts) {
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

        rapidjson::Value name(bo.first.toStdString().c_str(), alloc);
        doc.AddMember(name, item, alloc);
    };

    return Util::RapidjsonSerialize(doc);
}

void BuyoutManager::Deserialize(const QString& data, std::map<QString, Buyout>* buyouts) {
    buyouts->clear();

    // if data is empty (on first use) we shouldn't make user panic by showing ERROR messages
    if (data.isEmpty())
        return;

    rapidjson::Document doc;
    if (doc.Parse(data.toStdString().c_str()).HasParseError()) {
        QLOG_ERROR() << "Error while parsing buyouts.";
        QLOG_ERROR() << rapidjson::GetParseError_En(doc.GetParseError());
        return;
    };
    if (!doc.IsObject()) {
        return;
    };
    for (auto itr = doc.MemberBegin(); itr != doc.MemberEnd(); ++itr) {
        auto& object = itr->value;
        const QString& name = itr->name.GetString();
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


QString BuyoutManager::Serialize(const std::map<QString, bool>& obj) {
    rapidjson::Document doc;
    doc.SetObject();
    auto& alloc = doc.GetAllocator();

    for (auto& pair : obj) {
        rapidjson::Value key(pair.first.toStdString().c_str(), alloc);
        rapidjson::Value val(pair.second);
        doc.AddMember(key, val, alloc);
    };
    return Util::RapidjsonSerialize(doc);
}

void BuyoutManager::Deserialize(const QString& data, std::map<QString, bool>& obj) {
    // if data is empty (on first use) we shouldn't make user panic by showing ERROR messages
    if (data.isEmpty()) {
        return;
    };

    rapidjson::Document doc;
    if (doc.Parse(data.toStdString().c_str()).HasParseError()) {
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
    if (!m_save_needed) {
        return;
    };
    m_save_needed = false;
    m_data.Set("buyouts", Serialize(m_buyouts));
    m_data.Set("tab_buyouts", Serialize(m_tab_buyouts));
    m_data.Set("refresh_checked_state", Serialize(m_refresh_checked));
}

void BuyoutManager::Load() {
    Deserialize(m_data.Get("buyouts"), &m_buyouts);
    Deserialize(m_data.Get("tab_buyouts"), &m_tab_buyouts);
    Deserialize(m_data.Get("refresh_checked_state"), m_refresh_checked);
}
void BuyoutManager::SetStashTabLocations(const std::vector<ItemLocation>& tabs) {
    m_tabs = tabs;
}

const std::vector<ItemLocation> BuyoutManager::GetStashTabLocations() const {
    return m_tabs;
}

BuyoutType BuyoutManager::StringToBuyoutType(QString bo_str) const {
    auto const& it = m_string_to_buyout_type.find(bo_str);
    if (it != m_string_to_buyout_type.end()) {
        return it->second;
    };
    return BUYOUT_TYPE_INHERIT;
}

Buyout BuyoutManager::StringToBuyout(QString format) {
    // Parse format string and initialize buyout object, if string does not match any known format
    // then the buyout object will not be valid (IsValid will return false).
    QRegularExpression exp("(~\\S+)\\s+(\\d+\\.?\\d*)\\s+(\\w+)");

    Buyout tmp;
    // regex_search allows for stuff before ~ and after currency type.  We only want to honor the formats
    // that POE trade also accept so this may need to change if it's too generous
    QRegularExpressionMatch m = exp.match(format);
    if (m.hasMatch()) {
        tmp.type = StringToBuyoutType(m.captured(1));
        tmp.value = m.captured(2).toDouble();
        tmp.currency = Currency::FromString(m.captured(3));
        tmp.source = BUYOUT_SOURCE_GAME;
        tmp.last_update = QDateTime::currentDateTime();
    };
    return tmp;
}

void BuyoutManager::MigrateItem(const Item& item) {
    QString old_hash = item.old_hash();
    QString hash = item.hash();
    auto it = m_buyouts.find(old_hash);
    auto new_it = m_buyouts.find(hash);
    if (it != m_buyouts.end() && (new_it == m_buyouts.end() || new_it->second.source != BUYOUT_SOURCE_MANUAL)) {
        m_buyouts[hash] = it->second;
        m_buyouts.erase(it);
        m_save_needed = true;
    };
}
