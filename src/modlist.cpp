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

#include "modlist.h"

#include <QRegularExpression>
#include <QStringList>
#include <QtSql/QSqlDatabase>
#include <QtSql/QSqlError>
#include <QtSql/QSqlQuery>
#include <QtSql/QSqlRecord>

#include <memory>
#include <set>
#include <unordered_map>
#include <vector>

#include <util/spdlog_qt.h>
#include <util/util.h>

#include "item.h"

using rapidjson::HasBool;

// These are just summed, and the mod named as the first element of a vector is generated with value equaling the sum.
// Both implicit and explicit fields are considered.
// This is pretty much the same list as poe.trade uses

namespace {

    QStringListModel m_mod_list_model;
    std::set<QString> mods;
    std::unordered_map<QString, SumModGenerator *> mods_map;
    std::vector<SumModGen> mod_generators;

    using ModMap = std::unordered_map<QString, std::vector<QString>>;

    const ModMap summing_pseudo_mods = {
        {"+#% total to Cold Resistance",
         {
             "+#% to Cold Resistance",
             "+#% to Fire and Cold Resistances",
             "+#% to Cold and Lightning Resistances",
             "+#% to Cold and Chaos Resistances",
             "+#% to all Elemental Resistances",
         }},
        {"+#% total to Fire Resistance",
         {
             "+#% to Fire Resistance",
             "+#% to Fire and Cold Resistances",
             "+#% to Fire and Lightning Resistances",
             "+#% to Fire and Chaos Resistances",
             "+#% to all Elemental Resistances",
         }},
        {"+#% total to Lightning Resistance",
         {
             "+#% to Lightning Resistance",
             "+#% to Fire and Lightning Resistances",
             "+#% to Cold and Lightning Resistances",
             "+#% to Lightning and Chaos Resistances",
             "+#% to all Elemental Resistances",
         }},
        {"+#% total Elemental Resistance",
         {
             //"+#% total to Fire Resistance", (eventually we should just use the pseudo mods here)
             "+#% to Fire Resistance",
             "+#% to Fire and Cold Resistances",
             "+#% to Fire and Lightning Resistances",
             "+#% to Fire and Chaos Resistances",
             "+#% to all Elemental Resistances",
             //"+#% total to Cold Resistance",
             "+#% to Cold Resistance",
             "+#% to Fire and Cold Resistances",
             "+#% to Cold and Lightning Resistances",
             "+#% to Cold and Chaos Resistances",
             "+#% to all Elemental Resistances",
             //"+#% total to Lightning Resistance",
             "+#% to Lightning Resistance",
             "+#% to Fire and Lightning Resistances",
             "+#% to Cold and Lightning Resistances",
             "+#% to Lightning and Chaos Resistances",
             "+#% to all Elemental Resistances",
         }},
        {"+#% total to Chaos Resistance",
         {
             "+#% to Chaos Resistance",
             "+#% to Fire and Chaos Resistances",
             "+#% to Cold and Chaos Resistances",
             "+#% to Lightning and Chaos Resistances",
         }},
        {"+#% total Resistance",
         {
             //"+#% total Elemental Resistance", (again, this should be replaced by other pseudo mods ideally)
             // --> "+#% total to Fire Resistance",
             "+#% to Fire Resistance",
             "+#% to Fire and Cold Resistances",
             "+#% to Fire and Lightning Resistances",
             "+#% to Fire and Chaos Resistances",
             "+#% to all Elemental Resistances",
             // --> "+#% total to Cold Resistance",
             "+#% to Cold Resistance",
             "+#% to Fire and Cold Resistances",
             "+#% to Cold and Lightning Resistances",
             "+#% to Cold and Chaos Resistances",
             "+#% to all Elemental Resistances",
             // --> "+#% total to Lightning Resistance",
             "+#% to Lightning Resistance",
             "+#% to Fire and Lightning Resistances",
             "+#% to Cold and Lightning Resistances",
             "+#% to Lightning and Chaos Resistances",
             "+#% to all Elemental Resistances",
             //"+#% total to Chaos Resistance",
             "+#% to Chaos Resistance",
             "+#% to Fire and Chaos Resistances",
             "+#% to Cold and Chaos Resistances",
             "+#% to Lightning and Chaos Resistances",
         }},
        {"+# total to Strength",
         {
             "+# to Strength",
             "+# to Strength and Dexterity",
             "+# to Strength and Intelligence",
             "+# to all Attributes",
         }},
        {"+# total to Dexterity",
         {
             "+# to Dexterity",
             "+# to Strength and Dexterity",
             "+# to Dexterity and Intelligence",
             "+# to all Attributes",
         }},
        {"+# total to Intelligence",
         {
             "+# to Intelligence",
             "+# to Strength and Intelligence",
             "+# to Dexterity and Intelligence",
             "+# to all Attributes",
         }},
    };

