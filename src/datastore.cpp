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

#include "datastore.h"

#include "QsLog.h"
#include "rapidjson/error/en.h"
#include "rapidjson_util.h"
#include "util.h"

using rapidjson::HasInt;
using rapidjson::HasObject;
using rapidjson::HasString;

void DataStore::SetInt(const std::string& key, int value) {
    Set(key, std::to_string(value));
}

int DataStore::GetInt(const std::string& key, int default_value) {
    return std::stoi(Get(key, std::to_string(default_value)));
}

QString DataStore::Serialize(const Locations& tabs) {
    QStringList json;
    json.reserve(tabs.size());
    for (auto& tab : tabs) {
        json.push_back(QString::fromStdString(tab.get_json()));
    };
    return "[" + json.join(",") + "]";
}

QString DataStore::Serialize(const Items& items) {
    QStringList json;
    json.reserve(items.size());
    for (auto& item : items) {
        json.push_back(QString::fromStdString(item->json()));
    };
    return "[" + json.join(",") + "]";
}

Locations DataStore::DeserializeTabs(const QString& json) {

    if (json.isEmpty()) {
        QLOG_DEBUG() << "No tabs to deserialize.";
        return {};
    };

    rapidjson::Document doc;
    doc.Parse(json.toStdString().c_str());
    if (doc.HasParseError()) {
        QLOG_ERROR() << "Error parsing serialized tabs:" << rapidjson::GetParseError_En(doc.GetParseError());
        QLOG_ERROR() << "The malformed json is" << json;
        return {};
    };
    if (doc.IsArray() == false) {
        QLOG_ERROR() << "Error parsing serialized tabs: the json is not an array.";
        return {};
    };

    // Preallocate the return value.
    Locations tabs;
    tabs.reserve(doc.Size());

    // Keep track of which tabs have been parsed.
    std::set<std::string> tab_id_index_;

    for (auto& tab_json : doc) {

        // Detemine which kind of location this is.
        ItemLocationType type = (tab_json.HasMember("class"))
            ? ItemLocationType::CHARACTER
            : ItemLocationType::STASH;

        // Constructor values to fill in
        size_t index = 0;
        std::string tabUniqueId = "";
        std::string name = "";
        std::string tabType = "";
        int r = 0;
        int g = 0;
        int b = 0;

        switch (type) {
        case ItemLocationType::STASH:

            // Get the unique tab id
            if (!HasString(tab_json, "id")) {
                QLOG_ERROR() << "Malformed tab data missing unique id:" << Util::RapidjsonSerialize(tab_json);
                continue;
            };
            tabUniqueId = tab_json["id"].GetString();

            // Make sure we haven't seen this tab before.
            if (tab_id_index_.count(tabUniqueId)) {
                QLOG_ERROR() << "Duplicate tab found while deserializing tabs:" << tabUniqueId;
                continue;
            };

            // Get the tab name from "n" when using the legacy API and "name" when using the OAuth api.
            if (HasString(tab_json, "n")) {
                name = tab_json["n"].GetString();
            } else if (HasString(tab_json, "name")) {
                name = tab_json["name"].GetString();
            } else {
                QLOG_ERROR() << "Malformed tab data doesn't contain a name:" << Util::RapidjsonSerialize(tab_json);
                continue;
            };

            // Get the optional tab index.
            if (HasInt(tab_json, "i")) {
                index = tab_json["i"].GetInt();
            } else {
                index = tabs.size();
            };

            Util::GetTabColor(tab_json, r, g, b);

            // Get the tab type.
            if (HasString(tab_json, "type")) {
                tabType = tab_json["type"].GetString();
            } else {
                QLOG_DEBUG() << "Stash tab does not have a type:" << name;
                tabType = "";
            };

            break;

        case ItemLocationType::CHARACTER:

            // Get the character name
            if (!HasString(tab_json, "name")) {
                continue;
            };
            name = tab_json["name"].GetString();
            tabUniqueId = name;

            // Make sure this isn't a duplicate.
            if (tab_id_index_.count(name)) {
                QLOG_ERROR() << "Duplicate character found while deserializing tabs:" << tabUniqueId;
                continue;
            };

            // Get the optional tab index.
            if (HasInt(tab_json, "i")) {
                index = tab_json["i"].GetInt();
            } else {
                index = tabs.size();
            };

            break;

        default:

            QLOG_ERROR() << "Invalid item location type:" << type;
            continue;

        };
        ItemLocation loc(static_cast<int>(index), tabUniqueId, name, type, tabType, r, g, b, tab_json, doc.GetAllocator());
        tabs.push_back(loc);
        tab_id_index_.insert(loc.get_tab_uniq_id());
    };
    return tabs;
}

Items DataStore::DeserializeItems(const QString& json, const ItemLocation& tab) {

    // Parsed the serialized json and check for errors.
    rapidjson::Document doc;
    doc.Parse(json.toStdString().c_str());
    if (doc.HasParseError()) {
        QLOG_ERROR() << "Error parsing serialized items:" << rapidjson::GetParseError_En(doc.GetParseError());
        QLOG_ERROR() << "The malformed json is" << json;
        return {};
    };
    if (doc.IsArray() == false) {
        QLOG_ERROR() << "Error parsing serialized items: the json is not an array.";
        return {};
    };

    // Preallocate the return value.
    Items items;
    items.reserve(doc.Size());

    // Iterate over each item in the serialized json.
    for (auto item_json = doc.Begin(); item_json != doc.End(); ++item_json) {
        // Create a new location and make sure location-related information
        // such as x and y are pulled from the item json.
        ItemLocation loc = tab;
        loc.FromItemJson(*item_json);
        items.push_back(std::make_shared<Item>(*item_json, loc));
    };
    return items;
}
