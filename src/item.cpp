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

#include "item.h"

#include <QString>

#include <sstream>
#include <utility>

#include <rapidjson/document.h>

#include <util/rapidjson_util.h>
#include <util/util.h>

#include "itemcategories.h"
#include "itemlocation.h"
#include "modlist.h"

using rapidjson::HasArray;
using rapidjson::HasBool;
using rapidjson::HasInt;
using rapidjson::HasObject;
using rapidjson::HasString;
using rapidjson::HasUint;

const std::array<Item::CategoryReplaceMap, Item::k_CategoryLevels> Item::m_replace_map = {
    // Category hierarchy 0 replacement map
    Item::CategoryReplaceMap({{"Divination", "Divination Cards"}, {"QuestItems", "Quest Items"}}),
    // Category hierarchy 1 replacement map
    Item::CategoryReplaceMap({{"BodyArmours", "Body"},
                              {"VaalGems", "Vaal"},
                              {"AtlasMaps", "2.4"},
                              {"act4maps", "2.0"},
                              {"OneHandWeapons", "1Hand"},
                              {"TwoHandWeapons", "2Hand"}}),
    // Category hierarchy 2 replacement map
    Item::CategoryReplaceMap({{"OneHandAxes", "Axes"},
                              {"OneHandMaces", "Maces"},
                              {"OneHandSwords", "Swords"},
                              {"TwoHandAxes", "Axes"},
                              {"TwoHandMaces", "Maces"},
                              {"TwoHandSwords", "Swords"}})};

static QString item_unique_properties(const rapidjson::Value &json, const QString &name)
{
    const std::string name_s = name.toStdString();
    const char *name_c = name_s.c_str();
    if (!json.HasMember(name_c)) {
        return "";
    }
    QString result;
    for (auto &prop : json[name_c]) {
        result += QString(prop["name"].GetString()) + "~";
        for (auto &value : prop["values"]) {
            result += QString(value[0].GetString()) + "~";
        }
    }
    return result;
}

// Fix up names, remove all <<set:X>> modifiers
static QString fixup_name(const QString &name)
{
    const auto k = name.lastIndexOf(">>");
    if (k >= 0) {
        return name.sliced(k + 2);
    } else {
        return name;
    }
}

Item::Item(const QString &name, const ItemLocation &location)
    : m_name(name)
    , m_location(location)
    , m_hash(Util::Md5(name)) // Unique enough for tests
{}