    ModMap reverseModMap(const ModMap &map)
    {
        ModMap reverse_map;

        for (const auto &[pseudo_mod, real_mods] : map) {
            for (const auto &real_mod : real_mods) {
                reverse_map[real_mod].push_back(pseudo_mod);
            }
        }

        return reverse_map;
    }

    const ModMap summing_pseudo_mods_reverse_lookup = reverseModMap(summing_pseudo_mods);

} // namespace

// TBD: counting pseudo-mods like "# total Elemental Resistances"
// TBD: min pseudo-mods like "+#% total to All Elemental Resistances" (i.e. min(Cold, Fire, Lightning)

/*

const std::vector<std::vector<QString>> simple_sum = {
    {"#% increased Quantity of Items found"},
    {"#% increased Rarity of Items found"},
    {"+# to maximum Life"},
    {"#% increased maximum Life"},
    {"+# to maximum Energy Shield"},
    {"#% increased maximum Energy Shield"},
    {"+# to maximum Mana"},
    {"#% increased Mana Regeneration Rate"},
    {"+#% to Total Fire Resistance",
     "+#% to Fire and Cold Resistances",
     "+#% to Fire and Lightning Resistances",
     "+#% to all Elemental Resistances",
     "+#% to Fire and Chaos Resistances"},
    {"+#% to Total Cold Resistance",
     "+#% to Fire and Cold Resistances",
     "+#% to Cold and Lightning Resistances",
     "+#% to all Elemental Resistances",
     "+#% to Cold and Chaos Resistances"},
    {"+#% to Total Lightning Resistance",
     "+#% to Cold and Lightning Resistances",
     "+#% to Fire and Lightning Resistances",
     "+#% to all Elemental Resistances",
     "+#% to Lightning and Chaos Resistances"},
    {"+#% to all Elemental Resistances"},
    {"+#% to Total Chaos Resistance",
     "+#% to Fire and Chaos Resistances",
     "+#% to Cold and Chaos Resistances",
     "+#% to Lightning and Chaos Resistances"},
    {"+# to Level of Socketed Aura Gems", "+# to Level of Socketed Gems"},
    {"+# to Level of Socketed Fire Gems", "+# to Level of Socketed Gems"},
    {"+# to Level of Socketed Cold Gems", "+# to Level of Socketed Gems"},
    {"+# to Level of Socketed Lightning Gems", "+# to Level of Socketed Gems"},
    {"+# to Level of Socketed Chaos Gems", "+# to Level of Socketed Gems"},
    {"+# to Level of Socketed Elemental Gems", "+# to Level of Socketed Gems"},
    {"+# to Level of Socketed Spell Gems", "+# to Level of Socketed Gems"},
    {"+# to Level of Socketed Bow Gems", "+# to Level of Socketed Gems"},
    {"+# to Level of Socketed Minion Gems", "+# to Level of Socketed Gems"},
    {"+# to Level of Socketed Vaal Gems", "+# to Level of Socketed Gems"},
    {"+# to Level of Socketed Melee Gems", "+# to Level of Socketed Gems"},
    {"+# to Level of Socketed Movement Gems", "+# to Level of Socketed Gems"},
    {"+# to Level of Socketed Strength Gems", "+# to Level of Socketed Gems"},
    {"#% increased Physical Damage", "#% increased Global Physical Damage", "#% increased Damage"},
    {"#% increased Spell Damage", "#% increased Damage"},
    {"#% increased Elemental Damage", "#% increased Damage"},
    {"#% increased Fire Damage", "#% increased Elemental Damage", "#% increased Damage"},
    {"#% increased Cold Damage", "#% increased Elemental Damage", "#% increased Damage"},
    {"#% increased Lightning Damage", "#% increased Elemental Damage", "#% increased Damage"},
    {"#% increased Chaos Damage", "#% increased Damage"},
    {"#% increased Fire Damage with Attack Skills",
     "#% increased Fire Damage",
     "#% increased Elemental Damage with Attack Skills",
     "#% increased Elemental Damage",
     "#% increased Damage"},
    {"#% increased Cold Damage with Attack Skills",
     "#% increased Cold Damage",
     "#% increased Elemental Damage with Attack Skills",
     "#% increased Elemental Damage",
     "#% increased Damage"},
    {"#% increased Lightning Damage with Attack Skills",
     "#% increased Lightning Damage",
     "#% increased Elemental Damage with Attack Skills",
     "#% increased Elemental Damage",
     "#% increased Damage"},
    {"#% increased Elemental Damage with Attack Skills",
     "#% increased Elemental Damage",
     "#% increased Damage"},
    {"#% increased Fire Spell Damage",
     "#% increased Fire Damage",
     "#% increased Elemental Damage",
     "#% increased Spell Damage",
     "#% increased Damage"},
    {"#% increased Cold Spell Damage",
     "#% increased Cold Damage",
     "#% increased Elemental Damage",
     "#% increased Spell Damage",
     "#% increased Damage"},
    {"#% increased Lightning Spell Damage",
     "#% increased Lightning Damage",
     "#% increased Elemental Damage",
     "#% increased Spell Damage",
     "#% increased Damage"},
    {"#% increased Global Critical Strike Chance"},
    {"#% increased Critical Strike Chance for Spells",
     "#% increased Global Critical Strike Chance",
     "Spells have +#% to Critical Strike Chance "},
    {"+#% to Global Critical Strike Multiplier"},
    {"#% to Melee Critical Strike Multiplier"},
    {"#% to Critical Strike Multiplier with Elemental Skills",
     "#% to Global Critical Strike Multiplier"},
    {"#% to Critical Strike Multiplier with Fire Skills",
     "#% to Critical Strike Multiplier with Elemental Skills",
     "#% to Global Critical Strike Multiplier"},
    {"#% to Critical Strike Multiplier with Cold Skills",
     "#% to Critical Strike Multiplier with Elemental Skills",
     "#% to Global Critical Strike Multiplier"},
    {"#% to Critical Strike Multiplier with Lightning Skills",
     "#% to Critical Strike Multiplier with Elemental Skills",
     "#% to Global Critical Strike Multiplier"},
    {"#% to Critical Strike Multiplier with One Handed Melee Weapons",
     "#% to Global Critical Strike Multiplier"},
    {"#% to Critical Strike Multiplier while Dual Wielding",
     "#% to Global Critical Strike Multiplier"},
    {"#% to Critical Strike Multiplier with Two Handed Melee Weapons",
     "#% to Global Critical Strike Multiplier"},
    {"#% to Critical Strike Multiplier for Spells", "#% to Global Critical Strike Multiplier"},
    {"#% to Critical Strike Multiplier with Fire Spells",
     "#% to Critical Strike Multiplier for Spells",
     "#% to Critical Strike Multiplier with Fire Skills",
     "#% to Critical Strike Multiplier with Elemental Skills",
     "#% to Global Critical Strike Multiplier"},
    {"#% to Critical Strike Multiplier with Cold Spells",
     "#% to Critical Strike Multiplier for Spells",
     "#% to Critical Strike Multiplier with Fire Skills",
     "#% to Critical Strike Multiplier with Elemental Skills",
     "#% to Global Critical Strike Multiplier"},
    {"#% to Critical Strike Multiplier with Lightning Spells",
     "#% to Critical Strike Multiplier for Spells",
     "#% to Critical Strike Multiplier with Fire Skills",
     "#% to Critical Strike Multiplier with Elemental Skills",
     "#% to Global Critical Strike Multiplier"},
    {"+# to Accuracy Rating"},
    {"#% increased Accuracy Rating"},
    {"#% increased Area Damage", "#% increased Damage"},
    {"#% increased Damage over Time", "#% increased Damage"},
    {"#% increased Burning Damage",
     "#% increased Fire Damage",
     "#% increased Elemental Damage",
     "#% increased Damage over Time",
     "#% increased Damage"},
    {"#% of Physical Attack Damage Leeched as Life"},
    {"#% of Physical Attack Damage Leeched as Mana"},
    {"Adds # Physical Damage to Attacks",
     "Adds # to # Physical Damage",
     "Adds # to # Physical Damage to Attacks"},
    {"Adds # Elemental Damage to Attacks",
     "Adds # to # Fire Damage",
     "Adds # to # Cold Damage",
     "Adds # to # Lightning Damage",
     "Adds # to # Fire Damage to Attacks",
     "Adds # to # Cold Damage to Attacks",
     "Adds # to # Lightning Damage to Attacks"},
    {"Adds # Chaos Damage to Attacks", "Adds # to # Chaos Damage to Attacks"},
    {"Adds # Damage to Attacks",
     "Adds # to # Fire Damage",
     "Adds # to # Cold Damage",
     "Adds # to # Lightning Damage",
     "Adds # to # Physical Damage",
     "Adds # to # Chaos Damage",
     "Adds # to # Fire Damage to Attacks",
     "Adds # to # Cold Damage to Attacks",
     "Adds # to # Lightning Damage to Attacks",
     "Adds # to # Physical Damage to Attacks",
     "Adds # to # Chaos Damage to Attacks"},
    {"Adds # Elemental Damage to Spells",
     "Adds # to # Fire Damage to Spells",
     "Adds # to # Cold Damage to Spells",
     "Adds # to # Lightning Damage to Spells"},
    {"Adds # Chaos Damage to Spells", "Adds # to # Chaos Damage to Spells"},
    {"Adds # Damage to Spells",
     "Adds # to # Fire Damage to Spells",
     "Adds # to # Cold Damage to Spells",
     "Adds # to # Lightning Damage to Spells",
     "Adds # to # Chaos Damage to Spells"},
    {"#% increased Attack Speed", "#% increased Attack and Cast Speed"},
    {"#% increased Cast Speed", "#% increased Attack and Cast Speed"},
    {"#% increased Movement Speed"},
    {"+# to Dexterity",
     "+# to Strength and Dexterity",
     "+# to Dexterity and Intelligence",
     "+# to all Attributes"},
    {"+# to Strength",
     "+# to Strength and Dexterity",
     "+# to Strength and Intelligence",
     "+# to all Attributes"},
    {"+# to Intelligence",
     "+# to Dexterity and Intelligence",
     "+# to Strength and Intelligence",
     "+# to all Attributes"},
    {"+# to all Attributes"},
    {"#% increased Stun Duration on enemies"},
    {"#% increased Block and Stun Recovery"},
    // Master mods
    {"+#% to Quality of Socketed Support Gems"},
    {"-# to Total Mana Cost of Skills"},
    {"+# to Level of Socketed Support Gems", "+# to Level of Socketed Gems"},
    {"Causes Bleeding on Hit"},
    {"#% increased Life Leeched per second"},
    {"Hits can't be Evaded"},
    {"#% increased Damage"},
    {"* Zana's legacy Map Invasion Boss mod", "Area is inhabited by # additional Invasion Boss"},
    {"* Zana's Map Quantity and Rarity mod",
     "This Map's Modifiers to Quantity of Items found also apply to Rarity"},
    // Master meta-crafting
    {"Prefixes Cannot Be Changed"},
    {"Suffixes Cannot Be Changed"},
    {"Cannot roll Attack Mods"},
    {"Cannot roll Caster Mods"},
    {"Can have multiple Crafted Mods"},
    {"* Leo's Level-28-capped-rolls mod", "Cannot roll Mods with Required Level above #"},
};
*/

