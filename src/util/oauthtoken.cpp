// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2023 Tom Holz

#include "oauthtoken.h"

#include <QNetworkReply>

#include <util/spdlog_qt.h>
#include <util/util.h>

// Hard-code the token refresh lifetime for a public client:
// https://www.pathofexile.com/developer/docs/authorization#clients-public
constexpr long int REFRESH_LIFETIME_DAYS = 7;

OAuthToken OAuthToken::fromJson(const QString &json)
{
    return Util::parseJson<OAuthToken>(json);
}

OAuthToken OAuthToken::fromReply(QNetworkReply *reply)
{
    const QByteArray bytes = reply->readAll();
    reply->deleteLater();

    OAuthToken token = Util::parseJson<OAuthToken>(bytes);

    // Set birthday and expiration times.
    const QString timestamp = Util::FixTimezone(reply->rawHeader("Date"));
    token.setBirthday(QDateTime::fromString(timestamp, Qt::RFC2822Date));
    return token;
}

OAuthToken OAuthToken::fromTokens(const QVariantMap &tokens)
{
    // clang-format off
    OAuthToken token{
        .access_token  = tokens["access_token"].toString(),
        .expires_in    = tokens["expires_in"].toLongLong(),
        .refresh_token = tokens["refresh_token"].toString(),
        .scope         = tokens["scope"].toString(),
        .username      = tokens["username"].toString(),
        .sub           = tokens["sub"].toString(),
        .token_type    = tokens["token_type"].toString(),
    };
    // clang-format on

    token.setBirthday(QDateTime::currentDateTime());
    return token;
}

void OAuthToken::setBirthday(const QDateTime &date)
{
    // clang-format off
    this->birthday           = date.toLocalTime();
    this->access_expiration  = this->birthday->addSecs(this->expires_in);
    this->refresh_expiration = this->birthday->addDays(REFRESH_LIFETIME_DAYS);
    // clang-format on
}
