// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

#include "itemlocation.h"

#include <QString>

#include <legacy/legacycharacter.h>
#include <legacy/legacyitemlocation.h>
#include <legacy/legacystash.h>
#include <poe/types/character.h>
#include <poe/types/stashtab.h>
#include <util/spdlog_qt.h>
#include <util/util.h>

#include "itemconstants.h"

ItemLocation::ItemLocation()
    : m_type(ItemLocationType::STASH)
{}

ItemLocation::ItemLocation(const poe::Character &character, int tab_id)
    : m_type{ItemLocationType::CHARACTER}
    , m_tab_id{tab_id}
    , m_character{character.name}
    , m_character_sortname{character.name.toLower()}
{}

ItemLocation::ItemLocation(const LegacyCharacter &character, int tab_id)
    : m_type{ItemLocationType::CHARACTER}
    , m_tab_id{tab_id}
    , m_character{character.name}
    , m_character_sortname{character.name.toLower()}
{}

ItemLocation::ItemLocation(const poe::StashTab &stash)
    : m_removeonly{stash.name.endsWith("(Remove-only)")}
    , m_type{ItemLocationType::STASH}
    , m_tab_id{int(stash.index.value_or(0))}
    , m_tab_unique_id{stash.id}
    , m_tab_type{stash.type}
    , m_tab_label{stash.name}
{
    Util::GetTabColor(stash, m_red, m_green, m_blue);
}

ItemLocation::ItemLocation(const LegacyStash &stash)
    : m_type{ItemLocationType::STASH}
    , m_tab_id{stash.index}
    , m_tab_unique_id{stash.id}
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
        if (m_tab_unique_id.size() > 10) {
            m_tab_unique_id = m_tab_unique_id.first(10);
        }
    }
}

void ItemLocation::FromItem(const poe::Item &root)
{
    m_x = root.x.value_or(0);
    m_y = root.y.value_or(0);
    m_w = root.w;
    m_h = root.h;
    m_inventory_id = root.inventoryId.value_or("");
}

void ItemLocation::FromLegacyItemLocation(const LegacyItemLocation &item)
{
    m_type = ItemLocationType{item._type};
    m_socketed = item._socketed;
    m_removeonly = item._removeonly;

    // The x and y set here override the one set above in FromItem.
    // I'm not yet sure if this is correct, but it matches the old FromItemJson.
    if (m_socketed) {
        if (!item._x) {
            spdlog::error("ItemLocation: LegacyItemLocation for socketed item is missing _x");
        }
        if (!item._y) {
            spdlog::error("ItemLocation: LegacyItemLocation for socketed item is missing _x");
        }
        m_x = *item._x;
        m_y = *item._y;
    }

    switch (m_type) {
    case ItemLocationType::STASH:
        if (!item._tab) {
            spdlog::error("ItemLocation: LegacyItemLocation for stash is missing _tab");
        }
        if (!item._tab_label) {
            spdlog::error("ItemLocation: LegacyItemLocation for stash is missing _tab_label");
        }
        m_tab_label = item._tab_label.value_or("<<MISSING_LABEL>>");
        m_tab_id = item._tab.value_or(0);
        break;
    case ItemLocationType::CHARACTER:
        if (!item._character) {
            spdlog::error("ItemLocation: LegacyItemLocation for character is missing _character");
        }
        m_character = item._character.value_or("<<MISSING_NAME>>");
        m_character_sortname = m_character.toLower();
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
        return QString("[linkItem location=\"Stash%1\" league=\"%2\" x=\"%3\" y=\"%4\" "
                       "realm=\"%5\"]")
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
