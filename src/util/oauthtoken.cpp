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

OAuthToken::OAuthToken(const QString& json)
{
    Util::parseJson<OAuthToken>(json, *this);
}

OAuthToken::OAuthToken(QNetworkReply* reply)
{
    const QByteArray bytes = reply->readAll();
    reply->deleteLater();

    Util::parseJson<OAuthToken>(bytes, *this);

    // Determine birthday and expiration time.
    const QString timestamp = Util::FixTimezone(reply->rawHeader("Date"));
    birthday = QDateTime::fromString(timestamp, Qt::RFC2822Date).toLocalTime();
    access_expiration = birthday->addSecs(expires_in);
    refresh_expiration = birthday->addDays(REFRESH_LIFETIME_DAYS);
}
