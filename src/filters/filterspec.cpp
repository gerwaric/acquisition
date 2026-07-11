// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#include "filters/filterspec.h"

#include <utility>

FilterCatalog::FilterCatalog(std::vector<FilterSpec> specs)
    : m_specs(std::move(specs))
{}

FilterCatalog BuildFilterCatalog(const BuyoutManager & /* buyoutManager */)
{
    using enum LegacyFilterKind;
    using enum RefreshMode;

    const auto legacy =
        [](LegacyFilterKind kind, const char *caption, FilterGroup group, RefreshMode refreshMode) {
            return FilterSpec{caption, group, refreshMode, LegacyPayload{kind}};
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
    specs.push_back(legacy(AlternateArt, "Alt. art", FilterGroup::MiscFlags, Immediate));
    specs.push_back(legacy(Priced, "Priced", FilterGroup::MiscFlags, Immediate));
    specs.push_back(legacy(Unidentified, "Unidentified", FilterGroup::MiscFlags2, Immediate));
    specs.push_back(legacy(Influenced, "Influenced", FilterGroup::MiscFlags2, Immediate));
    specs.push_back(legacy(Crafted, "Crafted", FilterGroup::MiscFlags2, Immediate));
    specs.push_back(legacy(Enchanted, "Enchanted", FilterGroup::MiscFlags2, Immediate));
    specs.push_back(legacy(Corrupted, "Corrupted", FilterGroup::MiscFlags2, Immediate));
    specs.push_back(legacy(Fractured, "Fractured", FilterGroup::MiscFlags2, Immediate));
    specs.push_back(legacy(Split, "Split", FilterGroup::MiscFlags2, Immediate));
    specs.push_back(legacy(Synthesized, "Synthesized", FilterGroup::MiscFlags2, Immediate));
    specs.push_back(legacy(Mutated, "Mutated", FilterGroup::MiscFlags2, Immediate));
    specs.push_back(legacy(Mods, "Mods", FilterGroup::Mods, Debounced));
    return FilterCatalog(std::move(specs));
}
