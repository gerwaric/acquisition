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

    QString username;
    QString scope;
    QString sub;
    QString token_type;
    QByteArray access_token;
    QByteArray refresh_token;
    long long expires_in{-1};

    std::optional<QDateTime> birthday;
    std::optional<QDateTime> access_expiration;
    std::optional<QDateTime> refresh_expiration;

private:
    void setBirthday(const QDateTime& date);

};
