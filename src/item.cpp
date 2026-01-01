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

#include <util/util.h>

#include "itemcategories.h"
#include "itemconstants.h"
#include "itemlocation.h"
#include "modlist.h"
#include "poe/types/displaymode.h"
#include "poe/types/item.h"

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

static QString item_unique_properties(const std::optional<std::vector<poe::ItemProperty>> &props)
{
    if (!props) {
        return "";
    }
    QString result;
    for (const auto &prop : *props) {
        result += prop.name + "~";
        for (const auto &value : prop.values) {
            result += std::get<0>(value) + "~";
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

Item::Item(const poe::Item &json, const ItemLocation &loc)
    : m_location(loc)
{
    m_name = fixup_name(json.name);

    if (json.hybrid) {
        const auto &hybrid = json.hybrid.value();
        if (hybrid.isVaalGem.value_or(false)) {
            // Do not use the base type for vaal gems.
            m_typeLine = json.typeLine;
        } else {
            // Use base type for other hybrid items.
            m_typeLine = hybrid.baseTypeName;
        }
    } else {
        m_typeLine = json.typeLine;
    }
    m_typeLine = fixup_name(m_typeLine);
    m_baseType = fixup_name(json.baseType);
    m_identified = json.identified;

    if (json.corrupted) {
        m_corrupted = *json.corrupted;
    }
    if (json.fractured) {
        m_fractured = *json.fractured;
    }
    if (json.split) {
        m_split = *json.split;
    }
    if (json.synthesised) {
        m_synthesized = *json.synthesised;
    }
    if (json.mutated) {
        m_mutated = *json.mutated;
    }

    m_crafted = (json.craftedMods && !json.craftedMods->empty());
    m_enchanted = (json.enchantMods && !json.enchantMods->empty());

    if (json.influences) {
        if (json.influences->shaper.value_or(false)) {
            m_influenceList.push_back(SHAPER);
        }
        if (json.influences->elder.value_or(false)) {
            m_influenceList.push_back(ELDER);
        }
        if (json.influences->crusader.value_or(false)) {
            m_influenceList.push_back(CRUSADER);
        }
        if (json.influences->redeemer.value_or(false)) {
            m_influenceList.push_back(REDEEMER);
        }
        if (json.influences->hunter.value_or(false)) {
            m_influenceList.push_back(HUNTER);
        }
        if (json.influences->warlord.value_or(false)) {
            m_influenceList.push_back(WARLORD);
        }
    }
    if (json.synthesised.value_or(false)) {
        m_influenceList.push_back(SYNTHESISED);
    }
    if (json.fractured.value_or(false)) {
        m_influenceList.push_back(FRACTURED);
    }
    if (json.searing.value_or(false)) {
        m_influenceList.push_back(SEARING_EXARCH);
    }
    if (json.tangled.value_or(false)) {
        m_influenceList.push_back(EATER_OF_WORLDS);
    }

    m_w = json.w;
    m_h = json.h;
    m_frameType = static_cast<int>(json.frameType);
    m_icon = json.icon;

    using mod_set_t = std::pair<const char *, std::optional<std::vector<QString>>>;

    const std::array<mod_set_t, 5> mod_sets{{{"implicitMods", json.implicitMods},
                                             {"enchantMods", json.enchantMods},
                                             {"explicitMods", json.enchantMods},
                                             {"craftedMods", json.craftedMods},
                                             {"fracturedMods", json.fracturedMods}}};

    for (const auto &it : mod_sets) {
        const auto mod_type = it.first;
        const auto mods = it.second;
        m_text_mods[mod_type] = {};
        if (mods) {
            m_text_mods[mod_type].reserve(mods->size());
            for (const auto &mod : *mods) {
                m_text_mods[mod_type].push_back(mod);
            }
        }
    }

    // Other code assumes icon is proper size so force quad=1 to quad=0 here as it's clunky
    // to handle elsewhere
    m_icon.replace("quad=1", "quad=0");

    // quad stashes, currency stashes, etc
    m_icon.replace("scaleIndex=", "scaleIndex=0&");

    CalculateCategories();

    if (json.talismanTier) {
        m_talisman_tier = *json.talismanTier;
    }
    if (json.id) {
        m_uid = *json.id;
    }
    if (json.note) {
        m_note = *json.note;
    }

    if (json.properties) {
        for (const auto &prop : *json.properties) {
            const QString name = prop.name;
            const auto &values = prop.values;

            if (name == "Elemental Damage") {
                m_elemental_damage.reserve(values.size());
                for (const auto &value : values) {
                    m_elemental_damage.emplace_back(std::get<0>(value), std::get<1>(value));
                }
            } else if (values.size() > 0) {
                const auto &firstValue = values[0];
                QString strval = std::get<0>(firstValue);
                if (m_frameType == ItemEnums::FRAME_TYPE_GEM) {
                    if (name == "Level") {
                        // Gems at max level have the text "(Max)" after the level number.
                        // This needs to be removed so the search field can be matched.
                        if (strval.endsWith("(Max)")) {
                            // Remove "(Max)" and the space before it.
                            strval.chop(6);
                        }
                    } else if (name == "Quality") {
                        // Gem quality is stored like "+23%" but we want to store that as "23".
                        if (strval.startsWith("+")) {
                            strval.removeFirst();
                        }
                        if (strval.endsWith("%")) {
                            strval.chop(1);
                        }
                    }
                }
                m_properties[name] = strval;
            }

            ItemProperty property;
            property.name = name;
            property.display_mode = static_cast<int>(
                prop.displayMode.value_or(poe::DisplayMode::InsertedValues));
            for (const auto &value : values) {
                ItemPropertyValue v;
                v.str = std::get<0>(value);
                v.type = std::get<1>(value);
                property.values.push_back(v);
            }
            m_text_properties.push_back(property);
        }
    }

    if (json.requirements) {
        for (const auto &req : *json.requirements) {
            const auto &values = req.values;
            if (values.size() < 1) {
                continue;
            }
            const QString name = req.name;
            const QString value = std::get<0>(values[0]);
            m_requirements[name] = value.toInt();
            ItemPropertyValue v;
            v.str = value;
            v.type = std::get<1>(values[0]);
            m_text_requirements.push_back({name, v});
        }
    }

    if (json.sockets) {
        ItemSocketGroup current_group = {0, 0, 0, 0};
        m_sockets_cnt = static_cast<int>(json.sockets->size());
        int counter = 0;
        int prev_group = -1;
        for (const auto &socket : *json.sockets) {
            char attr = '\0';
            if (socket.attr) {
                attr = (*socket.attr)[0].toLatin1();
            } else if (socket.sColour) {
                attr = (*socket.sColour)[0].toLatin1();
            }

            if (!attr) {
                continue;
            }

            const int group = socket.group;
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
    const auto it = m_properties.find(QStringLiteral("Stack Size"));
    if (it != m_properties.end()) {
        const QString stack_size = it->second;
        if (stack_size.contains("/")) {
            const auto n = stack_size.indexOf("/");
            m_count = stack_size.first(n).toInt();
        }
    }

    m_ilvl = json.ilvl;

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
    const auto phys = m_properties.find(QStringLiteral("Physical Damage"));
    if (phys == m_properties.end()) {
        return 0;
    }
    const auto aps = m_properties.find(QStringLiteral("Attacks per Second"));
    if (aps == m_properties.end()) {
        return 0;
    }
    const double attacks = aps->second.toDouble();
    const QString hit = phys->second;
    return attacks * Util::AverageDamage(hit);
}

double Item::eDPS() const
{
    if (m_elemental_damage.empty()) {
        return 0;
    }
    const auto aps = m_properties.find(QStringLiteral("Attacks per Second"));
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
    const auto chaos = m_properties.find(QStringLiteral("Chaos Damage"));
    if (chaos == m_properties.end()) {
        return 0;
    }

    const auto aps = m_properties.find(QStringLiteral("Attacks per Second"));
    if (aps == m_properties.end()) {
        return 0;
    }
    const double attacks = aps->second.toDouble();
    const QString hit = chaos->second;

    return attacks * Util::AverageDamage(hit);
}

void Item::GenerateMods(const poe::Item &json)
{
    const std::array mod_sets = {json.implicitMods,
                                 json.enchantMods,
                                 json.explicitMods,
                                 json.craftedMods,
                                 json.fracturedMods};

    for (const auto &mod_set : mod_sets) {
        if (mod_set) {
            for (const auto &mod : *mod_set) {
                AddModToTable(mod, m_mod_table);
            }
        }
    }
}

void Item::CalculateHash(const poe::Item &json)
{
    QString unique_new = m_name + "~" + m_typeLine + "~";
    // GGG removed the <<set>> things in patch 3.4.3e but our hashes all include them, oops
    QString unique_old = "<<set:MS>><<set:M>><<set:S>>" + unique_new;

    QString unique_common;

    if (json.explicitMods) {
        for (const auto &mod : *json.explicitMods) {
            unique_common += mod + "~";
        }
    }
    if (json.implicitMods) {
        for (const auto &mod : *json.implicitMods) {
            unique_common += mod + "~";
        }
    }

    unique_common += item_unique_properties(json.properties) + "~";
    unique_common += item_unique_properties(json.additionalProperties) + "~";

    if (json.sockets) {
        for (const auto &socket : *json.sockets) {
            if (!socket.attr) {
                continue;
            }
            const int group = socket.group;
            const QString attr = *socket.attr;
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
    const QString name = PrettyName();
    const QString rhs_name = rhs.PrettyName();
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

    const auto qual = m_properties.find(QStringLiteral("Quality"));
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

    const auto &sockets = text_sockets();
    if (sockets.size() > 0) {
        pob << "\nSockets: ";
        ItemSocket prev = {255, '-'};
        size_t i = 0;
        for (const auto &socket : sockets) {
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

    const auto lvl = m_requirements.find(QStringLiteral("Level"));
    if (lvl != m_requirements.end()) {
        pob << "\nLevelReq: " << lvl->second;
    }

    const auto &implicitMods = m_text_mods.at("implicitMods");
    const auto &enchantMods = m_text_mods.at("enchantMods");
    pob << "\nImplicits: " << (implicitMods.size() + enchantMods.size());
    for (const auto &mod : enchantMods) {
        pob << "\n{crafted}" << mod.toStdString();
    }
    for (const auto &mod : implicitMods) {
        pob << "\n" << mod.toStdString();
    }

    const auto &fracturedMods = m_text_mods.at("fracturedMods");
    if (!fracturedMods.empty()) {
        for (const auto &mod : fracturedMods) {
            pob << "\n{fractured}" << mod.toStdString();
        }
    }

    const auto &explicitMods = m_text_mods.at("explicitMods");
    const auto &craftedMods = m_text_mods.at("craftedMods");
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
