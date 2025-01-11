/*
    Copyright (C) 2014-2024 Acquisition Contributors

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

#include "legacybuyoutvalidator.h"

LegacyBuyoutValidator::LegacyBuyoutValidator(const QString& filename)
    : m_filename(filename)
    , m_datastore(filename)
{
    if (!m_datastore.ok) {
        m_status = ValidationResult::Error;
    } else {
        validateTabBuyouts();
        validateItemBuyouts();
        m_status = m_issues.empty()
            ? ValidationResult::Valid
            : ValidationResult::Invalid;
    };
}

void LegacyBuyoutValidator::validateTabBuyouts() {

    const auto& stashes = m_datastore.tabs.stashes;
    const auto& characters = m_datastore.tabs.characters;
    const auto& buyouts = m_datastore.data.tab_buyouts;

    QLocale locale = QLocale::system();
    QLOG_INFO() << "Validating tab buyouts:";
    QLOG_INFO() << "Found" << locale.toString(stashes.size()) << "stash tabs";
    QLOG_INFO() << "Found" << locale.toString(characters.size()) << "characters";
    QLOG_INFO() << "Found" << locale.toString(buyouts.size()) << "tab buyouts";

    using Location = QString;

    std::set<Location> all_locations;
    std::set<Location> duplicated_locations;
    std::set<Location> duplicated_buyouts;
    std::set<Location> ambiguous_buyouts;
    std::set<Location> matched_buyouts;
    std::set<Location> orphaned_buyouts;

    // Add stash tab location tags.
    for (const auto& location : stashes) {
        const Location tag = "stash:" + location.name;
        if (all_locations.count(tag) <= 0) {
            all_locations.insert(tag);
        } else {
            duplicated_locations.insert(tag);
        };
    };

    // Add character location tags.
    for (const auto& location : characters) {
        const Location tag = "character:" + location.name;
        if (all_locations.count(tag) <= 0) {
            all_locations.insert(tag);
        } else {
            duplicated_locations.insert(tag);
        };
    };

    // Validate all the tab buyouts.
    for (const auto& buyout : buyouts) {
        const Location& tag = buyout.first;
        if (matched_buyouts.count(tag) > 0) {
            duplicated_buyouts.insert(tag);
        } else if (all_locations.count(tag) > 0) {
            matched_buyouts.insert(tag);
        } else {
            orphaned_buyouts.insert(tag);
        };
        // If the location tag is one of the duplicated locations,
        // then we don't know which tab this buyout really belongs to.
        if (duplicated_locations.count(tag) > 0) {
            ambiguous_buyouts.insert(tag);
        };
    };

    if (!duplicated_locations.empty()) {
        m_issues["Duplicated Tabs"] = duplicated_locations;
        QLOG_WARN() << "Found" << locale.toString(duplicated_locations.size()) << "duplicated tab locations";
    };
    if (!duplicated_buyouts.empty()) {
        m_issues["Duplicated Tab Buyouts"] = duplicated_buyouts;
        QLOG_WARN() << "Found" << locale.toString(duplicated_buyouts.size()) << "duplicated tab buyouts";
    };
    if (!ambiguous_buyouts.empty()) {
        m_issues["Ambiguous Tab Buyouts"] = ambiguous_buyouts;
        QLOG_WARN() << "Found" << locale.toString(ambiguous_buyouts.size()) << "ambiguous tab buyouts";
    };
    if (!orphaned_buyouts.empty()) {
        m_issues["Orphaned Tab Buyouts"] = orphaned_buyouts;
        QLOG_WARN() << "Found" << locale.toString(orphaned_buyouts.size()) << "orphaned buyouts";
    };
}

void LegacyBuyoutValidator::validateItemBuyouts() {

    const auto& collections = m_datastore.items;
    const auto& buyouts = m_datastore.data.buyouts;

    QLocale locale = QLocale::system();
    QLOG_INFO() << "Validating item buyouts";
    QLOG_INFO() << "Found" << locale.toString(buyouts.size()) << "item buyouts";

    using BuyoutHash = QString;

    std::set<BuyoutHash> unique_buyouts;
    std::set<BuyoutHash> duplicated_buyouts;
    std::set<BuyoutHash> matched_buyouts;
    std::set<BuyoutHash> orphaned_buyouts;

    for (const auto& pair : buyouts) {
        const BuyoutHash& hash = pair.first;
        if (unique_buyouts.count(hash) <= 0) {
            unique_buyouts.insert(hash);
        } else {
            duplicated_buyouts.insert(hash);
        };
    };

    size_t item_count = 0;

    for (const auto& collection : collections) {
        const QString& loc = collection.first;
        const std::vector<LegacyItem>& items = collection.second;
        for (const auto& item : items) {
            const BuyoutHash hash = item.hash();
            if (matched_buyouts.count(hash) > 0) {
                duplicated_buyouts.insert(hash);
            } else if (buyouts.count(hash) > 0) {
                matched_buyouts.insert(hash);
            };
        };
        item_count += items.size();
    };
    QLOG_INFO() << "Found" << locale.toString(item_count) << "items";

    // Now go back and make sure all of the buyouts have beem matched.
    for (const BuyoutHash& hash : unique_buyouts) {
        if (matched_buyouts.count(hash) <= 0) {
            orphaned_buyouts.insert(hash);
        };
    };

    if (!duplicated_buyouts.empty()) {
        m_issues["Duplicated Item Buyouts"] = duplicated_buyouts;
        QLOG_WARN() << "Found" << locale.toString(duplicated_buyouts.size()) << "duplicated item buyouts";
    };
    if (!orphaned_buyouts.empty()) {
        m_issues["Orphaned Item Buyouts"] = orphaned_buyouts;
        QLOG_WARN() << "Found" << locale.toString(orphaned_buyouts.size()) << "orphaned item buyouts";
    };
}
