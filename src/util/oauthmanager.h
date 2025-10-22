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

#include <QObject>
#include <QString>
#include <QTimer>
#include <QVariantMap>

#include "oauthtoken.h"

class QOAuth2AuthorizationCodeFlow;
class QOAuthHttpServerReplyHandler;

class NetworkManager;

class OAuthManager : public QObject
{
    Q_OBJECT

public:
    explicit OAuthManager(NetworkManager &manager, QObject *parent = nullptr);
    void setToken(const OAuthToken &token);
    void initLogin();

private slots:
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
