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

#include <QString>

#include <array>
#include <map>
#include <memory>
#include <unordered_map>
#include <vector>

#include <rapidjson/document.h>

#include "itemlocation.h"

extern const std::vector<QString> ITEM_MOD_TYPES;

struct ItemSocketGroup
{
    int r, g, b, w;
};

struct ItemPropertyValue
{
    QString str;
    int type;
};

struct ItemProperty
{
    QString name;
    std::vector<ItemPropertyValue> values;
    int display_mode;
};

struct ItemRequirement
{
    QString name;
    ItemPropertyValue value;
};

struct ItemSocket
{
    unsigned char group;
    char attr;
};

typedef std::vector<QString> ItemMods;
typedef std::unordered_map<QString, double> ModTable;

class Item
{
public:
    typedef const std::unordered_map<QString, QString> CategoryReplaceMap;

    enum INFLUENCE_TYPES {
        NONE,
        SHAPER,
        ELDER,
        CRUSADER,
        REDEEMER,
        HUNTER,
        WARLORD,
        SYNTHESISED,
        FRACTURED,
        SEARING_EXARCH,
        EATER_OF_WORLDS
    };

    explicit Item(const rapidjson::Value &json, const ItemLocation &loc);
    explicit Item(const QString &name, const ItemLocation &location); // used by tests
    QString name() const { return m_name; }
    QString typeLine() const { return m_typeLine; }
    QString PrettyName() const;
    bool identified() const { return m_identified; }
    bool corrupted() const { return m_corrupted; }
    bool crafted() const { return m_crafted; }
    bool enchanted() const { return m_enchanted; }
    bool hasInfluence(INFLUENCE_TYPES type) const
    {
        return std::find(m_influenceList.begin(), m_influenceList.end(), type)
               != m_influenceList.end();
    }
    INFLUENCE_TYPES influenceLeft() const
    {
        return m_influenceList.size() == 0 ? NONE : m_influenceList[0];
    }
    INFLUENCE_TYPES influenceRight() const
    {
        return m_influenceList.size() == 0   ? NONE
               : m_influenceList.size() == 1 ? m_influenceList[0]
                                             : m_influenceList[1];
    }
    bool hasInfluence() const { return !m_influenceList.empty(); }
    int w() const { return m_w; }
    int h() const { return m_h; }
    int frameType() const { return m_frameType; }
    const QString &icon() const { return m_icon; }
    const std::map<QString, QString> &properties() const { return m_properties; }
    const std::vector<ItemProperty> &text_properties() const { return m_text_properties; }
    const std::vector<ItemRequirement> &text_requirements() const { return m_text_requirements; }
    const std::map<QString, ItemMods> &text_mods() const { return m_text_mods; }
    const std::vector<ItemSocket> &text_sockets() const { return m_text_sockets; }
    const QString &hash() const { return m_hash; }
    const QString &old_hash() const { return m_old_hash; }
    const std::vector<std::pair<QString, int>> &elemental_damage() const
    {
        return m_elemental_damage;
    }
    const std::map<QString, int> &requirements() const { return m_requirements; }
    double DPS() const;
    double pDPS() const;
    double eDPS() const;
    double cDPS() const;
    int sockets_cnt() const { return m_sockets_cnt; }
    int links_cnt() const { return m_links_cnt; }
    const ItemSocketGroup &sockets() const { return m_sockets; }
    const std::vector<ItemSocketGroup> &socket_groups() const { return m_socket_groups; }
    const ItemLocation &location() const { return m_location; }
    const QString &json() const { return m_json; }
    const QString &note() const { return m_note; }
    const QString &category() const { return m_category; }
    uint talisman_tier() const { return m_talisman_tier; }
    int count() const { return m_count; }
    const ModTable &mod_table() const { return m_mod_table; }
    int ilvl() const { return m_ilvl; }
    bool operator<(const Item &other) const;
    bool Wearable() const;
    QString POBformat() const;
    static const size_t k_CategoryLevels = 3;
    static const std::array<CategoryReplaceMap, k_CategoryLevels> m_replace_map;

private:
    void CalculateCategories();
    // The point of GenerateMods is to create combined (e.g. implicit+explicit) poe.trade-like mod map to be searched by mod filter.
    // For now it only does that for a small chosen subset of mods (think "popular" + "pseudo" sections at poe.trade)
    void GenerateMods(const rapidjson::Value &json);
    void CalculateHash(const rapidjson::Value &json);

    QString m_name;
    ItemLocation m_location;
    QString m_typeLine;
    QString m_baseType;
    QString m_category;
    bool m_identified{true};
    bool m_corrupted{false};
    bool m_crafted{false};
    bool m_enchanted{false};
    std::vector<INFLUENCE_TYPES> m_influenceList;
    int m_w{0}, m_h{0};
    int m_frameType{0};
    QString m_icon;
    std::map<QString, QString> m_properties;
    QString m_old_hash, m_hash;
    // vector of pairs [damage, type]
    std::vector<std::pair<QString, int>> m_elemental_damage;
    int m_sockets_cnt{0}, m_links_cnt{0};
    ItemSocketGroup m_sockets{0, 0, 0, 0};
    std::vector<ItemSocketGroup> m_socket_groups;
    std::map<QString, int> m_requirements;
    QString m_json;
    int m_count{0};
    int m_ilvl{0};
    std::vector<ItemProperty> m_text_properties;
    std::vector<ItemRequirement> m_text_requirements;
    std::map<QString, ItemMods> m_text_mods;
    std::vector<ItemSocket> m_text_sockets;
    QString m_note;
    ModTable m_mod_table;
    QString m_uid;
    uint m_talisman_tier{0};
};

typedef std::vector<std::shared_ptr<Item>> Items;
