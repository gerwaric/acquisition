// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

#include "datastore.h"

#include <legacy/legacycharacter.h>
#include <legacy/legacyitemlocation.h>
#include <legacy/legacystash.h>
#include <poe/types/item.h>
#include <poe/types/stashtab.h>
#include <util/spdlog_qt.h>
#include <util/util.h>

void DataStore::SetInt(const QString &key, int value)
{
    Set(key, QString::number(value));
}

int DataStore::GetInt(const QString &key, int default_value)
{
    return Get(key, QString::number(default_value)).toInt();
}

QString DataStore::Serialize(const Locations &tabs)
{
    QStringList json;
    json.reserve(tabs.size());
    for (auto &tab : tabs) {
        json.push_back(tab.get_json());
    }
    return "[" + json.join(",") + "]";
}

QString DataStore::Serialize(const Items &items)
{
    QStringList json;
    json.reserve(items.size());
    for (auto &item : items) {
        json.push_back(item->json());
    }
    return "[" + json.join(",") + "]";
}

Locations DataStore::DeserializeTabs(const QString &json, ItemLocationType type)
{
    const auto bytes = json.toUtf8();
    const std::string_view sv{bytes, size_t(bytes.size())};

    switch (type) {
    case ItemLocationType::STASH:
        return DeserializeStashTabs(sv);
    case ItemLocationType::CHARACTER:
        return DeserializeCharacterTabs(sv);
    }

    spdlog::error("DataStore: cannot deserialize tabs: invalid location type: {}", type);
    return {};
}

Locations DataStore::DeserializeStashTabs(std::string_view json)
{
    const auto result = glz::read_json<std::vector<LegacyStash>>(json);
    if (!result) {
        const auto msg = glz::format_error(result.error(), json);
        spdlog::error("DataStore: error deserializing stash tabs: {}", msg);
        return {};
    }

    const auto stashes = *result;

    Locations tabs;
    tabs.reserve(stashes.size());
    for (const auto &stash : stashes) {
        tabs.emplace_back(stash);
    }
    return tabs;
}

Locations DataStore::DeserializeCharacterTabs(std::string_view json)
{
    const auto result = glz::read_json<std::vector<LegacyCharacter>>(json);
    if (!result) {
        const auto msg = glz::format_error(result.error(), json);
        spdlog::error("DataStore: error deserializing character tabs: {}", msg);
        return {};
    }

    const auto characters = *result;

    Locations tabs;
    tabs.reserve(characters.size());
    for (const auto &character : characters) {
        tabs.emplace_back(character, tabs.size());
    }
    return tabs;
}

Items DataStore::DeserializeItems(const QString &json, const ItemLocation &tab)
{
    const auto bytes{json.toUtf8()};
    const std::string_view sv{bytes, size_t(bytes.size())};

    std::vector<poe::Item> parsed_items;
    std::vector<LegacyItemLocation> parsed_locations;

    // We have to allow unknown keys otherwise parsing poe::Item will
    // break beacuse acquisition has injected it's special location metadata
    // such as _type, _socketed, _x, and _y.
    constexpr glz::opts permissive{.error_on_unknown_keys = false};

    // First parse the items ignoring the special location info.
    auto ec = glz::read<permissive>(parsed_items, sv);
    if (ec) {
        const auto msg = glz::format_error(ec, sv);
        spdlog::error("DataStore: error deserializing items: {}", msg);
        return {};
    }

    // Now parse the magic item location info acquisition attaches to each item.
    ec = glz::read<permissive>(parsed_locations, sv);
    if (ec) {
        const auto msg = glz::format_error(ec, sv);
        spdlog::error("DataStore: error deserializing legacy item location data: {}", msg);
        return {};
    }

    const size_t num_items{parsed_items.size()};

    // Make sure the two results have the same size.
    if (parsed_locations.size() != num_items) {
        spdlog::error("DataStore: deserialization mismatch: {} items, {} locations",
                      parsed_items.size(),
                      parsed_locations.size());
        return {};
    }

    // Preallocate the return value.
    Items items;
    items.reserve(num_items);

    // Iterate over each item in the serialized json.
    for (size_t i = 0; i < num_items; ++i) {
        const auto &item = parsed_items[i];
        const auto &location_info = parsed_locations[i];
        // Create a new location and make sure location-related information
        // such as x and y are pulled from the item json.
        ItemLocation loc = tab;
        loc.FromItem(item);
        loc.FromLegacyItemLocation(location_info);
        items.push_back(std::make_shared<Item>(item, loc));
    }
    return items;
}
