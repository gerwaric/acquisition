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
#include <QAbstractOAuth>
#include <QString>
#include <QTimer>
#include <QUrl>
#include <QVariantMap>

#include "oauthtoken.h"

class QOAuth2AuthorizationCodeFlow;
class QOAuthHttpServerReplyHandler;

class DataStore;
class NetworkManager;

class OAuthManager : public QObject
{
    Q_OBJECT

public:
    explicit OAuthManager(NetworkManager& manager, DataStore &datastore, QObject *parent = nullptr);
    void initLogin();

private slots:
    void onRequestFailure(const QAbstractOAuth::Error error);
    void onServerError(const QString &error, const QString &errorDescription, const QUrl &uri);
    void receiveToken(const QVariantMap &tokens);
    void receiveGrant();

signals:
    void grantAccess(const OAuthToken &token);
    void isAuthenticatedChanged();

private:
    void setToken(const OAuthToken &token);
    NetworkManager &m_network_manager;
    DataStore &m_data;

    QOAuth2AuthorizationCodeFlow* m_oauth;
    QOAuthHttpServerReplyHandler* m_handler;

    void initOAuth();

    bool m_authenticated{false};
    OAuthToken m_token;
};
