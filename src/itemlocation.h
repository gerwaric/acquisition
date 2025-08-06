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

#pragma once

#include <QColor>
#include <QObject>
#include <QRectF>
#include <QString>

#include <rapidjson/document.h>

#include <util/rapidjson_util.h>
#include <util/spdlog_qt.h>

class ItemLocation
{
    Q_GADGET
public:
    enum class ItemLocationType { STASH, CHARACTER };
    Q_ENUM(ItemLocationType)

    ItemLocation();
    explicit ItemLocation(const rapidjson::Value &root);
    explicit ItemLocation(int tab_id, const QString &tab_unique_id, const QString &name);
    explicit ItemLocation(int tab_id,
                          const QString &tab_unique_id,
                          const QString &name,
                          ItemLocationType type,
                          const QString &tab_type,
                          int r,
                          int g,
                          int b,
                          rapidjson::Value &value,
                          rapidjson_allocator &alloc);

    void ToItemJson(rapidjson::Value *root, rapidjson_allocator &alloc);
    void FromItemJson(const rapidjson::Value &root);
    QString GetHeader() const;
    QRectF GetRect() const;
    QString GetForumCode(const QString &realm,
                         const QString &league,
                         unsigned int stash_index) const;
    QString GetUniqueHash() const;
    bool IsValid() const;
    bool operator<(const ItemLocation &other) const;
    bool operator==(const ItemLocation &other) const;
    ItemLocationType get_type() const { return m_type; }
    QString get_tab_label() const { return m_tab_label; }
    QString get_character() const { return m_character; }
    bool socketed() const { return m_socketed; }
    bool removeonly() const { return m_removeonly; }
    void set_socketed(bool socketed) { m_socketed = socketed; }
    int get_tab_id() const { return m_tab_id; }
    int getR() const { return m_red; }
    int getG() const { return m_green; }
    int getB() const { return m_blue; }
    QString get_tab_uniq_id() const
    {
        return m_type == ItemLocationType::STASH ? m_tab_unique_id : m_character;
    }
    QString get_json() const { return m_json; }

private:
    void FixUid();

    int m_x, m_y, m_w, m_h;
    int m_red, m_green, m_blue;
    bool m_socketed;
    bool m_removeonly;
    ItemLocationType m_type;
    int m_tab_id;
    QString m_json;

    //this would be the value "tabs -> id", which seems to be a hashed value generated on their end
    QString m_tab_unique_id;

    // This is the "type" field from GGG, which is different from the ItemLocationType
    // used by Acquisition.
    QString m_tab_type;

    QString m_tab_label;
    QString m_character;
    QString m_inventory_id;

    QString m_character_sortname;
};

using ItemLocationType = ItemLocation::ItemLocationType;

template<>
struct fmt::formatter<ItemLocationType, char> : QtEnumFormatter<ItemLocationType>
{};

typedef std::vector<ItemLocation> Locations;
