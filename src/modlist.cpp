/*
    Copyright 2014 Ilya Zhuravlev

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

#include <memory>
#include <regex>
#include <set>
#include <string>
#include <unordered_map>
#include <utility>
#include <vector>

#include <QStringList>

#include <boost/algorithm/string.hpp>
#include "QsLog.h"

#include "item.h"
#include "util.h"

QStringListModel mod_list_model_;
std::set<std::string> mods;
std::unordered_map<std::string, SumModGenerator*> mods_map;
std::vector<SumModGen> mod_generators;

/* ------------------- TBD - FIX THIS!!! --------------------------

// These are just summed, and the mod named as the first element of a vector is generated with value equaling the sum.
// Both implicit and explicit fields are considered.
// This is pretty much the same list as poe.trade uses
//
// NOTE: This appears to need updating --gerwaric (2024-05-06)
const std::vector<std::vector<std::string>> simple_sum = {
    { "#% increased Quantity of Items found" },
    { "#% increased Rarity of Items found" },
    { "+# to maximum Life" },
    { "#% increased maximum Life" },
    { "+# to maximum Energy Shield" },
    { "#% increased maximum Energy Shield" },
    { "+# to maximum Mana" },
    { "#% increased Mana Regeneration Rate" },
    { "+#% to Fire Resistance", "+#% to Fire and Cold Resistances", "+#% to Fire and Lightning Resistances", "+#% to all Elemental Resistances", "+#% to Fire and Chaos Resistances" },
    { "+#% to Cold Resistance", "+#% to Fire and Cold Resistances", "+#% to Cold and Lightning Resistances", "+#% to all Elemental Resistances", "+#% to Cold and Chaos Resistances" },
    { "+#% to Lightning Resistance", "+#% to Cold and Lightning Resistances", "+#% to Fire and Lightning Resistances", "+#% to all Elemental Resistances", "+#% to Lightning and Chaos Resistances" },
    { "+#% to all Elemental Resistances" },
    { "+#% to Chaos Resistance", "+#% to Fire and Chaos Resistances", "+#% to Cold and Chaos Resistances", "+#% to Lightning and Chaos Resistances" },
    { "+# to Level of Socketed Aura Gems", "+# to Level of Socketed Gems" },
    { "+# to Level of Socketed Fire Gems", "+# to Level of Socketed Gems" },
    { "+# to Level of Socketed Cold Gems", "+# to Level of Socketed Gems" },
    { "+# to Level of Socketed Lightning Gems", "+# to Level of Socketed Gems" },
    { "+# to Level of Socketed Chaos Gems", "+# to Level of Socketed Gems" },
    { "+# to Level of Socketed Elemental Gems", "+# to Level of Socketed Gems" },
    { "+# to Level of Socketed Spell Gems", "+# to Level of Socketed Gems" },
    { "+# to Level of Socketed Bow Gems", "+# to Level of Socketed Gems" },
    { "+# to Level of Socketed Minion Gems", "+# to Level of Socketed Gems" },
    { "+# to Level of Socketed Vaal Gems", "+# to Level of Socketed Gems" },
    { "+# to Level of Socketed Melee Gems", "+# to Level of Socketed Gems" },
    { "+# to Level of Socketed Movement Gems", "+# to Level of Socketed Gems" },
    { "+# to Level of Socketed Strength Gems", "+# to Level of Socketed Gems" },
    { "#% increased Physical Damage", "#% increased Global Physical Damage", "#% increased Damage" },
    { "#% increased Spell Damage", "#% increased Damage" },
    { "#% increased Elemental Damage", "#% increased Damage" },
    { "#% increased Fire Damage", "#% increased Elemental Damage", "#% increased Damage" },
    { "#% increased Cold Damage", "#% increased Elemental Damage", "#% increased Damage" },
    { "#% increased Lightning Damage", "#% increased Elemental Damage", "#% increased Damage" },
    { "#% increased Chaos Damage", "#% increased Damage" },
    { "#% increased Fire Damage with Attack Skills", "#% increased Fire Damage", "#% increased Elemental Damage with Attack Skills", "#% increased Elemental Damage", "#% increased Damage" },
    { "#% increased Cold Damage with Attack Skills", "#% increased Cold Damage", "#% increased Elemental Damage with Attack Skills", "#% increased Elemental Damage", "#% increased Damage" },
    { "#% increased Lightning Damage with Attack Skills", "#% increased Lightning Damage", "#% increased Elemental Damage with Attack Skills", "#% increased Elemental Damage", "#% increased Damage" },
    { "#% increased Elemental Damage with Attack Skills", "#% increased Elemental Damage", "#% increased Damage" },
    { "#% increased Fire Spell Damage", "#% increased Fire Damage", "#% increased Elemental Damage", "#% increased Spell Damage", "#% increased Damage" },
    { "#% increased Cold Spell Damage", "#% increased Cold Damage", "#% increased Elemental Damage", "#% increased Spell Damage", "#% increased Damage" },
    { "#% increased Lightning Spell Damage", "#% increased Lightning Damage", "#% increased Elemental Damage", "#% increased Spell Damage", "#% increased Damage" },
    { "#% increased Global Critical Strike Chance" },
    { "#% increased Critical Strike Chance for Spells", "#% increased Global Critical Strike Chance", "Spells have +#% to Critical Strike Chance " },
    { "+#% to Global Critical Strike Multiplier" },
    { "#% to Melee Critical Strike Multiplier" },
    { "#% to Critical Strike Multiplier with Elemental Skills", "#% to Global Critical Strike Multiplier" },
    { "#% to Critical Strike Multiplier with Fire Skills", "#% to Critical Strike Multiplier with Elemental Skills", "#% to Global Critical Strike Multiplier" },
    { "#% to Critical Strike Multiplier with Cold Skills", "#% to Critical Strike Multiplier with Elemental Skills", "#% to Global Critical Strike Multiplier" },
    { "#% to Critical Strike Multiplier with Lightning Skills", "#% to Critical Strike Multiplier with Elemental Skills", "#% to Global Critical Strike Multiplier" },
    { "#% to Critical Strike Multiplier with One Handed Melee Weapons", "#% to Global Critical Strike Multiplier" },
    { "#% to Critical Strike Multiplier while Dual Wielding", "#% to Global Critical Strike Multiplier" },
    { "#% to Critical Strike Multiplier with Two Handed Melee Weapons", "#% to Global Critical Strike Multiplier" },
    { "#% to Critical Strike Multiplier for Spells", "#% to Global Critical Strike Multiplier" },
    { "#% to Critical Strike Multiplier with Fire Spells", "#% to Critical Strike Multiplier for Spells", "#% to Critical Strike Multiplier with Fire Skills", "#% to Critical Strike Multiplier with Elemental Skills", "#% to Global Critical Strike Multiplier" },
    { "#% to Critical Strike Multiplier with Cold Spells", "#% to Critical Strike Multiplier for Spells", "#% to Critical Strike Multiplier with Fire Skills", "#% to Critical Strike Multiplier with Elemental Skills", "#% to Global Critical Strike Multiplier" },
    { "#% to Critical Strike Multiplier with Lightning Spells", "#% to Critical Strike Multiplier for Spells", "#% to Critical Strike Multiplier with Fire Skills", "#% to Critical Strike Multiplier with Elemental Skills", "#% to Global Critical Strike Multiplier" },
    { "+# to Accuracy Rating" },
    { "#% increased Accuracy Rating" },
    { "#% increased Area Damage", "#% increased Damage" },
    { "#% increased Damage over Time", "#% increased Damage" },
    { "#% increased Burning Damage", "#% increased Fire Damage", "#% increased Elemental Damage", "#% increased Damage over Time", "#% increased Damage" },
    { "#% of Physical Attack Damage Leeched as Life" },
    { "#% of Physical Attack Damage Leeched as Mana" },
    { "Adds # Physical Damage to Attacks", "Adds # to # Physical Damage", "Adds # to # Physical Damage to Attacks" },
    { "Adds # Elemental Damage to Attacks", "Adds # to # Fire Damage", "Adds # to # Cold Damage", "Adds # to # Lightning Damage", "Adds # to # Fire Damage to Attacks", "Adds # to # Cold Damage to Attacks", "Adds # to # Lightning Damage to Attacks" },
    { "Adds # Chaos Damage to Attacks", "Adds # to # Chaos Damage to Attacks" },
    { "Adds # Damage to Attacks", "Adds # to # Fire Damage", "Adds # to # Cold Damage", "Adds # to # Lightning Damage", "Adds # to # Physical Damage", "Adds # to # Chaos Damage", "Adds # to # Fire Damage to Attacks", "Adds # to # Cold Damage to Attacks", "Adds # to # Lightning Damage to Attacks", "Adds # to # Physical Damage to Attacks", "Adds # to # Chaos Damage to Attacks" },
    { "Adds # Elemental Damage to Spells", "Adds # to # Fire Damage to Spells", "Adds # to # Cold Damage to Spells", "Adds # to # Lightning Damage to Spells" },
    { "Adds # Chaos Damage to Spells", "Adds # to # Chaos Damage to Spells" },
    { "Adds # Damage to Spells", "Adds # to # Fire Damage to Spells", "Adds # to # Cold Damage to Spells", "Adds # to # Lightning Damage to Spells", "Adds # to # Chaos Damage to Spells" },
    { "#% increased Attack Speed", "#% increased Attack and Cast Speed" },
    { "#% increased Cast Speed", "#% increased Attack and Cast Speed" },
    { "#% increased Movement Speed" },
    { "+# to Dexterity", "+# to Strength and Dexterity", "+# to Dexterity and Intelligence", "+# to all Attributes" },
    { "+# to Strength", "+# to Strength and Dexterity", "+# to Strength and Intelligence", "+# to all Attributes" },
    { "+# to Intelligence", "+# to Dexterity and Intelligence", "+# to Strength and Intelligence", "+# to all Attributes" },
    { "+# to all Attributes" },
    { "#% increased Stun Duration on enemies" },
    { "#% increased Block and Stun Recovery" },
    // Master mods
    { "+#% to Quality of Socketed Support Gems" },
    { "-# to Total Mana Cost of Skills" },
    { "+# to Level of Socketed Support Gems", "+# to Level of Socketed Gems" },
    { "Causes Bleeding on Hit" },
    { "#% increased Life Leeched per second" },
    { "Hits can't be Evaded" },
    { "#% increased Damage" },
    { "* Zana's legacy Map Invasion Boss mod", "Area is inhabited by # additional Invasion Boss" },
    { "* Zana's Map Quantity and Rarity mod", "This Map's Modifiers to Quantity of Items found also apply to Rarity" },
    // Master meta-crafting
    { "Prefixes Cannot Be Changed" },
    { "Suffixes Cannot Be Changed" },
    { "Cannot roll Attack Mods" },
    { "Cannot roll Caster Mods" },
    { "Can have multiple Crafted Mods" },
    { "* Leo's Level-28-capped-rolls mod", "Cannot roll Mods with Required Level above #" },
};
*/

