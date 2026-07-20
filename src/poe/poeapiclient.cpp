// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#include "poe/poeapiclient.h"

#include <utility>

#include <QNetworkRequest>
#include <QUrl>
#include <QUrlQuery>

#include "poe/poe_utils.h"
#include "ratelimit/ratelimiter.h"
#include "replytimeout.h"
#include "util/json_readers.h"

namespace {

    using RateLimit::FetchError;
    using RateLimit::FetchOutcome;

    // Chain a parse onto a fetch. The continuation is exception-tight
    // (IR4): anything that escapes becomes a value, because a throwing
    // continuation produces an EXCEPTIONAL QFuture, which crosses the
    // value-only boundary and rethrows out of the caller's co_await.
    //
    // No context object and values captured by copy — a context would bind
    // the continuation to an object's lifetime and thread affinity, and the
    // facade has neither.
    template<typename T, typename Reader>
    QFuture<std::expected<T, FetchError>> ParseInto(QFuture<FetchOutcome> fetch,
                                                    Reader reader,
                                                    QString endpoint,
                                                    QUrl url)
    {
        return fetch.then([reader, endpoint, url](
                              const FetchOutcome &outcome) -> std::expected<T, FetchError> {
            if (!outcome) {
                // Fetch failures pass through untouched: the pump already
                // said everything there is to say about them.
                return std::unexpected(outcome.error());
            }
            try {
                auto parsed = reader(*outcome);
                if (!parsed) {
                    FetchError error;
                    error.kind = FetchError::Kind::Parse;
                    error.endpoint = endpoint;
                    error.url = url;
                    error.message = QString("could not parse the response from '%1'").arg(endpoint);
                    return std::unexpected(std::move(error));
                }
                return std::move(*parsed);
            } catch (const std::exception &e) {
                FetchError error;
                error.kind = FetchError::Kind::Parse;
                error.endpoint = endpoint;
                error.url = url;
                error.message = QString("parsing the response from '%1' threw: %2")
                                    .arg(endpoint, QString::fromUtf8(e.what()));
                return std::unexpected(std::move(error));
            } catch (...) {
                FetchError error;
                error.kind = FetchError::Kind::Internal;
                error.endpoint = endpoint;
                error.url = url;
                error.message = QString("parsing the response from '%1' threw an unknown exception")
                                    .arg(endpoint);
                return std::unexpected(std::move(error));
            }
        });
    }

} // namespace

PoeApiClient::Result<poe::StashListWrapper> PoeApiClient::listStashes(const QString &realm,
                                                                      const QString &league,
                                                                      std::stop_token token)
{
    const auto [endpoint, request] = poe::MakeStashListRequest(realm, league);
    return ParseInto<poe::StashListWrapper>(m_rate_limiter.SubmitFuture(endpoint,
                                                                        request,
                                                                        std::move(token)),
                                            &json::readStashListWrapper,
                                            endpoint,
                                            request.url());
}

PoeApiClient::Result<poe::StashWrapper> PoeApiClient::getStash(const QString &realm,
                                                               const QString &league,
                                                               const QString &stash_id,
                                                               const QString &substash_id,
                                                               std::stop_token token)
{
    const auto [endpoint, request] = poe::MakeStashRequest(realm, league, stash_id, substash_id);
    return ParseInto<poe::StashWrapper>(m_rate_limiter.SubmitFuture(endpoint,
                                                                    request,
                                                                    std::move(token)),
                                        &json::readStashWrapper,
                                        endpoint,
                                        request.url());
}

PoeApiClient::Result<poe::CharacterListWrapper> PoeApiClient::listCharacters(const QString &realm,
                                                                             std::stop_token token)
{
    const auto [endpoint, request] = poe::MakeCharacterListRequest(realm);
    return ParseInto<poe::CharacterListWrapper>(m_rate_limiter.SubmitFuture(endpoint,
                                                                            request,
                                                                            std::move(token)),
                                                &json::readCharacterListWrapper,
                                                endpoint,
                                                request.url());
}

PoeApiClient::Result<poe::CharacterWrapper> PoeApiClient::getCharacter(const QString &realm,
                                                                       const QString &name,
                                                                       std::stop_token token)
{
    const auto [endpoint, request] = poe::MakeCharacterRequest(realm, name);
    return ParseInto<poe::CharacterWrapper>(m_rate_limiter.SubmitFuture(endpoint,
                                                                        request,
                                                                        std::move(token)),
                                            &json::readCharacterWrapper,
                                            endpoint,
                                            request.url());
}

PoeApiClient::Result<poe::WebStashListWrapper> PoeApiClient::getLegacyStashIndex(
    const QString &account, const QString &realm, const QString &league, std::stop_token token)
{
    static const QString kStashItemsUrl
        = "https://www.pathofexile.com/character-window/get-stash-items";

    QUrlQuery query;
    query.addQueryItem("accountName", account);
    query.addQueryItem("realm", realm);
    query.addQueryItem("league", league);
    query.addQueryItem("tabs", "1");
    query.addQueryItem("tabIndex", "0");

    QUrl url(kStashItemsUrl);
    url.setQuery(query);

    QNetworkRequest request(url);
    // F60: the hand-rolled request this replaces had no transfer timeout, so
    // a stalled connection could hold a gate slot forever.
    request.setTransferTimeout(kPoeApiTimeout);

    // The endpoint label is the bare URL — the query varies per account and
    // must not fragment the hub's endpoint-to-policy topology.
    return ParseInto<poe::WebStashListWrapper>(m_rate_limiter.SubmitFuture(kStashItemsUrl,
                                                                           request,
                                                                           std::move(token)),
                                               &json::readWebStashListWrapper,
                                               kStashItemsUrl,
                                               url);
}