QStringListModel &mod_list_model()
{
    return m_mod_list_model;
}

void InitStatTranslations()
{
    spdlog::trace("InitStatTranslations() entered");
    mods.clear();

    // Seed the mods list with pseudo mods.
    for (const auto &[pseudo_mod, real_mods] : summing_pseudo_mods) {
        mods.insert(pseudo_mod);
    }
}

void AddStatTranslations(const QByteArray &statTranslations)
{
    spdlog::trace("AddStatTranslations() entered");

    rapidjson::Document doc;
    doc.Parse(statTranslations.constData());
    if (doc.HasParseError()) {
        spdlog::error(
            "Couldn't properly parse Stat Translations from RePoE, canceling Mods Update");
        return;
    }

    for (auto &translation : doc) {
        for (auto &stat : translation["English"]) {
            if (HasBool(stat, "is_markup") && (stat["is_markup"].GetBool() == true)) {
                // This was added with the change to process json files inside
                // the stat_translations directory. In this case, the necropolis
                // mods from 3.24 have some kind of duplicate formatting with
                // markup that acquisition has not had to deal with before.
                //
                // It's possible this is true for other files in the stat_translations
                // folder, but acquisition has never needed to load modifiers from those
                // files before.
                continue;
            };
            std::vector<QString> formats;
            for (auto &format : stat["format"]) {
                formats.push_back(format.GetString());
            }
            QString stat_string = stat["string"].GetString();
            if (formats[0].compare("ignore") != 0) {
                for (size_t i = 0; i < formats.size(); i++) {
                    QString searchString = "{" + QString::number(i) + "}";
                    stat_string.replace(searchString, formats[i]);
                }
            }
            if (stat_string.length() > 0) {
                mods.insert(stat_string);
            }
        }
    }
}

