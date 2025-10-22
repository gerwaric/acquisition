/*
    Copyright (C) 2014-2025 Acquisition Contributors

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

#include "oauthtoken.h"

#include <QNetworkReply>

#include <util/spdlog_qt.h>
#include <util/util.h>

// Hard-code the token refresh lifetime for a public client:
// https://www.pathofexile.com/developer/docs/authorization#clients-public
constexpr long int REFRESH_LIFETIME_DAYS = 7;

OAuthToken::OAuthToken(const QString &json)
{
    Util::parseJson<OAuthToken>(json, *this);
}

OAuthToken::OAuthToken(QNetworkReply *reply)
{
    const QByteArray bytes = reply->readAll();
    reply->deleteLater();

    Util::parseJson<OAuthToken>(bytes, *this);

    // Set birthday and expiration times.
    const QString timestamp = Util::FixTimezone(reply->rawHeader("Date"));
    setBirthday(QDateTime::fromString(timestamp, Qt::RFC2822Date));
}

OAuthToken::OAuthToken(const QVariantMap &tokens)
{
    // clang-format off
    this->access_token  = tokens["access_token"].toString();
    this->expires_in    = tokens["expires_in"].toLongLong();
    this->refresh_token = tokens["refresh_token"].toString();
    this->scope         = tokens["scope"].toString();
    this->sub           = tokens["sub"].toString();
    this->token_type    = tokens["token_type"].toString();
    this->username      = tokens["username"].toString();
    // clang-format on

    setBirthday(QDateTime::currentDateTime());
}

void OAuthToken::setBirthday(const QDateTime &date)
{
    // clang-format off
    this->birthday           = date.toLocalTime();
    this->access_expiration  = this->birthday->addSecs(this->expires_in);
    this->refresh_expiration = this->birthday->addDays(REFRESH_LIFETIME_DAYS);
    // clang-format on
}
