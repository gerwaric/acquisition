// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2025 Tom Holz

#include "util/json_readers.h"

#include "poe/types/character.h"
#include "poe/types/league.h"
#include "poe/types/stashtab.h"
#include "poe/types/website/webstashtab.h"
#include "util/glaze_qt.h" // IWYU pragma: keep
#include "util/oauthtoken.h"
#include "util/spdlog_qt.h" // IWYU pragma: keep

namespace {
    template<typename T>
    std::optional<T> read_json(const QByteArray &json)
    {
        T result;
        const std::string_view str{json.constData(), size_t(json.size())};
        const auto err = glz::read_json(result, str);
        if (err) {
            const auto type = typeid(T).name();
            const auto msg = glz::format_error(err, str);
            spdlog::error("Error reading {} from json: {}", type, msg);
            return std::nullopt;
        }
        return result;
    };
} // namespace

std::optional<OAuthToken> json::readOAuthToken(const QByteArray &json)
{
    return read_json<OAuthToken>(json);
}

std::optional<poe::Character> json::readCharacter(const QByteArray &json)
{
    return read_json<poe::Character>(json);
}

std::optional<poe::CharacterListWrapper> json::readCharacterListWrapper(const QByteArray &json)
{
    return read_json<poe::CharacterListWrapper>(json);
}

std::optional<poe::CharacterWrapper> json::readCharacterWrapper(const QByteArray &json)
{
    return read_json<poe::CharacterWrapper>(json);
}

std::optional<std::vector<poe::Character>> json::readCharacterList(const QByteArray &json)
{
    return read_json<std::vector<poe::Character>>(json);
}

std::optional<std::vector<poe::League>> json::readLeagueList(const QByteArray &json)
{
    return read_json<std::vector<poe::League>>(json);
}

std::optional<poe::StashTab> json::readStash(const QByteArray &json)
{
    return read_json<poe::StashTab>(json);
}

std::optional<poe::StashListWrapper> json::readStashListWrapper(const QByteArray &json)
{
    return read_json<poe::StashListWrapper>(json);
}

std::optional<poe::StashWrapper> json::readStashWrapper(const QByteArray &json)
{
    return read_json<poe::StashWrapper>(json);
}

std::optional<std::vector<poe::StashTab>> json::readStashList(const QByteArray &json)
{
    return read_json<std::vector<poe::StashTab>>(json);
}

std::optional<poe::WebStashListWrapper> json::readWebStashListWrapper(const QByteArray &json)
{
    return read_json<poe::WebStashListWrapper>(json);
}