void InitModList()
{
    spdlog::trace("InitModList() entered");

    std::set<QString> mod_strings;
    mod_generators.clear();
    mods_map.clear();
    for (auto &mod : mods) {
        if (mod_strings.count(mod) > 0) {
            spdlog::warn("InitModList(): duplicate mod: {}", mod);
        } else {
            mod_strings.insert(mod);
            std::vector<QString> list = {mod};
            SumModGen gen = std::make_shared<SumModGenerator>(mod, list);
            mods_map[mod] = gen.get();
            mod_generators.push_back(gen);
        }
    }
    QStringList mod_list;
    mod_list.reserve(mod_strings.size());
    for (auto &mod : mod_strings) {
        mod_list.append(mod);
    }
    mod_list.sort(Qt::CaseInsensitive);
    m_mod_list_model.setStringList(mod_list);
}

void ModGenerator::Generate(const rapidjson::Value &json, ModTable &output)
{
    Generate(json.GetString(), output);
}

SumModGenerator::SumModGenerator(const QString &name, const std::vector<QString> &matches)
    : m_name(name)
    , m_matches(matches)
{}

bool SumModGenerator::Match(const char *mod, double &output)
{
    bool found = false;
    output = 0.0;
    for (auto &match : m_matches) {
        double result = 0.0;
        if (Util::MatchMod(match.toStdString().c_str(), mod, &result)) {
            output += result;
            found = true;
        }
    }
    return found;
}

