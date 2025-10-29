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

#pragma once

#include <QDateTime>
#include <QString>
#include <QVariantMap>

class QNetworkReply;

struct OAuthToken
{
    static OAuthToken fromJson(const QString &json);
    static OAuthToken fromReply(QNetworkReply *reply);
    static OAuthToken fromTokens(const QVariantMap &tokens);

    QString access_token;
    long long expires_in{-1};
    QString refresh_token;
    QString scope;
    QString username;
    QString sub;
    QString token_type;

    std::optional<QDateTime> birthday;
    std::optional<QDateTime> access_expiration;
    std::optional<QDateTime> refresh_expiration;

private:
    void setBirthday(const QDateTime& date);

};