Item::Item(const rapidjson::Value &json, const ItemLocation &loc)
    : m_location(loc)
    , m_json(Util::RapidjsonSerialize(json))
{
    if (HasString(json, "name")) {
        m_name = fixup_name(json["name"].GetString());
    }
    if (HasString(json, "typeLine")) {
        QString name;
        if (HasObject(json, "hybrid")) {
            const auto &hybrid = json["hybrid"];
            if (HasBool(hybrid, "isVaalGem") && hybrid["isVaalGem"].GetBool()) {
                // Do not use the base type for vaal gems.
                name = json["typeLine"].GetString();
            } else if (HasString(hybrid, "baseTypeName")) {
                // Use base type for other hybrid items.
                name = hybrid["baseTypeName"].GetString();
            } else {
                // Otherwise use the item's root type line.
                name = json["typeLine"].GetString();
            }
        } else {
            name = json["typeLine"].GetString();
        }
        m_typeLine = fixup_name(name);
    }
    if (HasString(json, "baseType")) {
        m_baseType = fixup_name(json["baseType"].GetString());
    }
    if (HasBool(json, "identified")) {
        m_identified = json["identified"].GetBool();
    }
    if (HasBool(json, "corrupted")) {
        m_corrupted = json["corrupted"].GetBool();
    }
    if (HasArray(json, "craftedMods") && !json["craftedMods"].Empty()) {
        m_crafted = true;
    }
    if (HasArray(json, "enchantMods") && !json["enchantMods"].Empty()) {
        m_enchanted = true;
    }
    if (HasObject(json, "influences")) {
        const auto &influences = json["influences"];
        if (influences.HasMember("shaper")) {
            m_influenceList.push_back(SHAPER);
        }
        if (influences.HasMember("elder")) {
            m_influenceList.push_back(ELDER);
        }
        if (influences.HasMember("crusader")) {
            m_influenceList.push_back(CRUSADER);
        }
        if (influences.HasMember("redeemer")) {
            m_influenceList.push_back(REDEEMER);
        }
        if (influences.HasMember("hunter")) {
            m_influenceList.push_back(HUNTER);
        }
        if (influences.HasMember("warlord")) {
            m_influenceList.push_back(WARLORD);
        }
    }

    if (HasBool(json, "synthesised") && json["synthesised"].GetBool()) {
        m_influenceList.push_back(SYNTHESISED);
    }
    if (HasBool(json, "fractured") && json["fractured"].GetBool()) {
        m_influenceList.push_back(FRACTURED);
    }
    if (HasBool(json, "searing") && json["searing"].GetBool()) {
        m_influenceList.push_back(SEARING_EXARCH);
    }
    if (HasBool(json, "tangled") && json["tangled"].GetBool()) {
        m_influenceList.push_back(EATER_OF_WORLDS);
    }

    if (HasInt(json, "w")) {
        m_w = json["w"].GetInt();
    }
    if (HasInt(json, "h")) {
        m_h = json["h"].GetInt();
    }

    if (HasInt(json, "frameType")) {
        m_frameType = json["frameType"].GetInt();
    }

    if (HasString(json, "icon")) {
        m_icon = json["icon"].GetString();
    }

    for (auto &mod_type : ITEM_MOD_TYPES) {
        m_text_mods[mod_type] = {};
        if (HasArray(json, mod_type)) {
            auto &mods = m_text_mods[mod_type];
            for (auto &mod : json[mod_type]) {
                if (mod.IsString()) {
                    mods.push_back(mod.GetString());
                }
            }
        }
    }

    // Other code assumes icon is proper size so force quad=1 to quad=0 here as it's clunky
    // to handle elsewhere
    m_icon.replace("quad=1", "quad=0");
    // quad stashes, currency stashes, etc
    m_icon.replace("scaleIndex=", "scaleIndex=0&");

    CalculateCategories();

    if (HasUint(json, "talismanTier")) {
        m_talisman_tier = json["talismanTier"].GetUint();
    }

    if (HasString(json, "id")) {
        m_uid = json["id"].GetString();
    }

    if (HasString(json, "note")) {
        m_note = json["note"].GetString();
    }

    if (HasArray(json, "properties")) {
        for (auto &prop : json["properties"]) {
            if (!HasString(prop, "name") || !HasArray(prop, "values")) {
                continue;
            }

            const QString name = prop["name"].GetString();
            const auto &values = prop["values"];

            if (name == "Elemental Damage") {
                for (auto &value : values) {
                    if (value.IsArray() && value.Size() >= 2) {
                        if (value[0].IsString() && value[1].IsInt()) {
                            m_elemental_damage.emplace_back(value[0].GetString(), value[1].GetInt());
                        }
                    }
                }
            } else if (values.Size() > 0) {
                if (values[0].IsArray() && values[0].Size() > 0 && values[0][0].IsString()) {
                    m_properties[name] = values[0][0].GetString();
                }
            }

            ItemProperty property;
            property.name = name;
            property.display_mode = prop["displayMode"].GetInt();
            for (const auto &value : values) {
                if (value.IsArray() && value.Size() >= 2 && value[0].IsString()
                    && value[1].IsInt()) {
                    ItemPropertyValue v;
                    v.str = value[0].GetString();
                    v.type = value[1].GetInt();
                    property.values.push_back(v);
                }
            }
            m_text_properties.push_back(property);
        }
    }

    if (HasArray(json, "requirements")) {
        for (auto &req : json["requirements"]) {
            if (!req.IsObject()) {
                continue;
            }
            if (!HasString(req, "name") || !HasArray(req, "values")) {
                continue;
            }
            const auto &values = req["values"];
            if (values.Size() < 1) {
                continue;
            }
            if (!values[0].IsArray() || values[0].Size() < 2) {
                continue;
            }
            if (!values[0][0].IsString() || !values[0][1].IsInt()) {
                continue;
            }
            const QString name = req["name"].GetString();
            const QString value = values[0][0].GetString();
            m_requirements[name] = value.toInt();
            ItemPropertyValue v;
            v.str = value;
            v.type = values[0][1].GetInt();
            m_text_requirements.push_back({name, v});
        }
    }

    if (HasArray(json, "sockets")) {
        ItemSocketGroup current_group = {0, 0, 0, 0};
        m_sockets_cnt = json["sockets"].Size();
        int counter = 0, prev_group = -1;
        for (auto &socket : json["sockets"]) {
            if (!socket.IsObject() || !HasInt(socket, "group")) {
                continue;
            }

            char attr = '\0';
            if (HasString(socket, "attr")) {
                attr = socket["attr"].GetString()[0];
            } else if (HasString(socket, "sColour")) {
                attr = socket["sColour"].GetString()[0];
            }

            if (!attr) {
                continue;
            }

            const int group = socket["group"].GetInt();
            ItemSocket current_socket = {static_cast<unsigned char>(group), attr};
            m_text_sockets.push_back(current_socket);
            if (prev_group != current_socket.group) {
                counter = 0;
                m_socket_groups.push_back(current_group);
                current_group = {0, 0, 0, 0};
            }
            prev_group = current_socket.group;
            ++counter;
            m_links_cnt = std::max(m_links_cnt, counter);
            switch (current_socket.attr) {
            case 'S':
                m_sockets.r++;
                current_group.r++;
                break;
            case 'D':
                m_sockets.g++;
                current_group.g++;
                break;
            case 'I':
                m_sockets.b++;
                current_group.b++;
                break;
            case 'G':
                m_sockets.w++;
                current_group.w++;
                break;
            }
        }
        m_socket_groups.push_back(current_group);
    }

    CalculateHash(json);

    m_count = 1;
    auto it = m_properties.find(QStringLiteral("Stack Size"));
    if (it != m_properties.end()) {
        QString stack_size = it->second;
        if (stack_size.contains("/")) {
            const auto n = stack_size.indexOf("/");
            m_count = stack_size.first(n).toInt();
        }
    }

    if (HasInt(json, "ilvl")) {
        m_ilvl = json["ilvl"].GetInt();
    }

    GenerateMods(json);
}

