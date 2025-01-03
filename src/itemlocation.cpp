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

#include "itemlocation.h"

#include <QString>

#include <boost/algorithm/string/predicate.hpp>
#include <QsLog/QsLog.h>

#include "util/rapidjson_util.h"
#include "util/util.h"

#include "itemconstants.h"

#include <string>

using boost::algorithm::ends_with;

QDebug& operator<<(QDebug& os, const ItemLocationType& obj) {
    switch (obj) {
    case ItemLocationType::STASH: return os << "STASH";
    case ItemLocationType::CHARACTER: return os << "CHARACTER";
    default: return os << "<Invalid ItemLocationType(" << static_cast<long long int>(obj) << ")>";
    };
}

ItemLocation::ItemLocation()
    : x_(0), y_(0)
    , w_(0), h_(0)
    , red_(0), green_(0), blue_(0)
    , socketed_(false)
    , removeonly_(false)
    , type_(ItemLocationType::STASH)
    , tab_type_("")
    , tab_id_(0)
{}

ItemLocation::ItemLocation(const rapidjson::Value& root)
    : ItemLocation()
{
    FromItemJson(root);
    FixUid();
}

ItemLocation::ItemLocation(
    int tab_id,
    const std::string tab_unique_id,
    const std::string name)
    : ItemLocation()
{
    tab_label_ = name;
    tab_id_ = tab_id;
    tab_unique_id_ = tab_unique_id;
}

ItemLocation::ItemLocation(
    int tab_id,
    const std::string tab_unique_id,
    const std::string name,
    ItemLocationType type,
    const std::string tab_type,
    int r, int g, int b,
    rapidjson::Value& value, rapidjson_allocator& alloc)
    : x_(0), y_(0)
    , w_(0), h_(0)
    , red_(r), green_(g), blue_(b)
    , socketed_(false)
    , type_(type)
    , tab_id_(tab_id)
    , tab_unique_id_(tab_unique_id)
{
    switch (type_) {
    case ItemLocationType::STASH:
        tab_type_ = tab_type;
        tab_label_ = name;
        character_ = "";
        character_sortname_ = "";
        removeonly_ = ends_with(name, "(Remove-only)");
        break;
    case ItemLocationType::CHARACTER:
        tab_type_ = "";
        tab_label_ = "";
        character_ = name;
        character_sortname_ = QString::fromStdString(character_).toLower();
        removeonly_ = false;
        break;
    };

    FixUid();

    if (type_ == ItemLocationType::STASH) {
        if (!value.HasMember("i")) {
            value.AddMember("i", tab_id_, alloc);
        };
        if (!value.HasMember("n")) {
            rapidjson::Value name_value;
            name_value.SetString(tab_label_.c_str(), alloc);
            value.AddMember("n", name_value, alloc);
        };
        if (!value.HasMember("colour")) {
            rapidjson::Value color_value;
            color_value.SetObject();
            color_value.AddMember("r", red_, alloc);
            color_value.AddMember("g", green_, alloc);
            color_value.AddMember("b", blue_, alloc);
            value.AddMember("colour", color_value, alloc);
        };
    };

    json_ = Util::RapidjsonSerialize(value);

}

void ItemLocation::FixUid() {
    // With the legacy API, stash tabs have a 64-digit identifier, but
    // the modern API only ten, and it appears to be the first 10.
    if (type_ == ItemLocationType::STASH) {
        if (tab_unique_id_.size() > 10) {
            tab_unique_id_ = tab_unique_id_.substr(0, 10);
        };
    };
}

void ItemLocation::FromItemJson(const rapidjson::Value& root) {
    if (root.HasMember("_type")) {
        type_ = static_cast<ItemLocationType>(root["_type"].GetInt());
        switch (type_) {
        case ItemLocationType::STASH:
            tab_label_ = root["_tab_label"].GetString();
            tab_id_ = root["_tab"].GetInt();
            break;
        case ItemLocationType::CHARACTER:
            character_ = root["_character"].GetString();
            break;
        };
        socketed_ = false;
        if (root.HasMember("_socketed")) {
            socketed_ = root["_socketed"].GetBool();
        };
        if (root.HasMember("_removeonly")) {
            removeonly_ = root["_removeonly"].GetBool();
        };
        // socketed items have x/y pointing to parent
        if (socketed_) {
            x_ = root["_x"].GetInt();
            y_ = root["_y"].GetInt();
        };
    };
    if (root.HasMember("x") && root.HasMember("y") && root["x"].IsInt() && root["y"].IsInt()) {
        x_ = root["x"].GetInt();
        y_ = root["y"].GetInt();
    };
    if (root.HasMember("w") && root.HasMember("h") && root["w"].IsInt() && root["h"].IsInt()) {
        w_ = root["w"].GetInt();
        h_ = root["h"].GetInt();
    };
    if (root.HasMember("inventoryId") && root["inventoryId"].IsString())
        inventory_id_ = root["inventoryId"].GetString();
}

