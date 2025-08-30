/*
    Copyright (C) 2014-2025 Acquisition Contributors

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

#include <util/rapidjson_util.h>
#include <util/spdlog_qt.h>
#include <util/util.h>

#include "itemconstants.h"

ItemLocation::ItemLocation()
    : m_x(0)
    , m_y(0)
    , m_w(0)
    , m_h(0)
    , m_red(0)
    , m_green(0)
    , m_blue(0)
    , m_socketed(false)
    , m_removeonly(false)
    , m_type(ItemLocationType::STASH)
    , m_tab_type("")
    , m_tab_id(0)
{}

ItemLocation::ItemLocation(const rapidjson::Value &root)
    : ItemLocation()
{
    FromItemJson(root);
    FixUid();
}

ItemLocation::ItemLocation(int tab_id, const QString &tab_unique_id, const QString &name)
    : ItemLocation()
{
    m_tab_label = name;
    m_tab_id = tab_id;
    m_tab_unique_id = tab_unique_id;
}

ItemLocation::ItemLocation(int tab_id,
                           const QString &tab_unique_id,
                           const QString &name,
                           ItemLocationType type,
                           const QString &tab_type,
                           int r,
                           int g,
                           int b,
                           rapidjson::Value &value,
                           rapidjson_allocator &alloc)
    : m_x(0)
    , m_y(0)
    , m_w(0)
    , m_h(0)
    , m_red(r)
    , m_green(g)
    , m_blue(b)
    , m_socketed(false)
    , m_type(type)
    , m_tab_id(tab_id)
    , m_tab_unique_id(tab_unique_id)
{
    switch (m_type) {
    case ItemLocationType::STASH:
        m_tab_type = tab_type;
        m_tab_label = name;
        m_character.clear();
        m_character_sortname.clear();
        m_removeonly = name.endsWith("(Remove-only)");
        break;
    case ItemLocationType::CHARACTER:
        m_tab_type.clear();
        m_tab_label.clear();
        m_character = name;
        m_character_sortname = m_character.toLower();
        m_removeonly = false;
        break;
    }

    FixUid();

    if (m_type == ItemLocationType::STASH) {
        if (!value.HasMember("i")) {
            value.AddMember("i", m_tab_id, alloc);
        }
        if (!value.HasMember("n")) {
            rapidjson::Value name_value;
            name_value.SetString(m_tab_label.toStdString().c_str(), alloc);
            value.AddMember("n", name_value, alloc);
        }
        if (!value.HasMember("colour")) {
            rapidjson::Value color_value;
            color_value.SetObject();
            color_value.AddMember("r", m_red, alloc);
            color_value.AddMember("g", m_green, alloc);
            color_value.AddMember("b", m_blue, alloc);
            value.AddMember("colour", color_value, alloc);
        }
    }

    m_json = Util::RapidjsonSerialize(value);
}

void ItemLocation::FixUid()
{
    // With the legacy API, stash tabs have a 64-digit identifier, but
    // the modern API only ten, and it appears to be the first 10.
    if (m_type == ItemLocationType::STASH) {
        if (m_tab_unique_id.size() > 10) {
            m_tab_unique_id = m_tab_unique_id.first(10);
        }
    }
}

void ItemLocation::FromItemJson(const rapidjson::Value &root)
{
    if (root.HasMember("_type")) {
        m_type = static_cast<ItemLocationType>(root["_type"].GetInt());
        switch (m_type) {
        case ItemLocationType::STASH:
            m_tab_label = root["_tab_label"].GetString();
            m_tab_id = root["_tab"].GetInt();
            break;
        case ItemLocationType::CHARACTER:
            m_character = root["_character"].GetString();
            break;
        }
        m_socketed = false;
        if (root.HasMember("_socketed")) {
            m_socketed = root["_socketed"].GetBool();
        }
        if (root.HasMember("_removeonly")) {
            m_removeonly = root["_removeonly"].GetBool();
        }
        // socketed items have x/y pointing to parent
        if (m_socketed) {
            m_x = root["_x"].GetInt();
            m_y = root["_y"].GetInt();
        }
    }
    if (root.HasMember("x") && root.HasMember("y") && root["x"].IsInt() && root["y"].IsInt()) {
        m_x = root["x"].GetInt();
        m_y = root["y"].GetInt();
    }
    if (root.HasMember("w") && root.HasMember("h") && root["w"].IsInt() && root["h"].IsInt()) {
        m_w = root["w"].GetInt();
        m_h = root["h"].GetInt();
    }
    if (root.HasMember("inventoryId") && root["inventoryId"].IsString()) {
        m_inventory_id = root["inventoryId"].GetString();
    }
}

void ItemLocation::ToItemJson(rapidjson::Value *root_ptr, rapidjson_allocator &alloc)
{
    auto &root = *root_ptr;
    rapidjson::Value string_val(rapidjson::kStringType);
    root.AddMember("_type", static_cast<int>(m_type), alloc);
    switch (m_type) {
    case ItemLocationType::STASH:
        root.AddMember("_tab", m_tab_id, alloc);
        string_val.SetString(m_tab_label.toStdString().c_str(), alloc);
        root.AddMember("_tab_label", string_val, alloc);
        break;
    case ItemLocationType::CHARACTER:
        string_val.SetString(m_character.toStdString().c_str(), alloc);
        root.AddMember("_character", string_val, alloc);
        break;
    }
    if (m_socketed) {
        root.AddMember("_x", m_x, alloc);
        root.AddMember("_y", m_y, alloc);
    }
    root.AddMember("_socketed", m_socketed, alloc);
    root.AddMember("_removeonly", m_removeonly, alloc);
}

QString ItemLocation::GetHeader() const
{
    switch (m_type) {
    case ItemLocationType::STASH:
        return QString("#%1, \"%2\"").arg(m_tab_id + 1).arg(m_tab_label);
    case ItemLocationType::CHARACTER:
        return m_character;
    default:
        return "";
    }
}

QRectF ItemLocation::GetRect() const
{
    QRectF result;
    position itemPos{double(m_x), double(m_y)};

    if ((!m_inventory_id.isEmpty()) && (m_type == ItemLocationType::CHARACTER)) {
        auto &map = POS_MAP();
        if (m_inventory_id == "MainInventory") {
            itemPos.y += map.at(m_inventory_id).y;
        } else if (m_inventory_id == "Flask") {
            itemPos.x += map.at(m_inventory_id).x;
            itemPos.y = map.at(m_inventory_id).y;
        } else if (map.count(m_inventory_id)) {
            itemPos = map.at(m_inventory_id);
        }
    }

    // The number of pixels per slot depends on whether we are looking
    // at a quad stash or not.
    float pixels_per_slot = static_cast<float>(PIXELS_PER_MINIMAP_SLOT);
    if (0 == m_tab_type.compare("QuadStash")) {
        pixels_per_slot /= 2.0;
    }

    result.setX(pixels_per_slot * itemPos.x);
    result.setY(pixels_per_slot * itemPos.y);
    result.setWidth(pixels_per_slot * m_w);
    result.setHeight(pixels_per_slot * m_h);
    return result;
}

QString ItemLocation::GetForumCode(const QString &realm,
                                   const QString &league,
                                   unsigned int tab_index) const
{
    switch (m_type) {
    case ItemLocationType::STASH:
        return QString(
                   "[linkItem location=\"Stash%1\" league=\"%2\" x=\"%3\" y=\"%4\" realm=\"%5\"]")
            .arg(QString::number(tab_index + 1),
                 league,
                 QString::number(m_x),
                 QString::number(m_y),
                 realm);
    case ItemLocationType::CHARACTER:
        return QString("[linkItem location=\"%1\" character=\"%2\" x=\"%3\" y=\"%4\" realm=\"%5\"]")
            .arg(m_inventory_id, m_character, QString::number(m_x), QString::number(m_y), realm);
    default:
        return "";
    }
}

bool ItemLocation::IsValid() const
{
    switch (m_type) {
    case ItemLocationType::STASH:
        return !m_tab_unique_id.isEmpty();
    case ItemLocationType::CHARACTER:
        return !m_character.isEmpty();
    default:
        return false;
    }
}

QString ItemLocation::GetUniqueHash() const
{
    if (!IsValid()) {
        spdlog::error("ItemLocation is invalid: {}", m_json);
    };
    switch (m_type) {
    case ItemLocationType::STASH:
        return "stash:" + m_tab_label; // TODO: tab labels are not guaranteed unique
    case ItemLocationType::CHARACTER:
        return "character:" + m_character;
    default:
        return "";
    }
}

bool ItemLocation::operator<(const ItemLocation &rhs) const
{
    if (m_type == rhs.m_type) {
        switch (m_type) {
        case ItemLocationType::STASH:
            return m_tab_id < rhs.m_tab_id;
        case ItemLocationType::CHARACTER:
            return (QString::localeAwareCompare(m_character_sortname, rhs.m_character_sortname) < 0);
        default:
            spdlog::error("Invalid location type: {}", m_type);
            return true;
        }
    } else {
        // STASH locations will always be less than CHARACTER locations.
        return (m_type == ItemLocationType::STASH);
    }
}

bool ItemLocation::operator==(const ItemLocation &other) const
{
    return m_tab_unique_id == other.m_tab_unique_id;
}
