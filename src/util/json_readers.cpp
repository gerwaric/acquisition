// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2025 Tom Holz

#include "util/json_readers.h"

#include "poe/types/character.h"
#include "poe/types/stashtab.h"
#include "util/glaze_qt.h"  // IWYU pragma: keep
#include "util/spdlog_qt.h" // IWYU pragma: keep

poe::Character readCharacter(const QByteArray &json)
{
    poe::Character character;

    const std::string_view sv{json.constBegin(), size_t(json.size())};
    const auto ec = glz::read_json(character, sv);
    if (ec) {
        const auto msg = glz::format_error(ec, sv);
        spdlog::error("Error parsing character: {}", msg);
        return {};
    }
    return character;
}

std::vector<poe::Character> readCharacterList(const QByteArray &json)
{
    std::vector<poe::Character> characters;

    const std::string_view sv{json.constBegin(), size_t(json.size())};
    const auto ec = glz::read_json(characters, sv);
    if (ec) {
        const auto msg = glz::format_error(ec, sv);
        spdlog::error("Error parsing character list: {}", msg);
        return {};
    }
    return characters;
}

poe::StashTab readStash(const QByteArray &json)
{
    poe::StashTab stash;

    const std::string_view sv{json.constBegin(), size_t(json.size())};
    const auto ec = glz::read_json(stash, sv);
    if (ec) {
        const auto msg = glz::format_error(ec, sv);
        spdlog::error("Error parsing stash tab: {}", msg);
        return {};
    }
    return stash;
}

std::vector<poe::StashTab> readStashList(const QByteArray &json)
{
    std::vector<poe::StashTab> stashes;

    const std::string_view sv{json.constBegin(), size_t(json.size())};
    const auto ec = glz::read_json(stashes, sv);
    if (ec) {
        const auto msg = glz::format_error(ec, sv);
        spdlog::error("Error parsing stash tab: {}", msg);
        return {};
    }
    return stashes;
}