void ItemLocation::ToItemJson(rapidjson::Value* root_ptr, rapidjson_allocator& alloc) {
    auto& root = *root_ptr;
    rapidjson::Value string_val(rapidjson::kStringType);
    root.AddMember("_type", static_cast<int>(type_), alloc);
    switch (type_) {
    case ItemLocationType::STASH:
        root.AddMember("_tab", tab_id_, alloc);
        string_val.SetString(tab_label_.c_str(), alloc);
        root.AddMember("_tab_label", string_val, alloc);
        break;
    case ItemLocationType::CHARACTER:
        string_val.SetString(character_.c_str(), alloc);
        root.AddMember("_character", string_val, alloc);
        break;
    };
    if (socketed_) {
        root.AddMember("_x", x_, alloc);
        root.AddMember("_y", y_, alloc);
    };
    root.AddMember("_socketed", socketed_, alloc);
    root.AddMember("_removeonly", removeonly_, alloc);
}

std::string ItemLocation::GetHeader() const {
    switch (type_) {
    case ItemLocationType::STASH: return QString("#%1, \"%2\"").arg(tab_id_ + 1).arg(tab_label_.c_str()).toStdString();
    case ItemLocationType::CHARACTER: return character_;
    default: return "";
    };
}

QRectF ItemLocation::GetRect() const {
    QRectF result;
    position itemPos{ double(x_), double(y_) };

    if ((!inventory_id_.empty()) && (type_ == ItemLocationType::CHARACTER)) {
        auto& map = POS_MAP();
        if (inventory_id_ == "MainInventory") {
            itemPos.y += map.at(inventory_id_).y;
        } else if (inventory_id_ == "Flask") {
            itemPos.x += map.at(inventory_id_).x;
            itemPos.y = map.at(inventory_id_).y;
        } else if (map.count(inventory_id_)) {
            itemPos = map.at(inventory_id_);
        };
    };

    // The number of pixels per slot depends on whether we are looking
    // at a quad stash or not.
    float pixels_per_slot = static_cast<float>(PIXELS_PER_MINIMAP_SLOT);
    if (0 == tab_type_.compare("QuadStash")) {
        pixels_per_slot /= 2.0;
    };

    result.setX(pixels_per_slot * itemPos.x);
    result.setY(pixels_per_slot * itemPos.y);
    result.setWidth(pixels_per_slot * w_);
    result.setHeight(pixels_per_slot * h_);
    return result;
}

std::string ItemLocation::GetForumCode(const std::string& realm, const std::string& league, unsigned int tab_index) const {
    switch (type_) {
    case ItemLocationType::STASH:
        return QString("[linkItem location=\"Stash%1\" league=\"%2\" x=\"%3\" y=\"%4\" realm=\"%5\"]")
            .arg(QString::number(tab_index + 1), league.c_str(), QString::number(x_), QString::number(y_), realm.c_str())
            .toStdString();
    case ItemLocationType::CHARACTER:
        return QString("[linkItem location=\"%1\" character=\"%2\" x=\"%3\" y=\"%4\" realm=\"%5\"]")
            .arg(inventory_id_.c_str(), character_.c_str(), QString::number(x_), QString::number(y_), realm.c_str())
            .toStdString();
    default:
        return "";
    };
}

bool ItemLocation::IsValid() const {
    switch (type_) {
    case ItemLocationType::STASH: return !tab_unique_id_.empty();
    case ItemLocationType::CHARACTER: return !character_.empty();
    default: return false;
    };
}

std::string ItemLocation::GetUniqueHash() const {
    if (!IsValid()) {
        QLOG_ERROR() << "ItemLocation is invalid:" << json_.c_str();;
    };
    switch (type_) {
    case ItemLocationType::STASH: return "stash:" + tab_label_; // TODO: tab labels are not guaranteed unique
    case ItemLocationType::CHARACTER: return "character:" + character_;
    default: return "";
    };
}

bool ItemLocation::operator<(const ItemLocation& rhs) const {
    if (type_ == rhs.type_) {
        switch (type_) {
        case ItemLocationType::STASH: return tab_id_ < rhs.tab_id_;
        case ItemLocationType::CHARACTER: return (QString::localeAwareCompare(character_sortname_, rhs.character_sortname_) < 0);
        default:
            QLOG_ERROR() << "Invalid location type:" << type_;
            return true;
        };
    } else {
        // STASH locations will always be less than CHARACTER locations.
        return (type_ == ItemLocationType::STASH);
    };
}

bool ItemLocation::operator==(const ItemLocation& other) const {
    return tab_unique_id_ == other.tab_unique_id_;
}
