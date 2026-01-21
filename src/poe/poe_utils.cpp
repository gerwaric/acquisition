// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#include "poe/poe_utils.h"

namespace {

    constexpr const char *kOauthListStashesEndpoint = "List Stashes";
    constexpr const char *kOAuthListStashesUrl = "https://api.pathofexile.com/stash";

    constexpr const char *kOAuthListCharactersEndpoint = "List Characters";
    constexpr const char *kOAuthListCharactersUrl = "https://api.pathofexile.com/character";

    constexpr const char *kOAuthGetStashEndpoint = "Get Stash";
    constexpr const char *kOAuthGetStashUrl = "https://api.pathofexile.com/stash";

    constexpr const char *kOAuthGetCharacterEndpoint = "Get Character";
    constexpr const char *kOAuthGetCharacterUrl = "https://api.pathofexile.com/character";

} // namespace

std::pair<const QString &, QNetworkRequest> poe::MakeStashListRequest(const QString &realm,
                                                                      const QString &league)
{
    static const QString endpoint{kOauthListStashesEndpoint};

    QString url(kOAuthListStashesUrl);
    if (realm != "pc") {
        url += "/" + realm;
    }
    url += "/" + league;
    const QNetworkRequest request{QUrl(url)};

    return {endpoint, request};
}

std::pair<const QString &, QNetworkRequest> poe::MakeCharacterListRequest(const QString &realm)
{
    static const QString endpoint{kOAuthListCharactersEndpoint};

    QString url(kOAuthListCharactersUrl);
    if (realm != "pc") {
        url += "/" + realm;
    }
    const QNetworkRequest request{QUrl(url)};

    return {endpoint, request};
}

std::pair<const QString &, QNetworkRequest> poe::MakeStashRequest(const QString &realm,
                                                                  const QString &league,
                                                                  const QString &stash_id,
                                                                  const QString &substash_id)
{
    static const QString endpoint{kOAuthGetStashEndpoint};

    QString url(kOAuthGetStashUrl);
    if (realm != "pc") {
        url += "/" + realm;
    }
    url += "/" + league;
    url += "/" + stash_id;
    if (!substash_id.isEmpty()) {
        url += "/" + substash_id;
    }
    const QNetworkRequest request{QUrl(url)};

    return {endpoint, request};
}

std::pair<const QString &, QNetworkRequest> poe::MakeCharacterRequest(const QString &realm,
                                                                      const QString &name)
{
    static const QString endpoint{kOAuthGetCharacterEndpoint};

    QString url(kOAuthGetCharacterUrl);
    if (realm != "pc") {
        url += "/" + realm;
    }
    url += "/" + name;
    const QNetworkRequest request{QUrl(url)};

    return {endpoint, request};
}
