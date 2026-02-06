// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

#pragma once

#include <QColor>
#include <QObject>
#include <QRectF>
#include <QString>

#include "util/spdlog_qt.h"

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

    ItemLocation getItemLocation(const poe::Item &item) const;
    void AddLegacyItemLocation(const LegacyItemLocation &item);
    QString GetHeader() const;
    QRectF GetRect() const;
    QString GetForumCode(const QString &realm,
                         const QString &league,
                         unsigned int stash_index) const;
    QString GetLegacyHash() const;

    bool IsValid() const;
    bool operator<(const ItemLocation &other) const;
    bool operator==(const ItemLocation &other) const;

    ItemLocationType get_type() const { return m_type; }
    QString get_tab_label() const { return m_tab_label; }
    QString get_character() const { return m_character; }
    bool socketed() const { return m_socketed; }
    bool removeonly() const { return m_removeonly; }
    int get_tab_id() const { return m_tab_id; }
    int getR() const { return m_red; }
    int getG() const { return m_green; }
    int getB() const { return m_blue; }
    QString get_tab_uniq_id() const { return m_unique_id; }
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
    QString m_unique_id;

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