void SumModGenerator::Generate(const QString &mod, ModTable &output)
{
    double result;
    if (Match(mod.toStdString().c_str(), result)) {
        output[m_name] = result;
    }
}

void AddModToTable(const QString &raw_mod, ModTable &output)
{
    QString generic_mod = raw_mod;

    if (generic_mod.startsWith("1 Added Passive Skill")) {
        // Skip modifiers that appear to be cluster jewel notables.
        // This is such a terrible hack. The entire mods stuff needs to be completely redone.
    } else {
        static const QRegularExpression rep("([0-9\\.]+)");
        generic_mod.replace(rep, "#");
    }

    const std::string raw_str = raw_mod.toStdString();

    // First, process natural modifiers.
    {
        auto it = mods_map.find(generic_mod);
        if (it != mods_map.end()) {
            SumModGenerator *gen = it->second;
            gen->Generate(raw_mod, output);
        }
    }

    // Next, process summing pseudo-mods.
    {
        auto it = summing_pseudo_mods_reverse_lookup.find(generic_mod);
        if (it != summing_pseudo_mods_reverse_lookup.end()) {
            const std::string generic_str = generic_mod.toStdString();

            // Get the list of summing pseudo-mods this modifier impacts.
            const std::vector<QString> &pseudo_mods = it->second;

            // Process each summing pseudo-mod.
            for (const QString &pseudo_mod : pseudo_mods) {
                double value;
                Util::MatchMod(generic_str.c_str(), raw_str.c_str(), &value);
                if (output.contains(pseudo_mod)) {
                    output[pseudo_mod] += value;
                } else {
                    output[pseudo_mod] = value;
                }
            }
        }
    }
}

