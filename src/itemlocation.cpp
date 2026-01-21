// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

#include "itemlocation.h"

#include <QString>

#include "itemconstants.h"
#include "legacy/legacycharacter.h"
#include "legacy/legacyitemlocation.h"
#include "legacy/legacystash.h"
#include "poe/types/character.h"
#include "poe/types/stashtab.h"
#include "util/spdlog_qt.h" // IWYU pragma: keep
#include "util/util.h"

ItemLocation::ItemLocation()
    : m_type(ItemLocationType::STASH)
{}

ItemLocation::ItemLocation(const poe::Character &character, int tab_id)
    : m_type{ItemLocationType::CHARACTER}
    , m_tab_id{tab_id}
    , m_unique_id(character.id)
    , m_character{character.name}
    , m_character_sortname{character.name.toLower()}
{}

ItemLocation::ItemLocation(const LegacyCharacter &character, int tab_id)
    : m_type{ItemLocationType::CHARACTER}
    , m_tab_id{tab_id}
    , m_unique_id(character.id)
    , m_character{character.name}
    , m_character_sortname{character.name.toLower()}
{}

ItemLocation::ItemLocation(const poe::StashTab &stash)
    : m_removeonly{stash.name.endsWith("(Remove-only)")}
    , m_type{ItemLocationType::STASH}
    , m_tab_id{int(stash.index.value_or(0))}
    , m_unique_id{stash.id}
    , m_tab_type{stash.type}
    , m_tab_label{stash.name}
{
    Util::GetTabColor(stash, m_red, m_green, m_blue);
}

ItemLocation::ItemLocation(const LegacyStash &stash)
    : m_type{ItemLocationType::STASH}
    , m_tab_id{stash.index}
    , m_unique_id{stash.id}
    , m_tab_type{stash.type}
    , m_tab_label{stash.name}
{
    if (stash.i && (*stash.i != stash.index)) {
        spdlog::error("ItemLocation: LegacyStash is inconsistent: i = {}, index = {}",
                      *stash.i,
                      stash.index);
    }

    if (stash.n && (*stash.n != stash.name)) {
        spdlog::error("ItemLocation: LegacyStash is inconsistent: n = {}, name = {}",
                      *stash.n,
                      stash.name);
    }

    if (stash.colour) {
        m_red = stash.colour->r;
        m_green = stash.colour->g;
        m_blue = stash.colour->b;
    }
}

void ItemLocation::FixUid()
{
    // With the legacy API, stash tabs have a 64-digit identifier, but
    // the modern API only ten, and it appears to be the first 10.
    if (m_type == ItemLocationType::STASH) {
        if (m_unique_id.size() > 10) {
            m_unique_id = m_unique_id.first(10);
        }
    }
}

ItemLocation ItemLocation::getItemLocation(const poe::Item &item) const
{
    ItemLocation item_location = *this;
    if (item.x) {
        item_location.m_x = *item.x;
    }
    if (item.y) {
        item_location.m_y = *item.y;
    }
    item_location.m_w = item.w;
    item_location.m_h = item.h;
    if (item.inventoryId) {
        item_location.m_inventory_id = *item.inventoryId;
    }
    if (item.socket) {
        item_location.m_socketed = true;
    }
    return item_location;
}

void ItemLocation::AddLegacyItemLocation(const LegacyItemLocation &item)
{
    const auto _type = ItemLocationType{item._type};
    if (m_type != _type) {
        spdlog::warn("ItemLocation: legacy item location mismatch: _type");
    }
    m_type = _type;

    if (m_socketed != item._socketed) {
        spdlog::warn("ItemLocation: legacy item location mismatch: _removeonly");
    }
    m_socketed = item._socketed;

    if (m_removeonly != item._removeonly) {
        spdlog::warn("ItemLocation: legacy item location mismatch: _removeonly");
    }
    m_removeonly = item._removeonly;

    // The x and y set here override the one set above in FromItem.
    // I'm not yet sure if this is correct, but it matches the old FromItemJson.
    if (m_socketed) {
        // x-location
        if (!item._x) {
            spdlog::warn("ItemLocation: LegacyItemLocation for socketed item is missing _x");
        } else {
            if (m_x != *item._x) {
                spdlog::warn("ItemLocation: legacy item location mismatch: _x");
            }
            m_x = *item._x;
        }

        // y-location
        if (!item._y) {
            spdlog::warn("ItemLocation: LegacyItemLocation for socketed item is missing _y");
        } else {
            if (m_x != *item._y) {
                spdlog::warn("ItemLocation: legacy item location mismatch: _y");
            }
            m_y = *item._y;
        }
    }

    switch (m_type) {
    case ItemLocationType::STASH:
        // m_tab_id
        if (!item._tab) {
            spdlog::error("ItemLocation: LegacyItemLocation for stash is missing _tab");
        } else {
            if (m_tab_id != *item._tab) {
                spdlog::warn("ItemLocation: legacy item location mismatch: _tab");
            }
            m_tab_id = *item._tab;
        }
        // m_tab_label
        if (!item._tab_label) {
            spdlog::error("ItemLocation: LegacyItemLocation for stash is missing _tab_label");
        } else {
            if (m_tab_label != *item._tab_label) {
                spdlog::warn("ItemLocation: legacy item location mismatch: _tab_label");
            }
            m_tab_label = *item._tab_label;
        }
        break;
    case ItemLocationType::CHARACTER:
        // m_character
        if (!item._character) {
            spdlog::error("ItemLocation: LegacyItemLocation for stash is missing _character");
        } else {
            if (m_character != *item._character) {
                spdlog::warn("ItemLocation: legacy item location mismatch: _character");
            }
            m_character = *item._character;
            m_character_sortname = m_character.toLower();
        }
        break;
    }
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
        return QString(R"([linkItem location="Stash%1" league="%2" x="%3" y="%4" realm="%5"])")
            .arg(QString::number(tab_index + 1),
                 league,
                 QString::number(m_x),
                 QString::number(m_y),
                 realm);
    case ItemLocationType::CHARACTER:
        return QString(R"([linkItem location="%1" character="%2" x="%3" y="%4" realm="%5"])")
            .arg(m_inventory_id, m_character, QString::number(m_x), QString::number(m_y), realm);
    default:
        return "";
    }
}

bool ItemLocation::IsValid() const
{
    return !m_unique_id.isEmpty();
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
    return m_unique_id == other.m_unique_id;
}
