// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#include "filters/filterspec.h"

#include <utility>

#include "buyoutmanager.h"
#include "filters/filtermatchers.h"
#include "item.h"

FilterCatalog::FilterCatalog(std::vector<FilterSpec> specs)
    : m_specs(std::move(specs))
{}

FilterCatalog BuildFilterCatalog(const BuyoutManager &buyoutManager)
{
    using enum LegacyFilterKind;
    using enum RefreshMode;

    const auto legacy =
        [](LegacyFilterKind kind, const char *caption, FilterGroup group, RefreshMode refreshMode) {
            return FilterSpec{caption, group, refreshMode, LegacyPayload{kind}};
        };
    const auto boolean =
        [](const char *caption, FilterGroup group, std::function<bool(const Item &)> predicate) {
            return FilterSpec{caption, group, Immediate, BoolPayload{std::move(predicate)}};
        };

    std::vector<FilterSpec> specs;
    specs.reserve(38);
    specs.push_back(legacy(Tab, "Tab", FilterGroup::TopForm, Debounced));
    specs.push_back(legacy(Name, "Name", FilterGroup::TopForm, Debounced));
    specs.push_back(legacy(Category, "Category", FilterGroup::TopForm, Debounced));
    specs.push_back(legacy(Rarity, "Rarity", FilterGroup::TopForm, Debounced));
    specs.push_back(legacy(CriticalStrikeChance, "Crit.", FilterGroup::Offense, Debounced));
    specs.push_back(legacy(Dps, "DPS", FilterGroup::Offense, Debounced));
    specs.push_back(legacy(PhysicalDps, "pDPS", FilterGroup::Offense, Debounced));
    specs.push_back(legacy(ElementalDps, "eDPS", FilterGroup::Offense, Debounced));
    specs.push_back(legacy(ChaosDps, "cDPS", FilterGroup::Offense, Debounced));
    specs.push_back(legacy(AttacksPerSecond, "APS", FilterGroup::Offense, Debounced));
    specs.push_back(legacy(Armour, "Armour", FilterGroup::Defense, Debounced));
    specs.push_back(legacy(Evasion, "Evasion", FilterGroup::Defense, Debounced));
    specs.push_back(legacy(EnergyShield, "Shield", FilterGroup::Defense, Debounced));
    specs.push_back(legacy(Block, "Block", FilterGroup::Defense, Debounced));
    specs.push_back(legacy(Sockets, "Sockets", FilterGroup::Sockets, Debounced));
    specs.push_back(legacy(Links, "Links", FilterGroup::Sockets, Debounced));
    specs.push_back(legacy(SocketColors, "Sockets", FilterGroup::Sockets, Immediate));
    specs.push_back(legacy(LinkColors, "Links", FilterGroup::Sockets, Immediate));
    specs.push_back(legacy(RequiredLevel, "R. Level", FilterGroup::Requirements, Debounced));
    specs.push_back(legacy(RequiredStrength, "R. Str", FilterGroup::Requirements, Debounced));
    specs.push_back(legacy(RequiredDexterity, "R. Dex", FilterGroup::Requirements, Debounced));
    specs.push_back(legacy(RequiredIntelligence, "R. Int", FilterGroup::Requirements, Debounced));
    specs.push_back(legacy(Quality, "Quality", FilterGroup::Misc, Debounced));
    specs.push_back(legacy(Level, "Level", FilterGroup::Misc, Debounced));
    specs.push_back(legacy(MapTier, "Map Tier", FilterGroup::Misc, Debounced));
    specs.push_back(legacy(ItemLevel, "ilvl", FilterGroup::Misc, Debounced));
    specs.push_back(boolean("Alt. art", FilterGroup::MiscFlags, MatchesAltart));
    specs.push_back(boolean("Priced", FilterGroup::MiscFlags, [&buyoutManager](const Item &item) {
        return buyoutManager.Get(item).IsActive();
    }));
    specs.push_back(boolean("Unidentified", FilterGroup::MiscFlags2, [](const Item &item) {
        return !item.identified();
    }));
    specs.push_back(boolean("Influenced", FilterGroup::MiscFlags2, [](const Item &item) {
        return item.hasInfluence();
    }));
    specs.push_back(boolean("Crafted", FilterGroup::MiscFlags2, [](const Item &item) {
        return item.crafted();
    }));
    specs.push_back(boolean("Enchanted", FilterGroup::MiscFlags2, [](const Item &item) {
        return item.enchanted();
    }));
    specs.push_back(boolean("Corrupted", FilterGroup::MiscFlags2, [](const Item &item) {
        return item.corrupted();
    }));
    specs.push_back(boolean("Fractured", FilterGroup::MiscFlags2, [](const Item &item) {
        return item.fractured();
    }));
    specs.push_back(
        boolean("Split", FilterGroup::MiscFlags2, [](const Item &item) { return item.split(); }));
    specs.push_back(boolean("Synthesized", FilterGroup::MiscFlags2, [](const Item &item) {
        return item.synthesized();
    }));
    specs.push_back(boolean("Mutated", FilterGroup::MiscFlags2, [](const Item &item) {
        return item.mutated();
    }));
    specs.push_back(legacy(Mods, "Mods", FilterGroup::Mods, Debounced));
    return FilterCatalog(std::move(specs));
}