/*
NormalizedModifier normalize_modifier(QStringView s)
{
    NormalizedModifier r;
    r.normalized.reserve(s.size());

    const qsizetype n = s.size();
    qsizetype i = 0;

    while (i < n) {
        const QChar u0 = s[i];

        const bool startsInt = u0.isDigit();
        const bool startsDotFrac = (u0 == '.') && ((i + 1) < n) && s[i + 1].isDigit();

        if (startsInt || startsDotFrac) {
            r.normalized.append("#"); // one token per number

            // Integer part (optional if starts with ".")
            int64_t intPart = 0;
            while ((i < n) && s[i].isDigit()) {
                intPart = (intPart * 10) + s[i].unicode() - QChar('0').unicode();
                ++i;
            }

            // Fractional part: '.' + digits (optional)
            int64_t frac = 0;     // store up to 2 digits
            int fracCount = 0;    // digits stored (0..2)
            bool roundUp = false; // 3rd fractional digit rounds half-up

            if ((i < n) && (s[i] == '.') && ((i + 1) < n) && s[i + 1].isDigit()) {
                ++i; // consume '.'

                int fracPos = 0;
                while ((i < n) && s[i].isDigit()) {
                    const int d = s[i].unicode() - QChar('0').unicode();

                    if (fracPos < 2) {
                        frac = (frac * 10) + d;
                        ++fracCount;
                    } else if (fracPos == 2) {
                        roundUp = (d >= 5);
                    }
                    ++fracPos;
                    ++i;
                }
            }

            if (fracCount == 1) {
                frac *= 10; // ".5" => 50
            }
            // fracCount == 0 => 0; fracCount == 2 => already correct

            int64_t v = (intPart * 100) + frac;
            if (roundUp) {
                ++v;
            }

            // If you want safety, clamp here. Assuming values fit:
            r.values_x100.push_back(static_cast<int32_t>(v));
            continue;
        }

        // Non-number: lowercase (your original behavior)
        r.normalized.append(u0.toLower());
        ++i;
    }

    return r;
}

uint32_t intern_modifier(const QString &modifier)
{
    const uint32_t id = static_cast<uint32_t>(modifier_strings.size());

    if (id == modifier_strings.capacity()) {
        modifier_strings.reserve(id + 500);
    }

    QString mod = normalize_modifier(QStringView(modifier));

    auto [it, inserted] = modifier_ids.try_emplace(std::move(mod), id);
    if (inserted) {
        // Store a view into the QString owned by the unordered_map key.
        modifier_strings.push_back(QStringView(it->first));
        return id;
    }

    // Already existed; return existing id (and do NOT push to modifier_strings).
    return it->second;
}
*/
