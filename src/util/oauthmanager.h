// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2023 Tom Holz

#pragma once

#include <QObject>
#include <QAbstractOAuth>
#include <QString>
#include <QTimer>
#include <QUrl>
#include <QVariantMap>

#include "util/oauthtoken.h"

class QOAuth2AuthorizationCodeFlow;
class QOAuthHttpServerReplyHandler;

class NetworkManager;

class OAuthManager : public QObject
{
    Q_OBJECT

public:
    explicit OAuthManager(NetworkManager &manager);
    void initLogin();

public slots:
    void setToken(const OAuthToken &token);

    void onRequestFailure(const QAbstractOAuth::Error error);
    void onServerError(const QString &error, const QString &errorDescription, const QUrl &uri);
    void onOAuthError(const QString &error, const QString &errorDescription, const QUrl &uri);
    void receiveToken(const QVariantMap &tokens);
    void receiveGrant();

signals:
    void grantAccess(const OAuthToken &token);
    void isAuthenticatedChanged();

private:

    QOAuth2AuthorizationCodeFlow* m_oauth;
    QOAuthHttpServerReplyHandler* m_handler;

    void initOAuth();

    bool m_authenticated{false};
    OAuthToken m_token;
};
