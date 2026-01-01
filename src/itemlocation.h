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

#include "util/spdlog_qt.h"

static_assert(ACQUISITION_USE_SPDLOG);

struct LegacyCharacter;
struct LegacyItemLocation;
struct LegacyStash;

namespace poe {
    struct Character;
    struct Item;
    struct StashTab;
} // namespace poe

class ItemLocation
{
    Q_GADGET
public:
    enum class ItemLocationType : int { STASH, CHARACTER };
    Q_ENUM(ItemLocationType)

    ItemLocation();
    ItemLocation(const poe::Character &character, int tab_id);
    ItemLocation(const poe::StashTab &stash);
    ItemLocation(const LegacyCharacter &character, int tab_id);
    ItemLocation(const LegacyStash &stash);

    void FromItem(const poe::Item &item);
    void FromLegacyItemLocation(const LegacyItemLocation &item);
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

    int m_x{0}, m_y{0}, m_w{0}, m_h{0};
    int m_red{0}, m_green{0}, m_blue{0};
    bool m_socketed{false};
    bool m_removeonly{false};
    ItemLocationType m_type;
    int m_tab_id{0};
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
