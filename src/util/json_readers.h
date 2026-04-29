// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2025 Tom Holz

#pragma once

#include <QByteArray>

#include <optional>
#include <vector>

namespace poe {
    struct Character;
    struct CharacterListWrapper;
    struct CharacterWrapper;
    struct League;
    struct StashTab;
    struct StashListWrapper;
    struct StashWrapper;
    struct WebStashListWrapper;
} // namespace poe

struct OAuthToken;

// Writers and readers are broken up to avoid an issue on Windows where
// too many templated functions in one file causes a build error and would
// require using /bigobj.

namespace json {

    std::optional<OAuthToken> readOAuthToken(const QByteArray &json);

    std::optional<poe::CharacterWrapper> readCharacterWrapper(const QByteArray &json);
    std::optional<poe::CharacterListWrapper> readCharacterListWrapper(const QByteArray &json);
    std::optional<poe::StashWrapper> readStashWrapper(const QByteArray &json);
    std::optional<poe::StashListWrapper> readStashListWrapper(const QByteArray &json);

    std::optional<poe::WebStashListWrapper> readWebStashListWrapper(const QByteArray &json);

    std::optional<poe::Character> readCharacter(const QByteArray &json);
    std::optional<poe::StashTab> readStash(const QByteArray &json);

    std::optional<std::vector<poe::League>> readLeagueList(const QByteArray &json);
    std::optional<std::vector<poe::Character>> readCharacterList(const QByteArray &json);
    std::optional<std::vector<poe::StashTab>> readStashList(const QByteArray &json);

} // namespace json
