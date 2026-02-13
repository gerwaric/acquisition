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
    ItemLocation(const poe::Character &character, int tab_index);
    ItemLocation(const LegacyCharacter &character, int tab_index);
    ItemLocation(const QString &realm, const QString &league, const poe::StashTab &stash);
    ItemLocation(const QString &realm, const QString &league, const LegacyStash &stash);

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

    ItemLocationType type() const { return m_type; }
    QString typeAsString() const { return TypeToString(m_type); }
    QString tab_label() const { return m_tab_label; }
    QString character() const { return m_character; }
    bool socketed() const { return m_socketed; }
    bool removeonly() const { return m_removeonly; }
    int tab_index() const { return m_tab_index; }
    int getR() const { return m_red; }
    int getG() const { return m_green; }
    int getB() const { return m_blue; }
    QString id() const { return m_unique_id; }
    QString json() const { return m_json; }
    QString realm() const { return m_realm; }
    QString league() const { return m_league; }

    static QString TypeToString(ItemLocationType type);
    static std::optional<ItemLocationType> TypeFromString(const QString &str);

private:
    void FixUid();

    int m_x{0}, m_y{0}, m_w{0}, m_h{0};
    int m_red{0}, m_green{0}, m_blue{0};
    bool m_socketed{false};
    bool m_removeonly{false};
    ItemLocationType m_type;
    int m_tab_index{0};
    QString m_json;

    //this would be the value "tabs -> id", which seems to be a hashed value generated on their end
    QString m_unique_id;

    // This is the "type" field from GGG, which is different from the ItemLocationType
    // used by Acquisition.
    QString m_tab_type;
    QString m_tab_label;
    QString m_character;
    QString m_character_sortname;
    QString m_inventory_id;
    QString m_realm;
    QString m_league;
};

using ItemLocationType = ItemLocation::ItemLocationType;

template<>
struct fmt::formatter<ItemLocationType, char> : QtEnumFormatter<ItemLocationType>
{};

typedef std::vector<ItemLocation> Locations;
