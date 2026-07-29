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

// The wire shape of a single stash/character reply, with the payload
// captured as the raw substring (F62): glz::raw_json keeps the exact bytes
// GGG sent, including every field the poe:: types do not model. In a named
// namespace (not the anonymous one) because glaze reflection requires the
// type to have linkage.
namespace json::raw {

    struct StashWrapper
    {
        std::optional<glz::raw_json> stash;
    };

    struct CharacterWrapper
    {
        std::optional<glz::raw_json> character;
    };

} // namespace json::raw

namespace {

    template<typename T>
    std::optional<T> read_json(const QByteArray &json)
    {
        // Tolerate unknown keys so new fields in GGG's API responses do
        // not break parsing.
        static constexpr glz::opts opts{.error_on_unknown_keys = false};
        T result;
        const std::string_view str{json.constData(), size_t(json.size())};
        const auto err = glz::read<opts>(result, str);
        if (err) {
            const auto type = typeid(T).name();
            const auto msg = glz::format_error(err, str);
            spdlog::error("Error reading {} from json: {}", type, msg);
            return std::nullopt;
        }
        return result;
    }

    // Capture the reply's sub-object losslessly, then parse the typed
    // payload from that same substring, so the stored bytes and the parsed
    // object cannot diverge. A reply whose sub-object is missing or null
    // yields an empty payload (the caller treats it as "nothing fetched");
    // a sub-object that fails the typed parse fails the whole read.
    template<typename Payload, typename Value, typename Raw>
    std::optional<Payload> read_payload(const QByteArray &json,
                                        std::optional<glz::raw_json> Raw::*member,
                                        std::optional<Value> Payload::*value)
    {
        const auto raw = read_json<Raw>(json);
        if (!raw) {
            return std::nullopt;
        }
        Payload payload;
        if (!(*raw.*member)) {
            return payload;
        }
        const std::string &str = (*raw.*member)->str;
        const auto bytes = QByteArray::fromRawData(str.data(), qsizetype(str.size()));
        auto parsed = read_json<Value>(bytes);
        if (!parsed) {
            return std::nullopt;
        }
        payload.*value = std::move(*parsed);
        payload.bytes = QByteArray(str.data(), qsizetype(str.size()));
        return payload;
    }

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

std::optional<poe::CharacterPayload> json::readCharacterPayload(const QByteArray &json)
{
    return read_payload(json,
                        &json::raw::CharacterWrapper::character,
                        &poe::CharacterPayload::character);
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

std::optional<poe::StashPayload> json::readStashPayload(const QByteArray &json)
{
    return read_payload(json, &json::raw::StashWrapper::stash, &poe::StashPayload::stash);
}

std::optional<std::vector<poe::StashTab>> json::readStashList(const QByteArray &json)
{
    return read_json<std::vector<poe::StashTab>>(json);
}

std::optional<poe::WebStashListWrapper> json::readWebStashListWrapper(const QByteArray &json)
{
    return read_json<poe::WebStashListWrapper>(json);
}