QStringListModel& mod_list_model() {
    return mod_list_model_;
}

void InitStatTranslations() {
    QLOG_TRACE() << "InitStatTranslations() entered";
    mods.clear();
}

void AddStatTranslations(const QByteArray& statTranslations) {
    QLOG_TRACE() << "AddStatTranslations() entered";

    rapidjson::Document doc;
    doc.Parse(statTranslations.constData());
    if (doc.HasParseError()) {
        QLOG_ERROR() << "Couldn't properly parse Stat Translations from RePoE, canceling Mods Update";
        return;
    };

    for (auto& translation : doc) {
        for (auto& stat : translation["English"]) {
            if (stat.HasMember("is_markup") && (stat["is_markup"].GetBool() == true)) {
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
            std::vector<std::string> formats;
            for (auto& format : stat["format"]) {
                formats.push_back(format.GetString());
            };
            std::string stat_string = stat["string"].GetString();
            if (formats[0].compare("ignore") != 0) {
                for (size_t i = 0; i < formats.size(); i++) {
                    std::string searchString = "{" + std::to_string(i) + "}";
                    boost::replace_all(stat_string, searchString, formats[i]);
                };
            };
            if (stat_string.length() > 0) {
                mods.insert(stat_string);
            };
        };
    };
}

void InitModList() {
    QLOG_TRACE() << "InitModList() entered";

    std::set<std::string> mod_strings;
    mod_generators.clear();
    mods_map.clear();
    for (auto& mod : mods) {
        if (mod_strings.count(mod) > 0) {
            QLOG_WARN() << "InitModList(): duplicate mod:" << mod;
        } else {
            mod_strings.insert(mod);
            std::vector<std::string> list = { mod };
            SumModGen gen = std::make_shared<SumModGenerator>(mod, list);
            mods_map.insert(std::make_pair(mod, gen.get()));
            mod_generators.push_back(gen);
        };
    };
    QStringList mod_list;
    mod_list.reserve(mod_strings.size());
    for (auto& mod : mod_strings) {
        mod_list.append(QString::fromStdString(mod));
    };
    mod_list.sort(Qt::CaseInsensitive);
    mod_list_model_.setStringList(mod_list);
}

void ModGenerator::Generate(const rapidjson::Value& json, ModTable* output) {
    Generate(json.GetString(), output);
}

SumModGenerator::SumModGenerator(const std::string& name, const std::vector<std::string>& matches) :
    name_(name),
    matches_(matches)
{}

bool SumModGenerator::Match(const char* mod, double* output) {
    bool found = false;
    *output = 0.0;
    for (auto& match : matches_) {
        double result = 0.0;
        if (Util::MatchMod(match.c_str(), mod, &result)) {
            *output += result;
            found = true;
        };
    };
    return found;
}

void SumModGenerator::Generate(const std::string& mod, ModTable* output) {
    double result;
    if (Match(mod.c_str(), &result)) {
        (*output)[name_] = result;
    };
}

void AddModToTable(const std::string& raw_mod, ModTable* output) {
    const std::regex rep("([0-9\\.]+)");
    const std::string mod = std::regex_replace(raw_mod, rep, "#");
    auto rslt = mods_map.find(mod);
    if (rslt != mods_map.end()) {
        SumModGenerator* gen = rslt->second;
        gen->Generate(raw_mod, output);
    };
}
