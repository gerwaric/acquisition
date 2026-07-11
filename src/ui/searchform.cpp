// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#include "ui/searchform.h"

#include <QAbstractItemModel>
#include <QLabel>
#include <QStringListModel>
#include <QVBoxLayout>
#include <QWidget>

#include <array>

#include "buyoutmanager.h"
#include "itemcategories.h"
#include "modsfilter.h"
#include "ui/flowlayout.h"

SearchForm::SearchForm(QVBoxLayout &layout,
                       const FilterCatalog &catalog,
                       BuyoutManager &buyoutManager,
                       const FilterCallbacks &callbacks)
    : m_layout(layout)
    , m_catalog(catalog)
{
    auto categoryModel = std::make_unique<QStringListModel>(GetItemCategories());
    auto rarityModel = std::make_unique<QStringListModel>(RaritySearchFilter::RARITY_LIST);
    auto *const categoryModelPtr = categoryModel.get();
    auto *const rarityModelPtr = rarityModel.get();
    m_models.push_back(std::move(categoryModel));
    m_models.push_back(std::move(rarityModel));

    m_slots.reserve(static_cast<size_t>(m_catalog.size()));
    m_legacyFilters.reserve(static_cast<size_t>(m_catalog.size()));
    const auto add = [this]<typename FilterType, typename... Args>(Args &&...args) {
        auto filter = std::make_unique<FilterType>(std::forward<Args>(args)...);
        m_legacyFilters.push_back(filter.get());
        m_slots.emplace_back(std::move(filter));
    };

    Q_ASSERT(m_catalog.size() >= 4);
    constexpr std::array topKinds{LegacyFilterKind::Tab,
                                  LegacyFilterKind::Name,
                                  LegacyFilterKind::Category,
                                  LegacyFilterKind::Rarity};
    for (qsizetype index = 0; index < static_cast<qsizetype>(topKinds.size()); ++index) {
        const auto *legacy = std::get_if<LegacyPayload>(&m_catalog[index].payload);
        Q_ASSERT(legacy && legacy->kind == topKinds[static_cast<size_t>(index)]);
    }
    add.template operator()<TabSearchFilter>(&m_layout, callbacks);
    add.template operator()<NameSearchFilter>(&m_layout, callbacks);
    add.template operator()<CategorySearchFilter>(&m_layout, categoryModelPtr, callbacks);
    add.template operator()<RaritySearchFilter>(&m_layout, rarityModelPtr, callbacks);

    auto *offenseLayout = new FlowLayout;
    auto *defenseLayout = new FlowLayout;
    auto *socketsLayout = new FlowLayout;
    auto *requirementsLayout = new FlowLayout;
    auto *miscLayout = new FlowLayout;
    auto *miscFlagsLayout = new FlowLayout;
    auto *miscFlags2Layout = new FlowLayout;
    auto *modsLayout = new QVBoxLayout;

    addSearchGroup(offenseLayout, "Offense");
    addSearchGroup(defenseLayout, "Defense");
    addSearchGroup(socketsLayout, "Sockets");
    addSearchGroup(requirementsLayout, "Requirements");
    addSearchGroup(miscLayout, "Misc");
    addSearchGroup(miscFlagsLayout);
    addSearchGroup(miscFlags2Layout);
    addSearchGroup(modsLayout, "Mods");

    for (qsizetype index = 4; index < m_catalog.size(); ++index) {
        const auto &spec = m_catalog[index];
        const auto *legacy = std::get_if<LegacyPayload>(&spec.payload);
        Q_ASSERT(legacy);
        if (!legacy) {
            continue;
        }

        using Kind = LegacyFilterKind;
        switch (legacy->kind) {
        case Kind::Tab:
            add.template operator()<TabSearchFilter>(&m_layout, callbacks);
            break;
        case Kind::Name:
            add.template operator()<NameSearchFilter>(&m_layout, callbacks);
            break;
        case Kind::Category:
            add.template operator()<CategorySearchFilter>(&m_layout, categoryModelPtr, callbacks);
            break;
        case Kind::Rarity:
            add.template operator()<RaritySearchFilter>(&m_layout, rarityModelPtr, callbacks);
            break;
        case Kind::CriticalStrikeChance:
            add.template operator()<SimplePropertyFilter>(offenseLayout,
                                                          "Critical Strike Chance",
                                                          "Crit.",
                                                          callbacks);
            break;
        case Kind::Dps:
            add.template operator()<ItemMethodFilter>(
                offenseLayout, [](Item *item) { return item->DPS(); }, "DPS", callbacks);
            break;
        case Kind::PhysicalDps:
            add.template operator()<ItemMethodFilter>(
                offenseLayout, [](Item *item) { return item->pDPS(); }, "pDPS", callbacks);
            break;
        case Kind::ElementalDps:
            add.template operator()<ItemMethodFilter>(
                offenseLayout, [](Item *item) { return item->eDPS(); }, "eDPS", callbacks);
            break;
        case Kind::ChaosDps:
            add.template operator()<ItemMethodFilter>(
                offenseLayout, [](Item *item) { return item->cDPS(); }, "cDPS", callbacks);
            break;
        case Kind::AttacksPerSecond:
            add.template operator()<SimplePropertyFilter>(offenseLayout,
                                                          "Attacks per Second",
                                                          "APS",
                                                          callbacks);
            break;
        case Kind::Armour:
            add.template operator()<SimplePropertyFilter>(defenseLayout, "Armour", callbacks);
            break;
        case Kind::Evasion:
            add.template operator()<SimplePropertyFilter>(defenseLayout,
                                                          "Evasion Rating",
                                                          "Evasion",
                                                          callbacks);
            break;
        case Kind::EnergyShield:
            add.template operator()<SimplePropertyFilter>(defenseLayout,
                                                          "Energy Shield",
                                                          "Shield",
                                                          callbacks);
            break;
        case Kind::Block:
            add.template operator()<SimplePropertyFilter>(defenseLayout,
                                                          "Chance to Block",
                                                          "Block",
                                                          callbacks);
            break;
        case Kind::Sockets:
            add.template operator()<SocketsFilter>(socketsLayout, "Sockets", callbacks);
            break;
        case Kind::Links:
            add.template operator()<LinksFilter>(socketsLayout, "Links", callbacks);
            break;
        case Kind::SocketColors:
            add.template operator()<SocketsColorsFilter>(socketsLayout, callbacks);
            break;
        case Kind::LinkColors:
            add.template operator()<LinksColorsFilter>(socketsLayout, callbacks);
            break;
        case Kind::RequiredLevel:
            add.template operator()<RequiredStatFilter>(requirementsLayout,
                                                        "Level",
                                                        "R. Level",
                                                        callbacks);
            break;
        case Kind::RequiredStrength:
            add.template operator()<RequiredStatFilter>(requirementsLayout,
                                                        "Str",
                                                        "R. Str",
                                                        callbacks);
            break;
        case Kind::RequiredDexterity:
            add.template operator()<RequiredStatFilter>(requirementsLayout,
                                                        "Dex",
                                                        "R. Dex",
                                                        callbacks);
            break;
        case Kind::RequiredIntelligence:
            add.template operator()<RequiredStatFilter>(requirementsLayout,
                                                        "Int",
                                                        "R. Int",
                                                        callbacks);
            break;
        case Kind::Quality:
            add.template operator()<DefaultPropertyFilter>(miscLayout, "Quality", 0, callbacks);
            break;
        case Kind::Level:
            add.template operator()<SimplePropertyFilter>(miscLayout, "Level", callbacks);
            break;
        case Kind::MapTier:
            add.template operator()<SimplePropertyFilter>(miscLayout, "Map Tier", callbacks);
            break;
        case Kind::ItemLevel:
            add.template operator()<ItemlevelFilter>(miscLayout, "ilvl", callbacks);
            break;
        case Kind::AlternateArt:
            add.template operator()<AltartFilter>(miscFlagsLayout, "", "Alt. art", callbacks);
            break;
        case Kind::Priced:
            add.template operator()<PricedFilter>(miscFlagsLayout,
                                                  "",
                                                  "Priced",
                                                  callbacks,
                                                  buyoutManager);
            break;
        case Kind::Unidentified:
            add.template operator()<UnidentifiedFilter>(miscFlags2Layout,
                                                        "",
                                                        "Unidentified",
                                                        callbacks);
            break;
        case Kind::Influenced:
            add.template operator()<InfluencedFilter>(miscFlags2Layout, "", "Influenced", callbacks);
            break;
        case Kind::Crafted:
            add.template operator()<CraftedFilter>(miscFlags2Layout, "", "Crafted", callbacks);
            break;
        case Kind::Enchanted:
            add.template operator()<EnchantedFilter>(miscFlags2Layout, "", "Enchanted", callbacks);
            break;
        case Kind::Corrupted:
            add.template operator()<CorruptedFilter>(miscFlags2Layout, "", "Corrupted", callbacks);
            break;
        case Kind::Fractured:
            add.template operator()<FracturedFilter>(miscFlags2Layout, "", "Fractured", callbacks);
            break;
        case Kind::Split:
            add.template operator()<SplitFilter>(miscFlags2Layout, "", "Split", callbacks);
            break;
        case Kind::Synthesized:
            add.template operator()<SynthesizedFilter>(miscFlags2Layout,
                                                       "",
                                                       "Synthesized",
                                                       callbacks);
            break;
        case Kind::Mutated:
            add.template operator()<MutatedFilter>(miscFlags2Layout, "", "Mutated", callbacks);
            break;
        case Kind::Mods:
            add.template operator()<ModsFilter>(modsLayout, callbacks);
            break;
        }
    }

    Q_ASSERT(m_slots.size() == static_cast<size_t>(m_catalog.size()));
    Q_ASSERT(m_legacyFilters.size() == static_cast<size_t>(m_catalog.size()));
}

SearchForm::~SearchForm() = default;

void SearchForm::addSearchGroup(QLayout *layout, const QString &name)
{
    if (!name.isEmpty()) {
        m_layout.addWidget(new QLabel("<h3>" + name + "</h3>"));
    }
    layout->setContentsMargins(0, 0, 0, 0);
    auto *layoutContainer = new QWidget;
    layoutContainer->setLayout(layout);
    m_layout.addWidget(layoutContainer);
}