QString Item::PrettyName() const
{
    if (!m_name.isEmpty()) {
        return m_name + " " + m_typeLine;
    } else {
        return m_typeLine;
    }
}

void Item::CalculateCategories()
{
    m_category = GetItemCategory(m_baseType);
    if (m_category.isEmpty() == false) {
        return;
    }
    // If we didn't find a category on the first try, check to see if
    // this might be a transfigured skill gem by looking for the base
    // name and seeing if that's something we can categorize.
    const auto indx = m_baseType.indexOf(" of ");
    if (indx >= 0) {
        const auto altBaseType = m_baseType.first(indx);
        m_category = GetItemCategory(altBaseType);
        if (m_category.isEmpty() == false) {
            return;
        }
    }
}

double Item::DPS() const
{
    return pDPS() + eDPS() + cDPS();
}

double Item::pDPS() const
{
    auto phys = m_properties.find(QStringLiteral("Physical Damage"));
    if (phys == m_properties.end()) {
        return 0;
    }
    auto aps = m_properties.find(QStringLiteral("Attacks per Second"));
    if (aps == m_properties.end()) {
        return 0;
    }
    const double attacks = aps->second.toDouble();
    QString hit = phys->second;
    return attacks * Util::AverageDamage(hit);
}

double Item::eDPS() const
{
    if (m_elemental_damage.empty()) {
        return 0;
    }
    auto aps = m_properties.find(QStringLiteral("Attacks per Second"));
    if (aps == m_properties.end()) {
        return 0;
    }
    double damage = 0;
    for (auto &x : m_elemental_damage) {
        damage += Util::AverageDamage(x.first);
    }
    const double attacks = aps->second.toDouble();
    return attacks * damage;
}

double Item::cDPS() const
{
    auto chaos = m_properties.find(QStringLiteral("Chaos Damage"));
    if (chaos == m_properties.end()) {
        return 0;
    }

    auto aps = m_properties.find(QStringLiteral("Attacks per Second"));
    if (aps == m_properties.end()) {
        return 0;
    }
    double attacks = aps->second.toDouble();
    QString hit = chaos->second;

    return attacks * Util::AverageDamage(hit);
}

void Item::GenerateMods(const rapidjson::Value &json)
{
    for (const auto &type : ITEM_MOD_TYPES) {
        if (HasArray(json, type)) {
            for (const auto &mod : json[type]) {
                if (mod.IsString()) {
                    AddModToTable(mod.GetString(), &m_mod_table);
                }
            }
        }
    }
}

void Item::CalculateHash(const rapidjson::Value &json)
{
    QString unique_new = m_name + "~" + m_typeLine + "~";
    // GGG removed the <<set>> things in patch 3.4.3e but our hashes all include them, oops
    QString unique_old = "<<set:MS>><<set:M>><<set:S>>" + unique_new;

    QString unique_common;

    if (HasArray(json, "explicitMods")) {
        for (auto &mod : json["explicitMods"]) {
            if (mod.IsString()) {
                unique_common += QString(mod.GetString()) + "~";
            }
        }
    }
    if (HasArray(json, "implicitMods")) {
        for (auto &mod : json["implicitMods"]) {
            if (mod.IsString()) {
                unique_common += QString(mod.GetString()) + "~";
            }
        }
    }

    unique_common += item_unique_properties(json, "properties") + "~";
    unique_common += item_unique_properties(json, "additionalProperties") + "~";

    if (HasArray(json, "sockets")) {
        for (auto &socket : json["sockets"]) {
            if (!HasInt(socket, "group") || !HasString(socket, "attr")) {
                continue;
            }
            const int group = socket["group"].GetInt();
            const QString attr = socket["attr"].GetString();
            unique_common += QString::number(group) + "~" + attr + "~";
        }
    }

    unique_common += "~" + m_location.GetUniqueHash();

    unique_old += unique_common;
    unique_new += unique_common;

    m_old_hash = Util::Md5(unique_old);
    m_hash = Util::Md5(unique_new);
}

