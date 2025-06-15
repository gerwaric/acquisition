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

#include <glaze/glaze.hpp>

#include <util/spdlog_qt.h>
#include <util/util.h>

/*

OAuthToken::OAuthToken(const QString& json)
{
    const char* data = reinterpret_cast<const char*>(json.toUtf8().constData());
    const size_t size = json.toUtf8().size();
    const std::string_view json_view(data, size);
    auto result = glz::read_json(*this, json_view);
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

*/
