// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2023 Tom Holz

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