bool Item::operator<(const Item &rhs) const
{
    QString name = PrettyName();
    QString rhs_name = rhs.PrettyName();
    return std::tie(name, m_uid, m_hash) < std::tie(rhs_name, rhs.m_uid, m_hash);
}

bool Item::Wearable() const
{
    return (m_category == "flasks" || m_category == "amulet" || m_category == "ring"
            || m_category == "belt" || m_category.contains("armour")
            || m_category.contains("weapons") || m_category.contains("jewels"));
}

QString Item::POBformat() const
{
    std::stringstream pob;
    // clang-format off
    switch (m_frameType) {
    case  0: pob << "Rarity: NORMAL"; break;
    case  1: pob << "Rarity: MAGIC"; break;
    case  2: pob << "Rarity: RARE"; break;
    case  3: pob << "Rarity: UNIQUE"; break;
    case  4: // gem
    case  5: // currency
    case  6: // divination card
    case  7: // quest
    case  8: spdlog::error("Cannot build POB format: unsupported frameType: {}", m_frameType); break; // prophecy (legacy)
    case  9: pob << "Rarity: UNIQUE"; break; // foil
    case 10: pob << "Rarity: UNIQUE"; break; // supporter foil
    case 11: spdlog::error("Cannot build POB format: unsupported frameType: {}", m_frameType); break; // necropolis
    default: spdlog::error("Cannot build POB format: unrecognized frameType: {}", m_frameType); break;
    }
    // clang-format on
    pob << "\n" << name().toStdString();
    pob << "\n" << typeLine().toStdString();
    pob << "\nUnique ID: " << m_uid.toStdString();
    pob << "\nItem Level: " << m_ilvl;

    auto qual = m_properties.find(QStringLiteral("Quality"));
    if (qual != m_properties.end()) {
        QString quality = qual->second;
        if (quality.startsWith("+")) {
            quality.slice(1);
        }
        if (quality.endsWith("%")) {
            quality.chop(1);
        }
        pob << "\nQuality: " << quality.toInt();
    }

    auto &sockets = text_sockets();
    if (sockets.size() > 0) {
        pob << "\nSockets: ";
        ItemSocket prev = {255, '-'};
        size_t i = 0;
        for (auto &socket : sockets) {
            bool link = socket.group == prev.group;
            if (i > 0) {
                pob << (link ? "-" : " ");
            }
            switch (socket.attr) {
            case 'S':
                pob << "R";
                break;
            case 'D':
                pob << "G";
                break;
            case 'I':
                pob << "B";
                break;
            case 'G':
                pob << "W";
                break;
            default:
                pob << socket.attr;
                break;
            }
            prev = socket;
            ++i;
        }
    }

    auto lvl = m_requirements.find(QStringLiteral("Level"));
    if (lvl != m_requirements.end()) {
        pob << "\nLevelReq: " << lvl->second;
    }

    auto &mods = text_mods();

    auto &implicitMods = mods.at("implicitMods");
    auto &enchantMods = mods.at("enchantMods");
    pob << "\nImplicits: " << (implicitMods.size() + enchantMods.size());
    for (const auto &mod : enchantMods) {
        pob << "\n{crafted}" << mod.toStdString();
    }
    for (const auto &mod : implicitMods) {
        pob << "\n" << mod.toStdString();
    }

    auto &fracturedMods = mods.at("fracturedMods");
    if (!fracturedMods.empty()) {
        for (const auto &mod : fracturedMods) {
            pob << "\n{fractured}" << mod.toStdString();
        }
    }

    auto &explicitMods = mods.at("explicitMods");
    auto &craftedMods = mods.at("craftedMods");
    if (!explicitMods.empty() || !craftedMods.empty()) {
        for (const auto &mod : explicitMods) {
            pob << "\n" << mod.toStdString();
        }
        for (const auto &mod : craftedMods) {
            pob << "\n{crafted}" << mod.toStdString();
        }
    }

    if (m_corrupted) {
        pob << "\nCorrupted";
    }

    return QString::fromStdString(pob.str());
}
