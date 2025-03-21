/*
    Copyright (C) 2014-2024 Acquisition Contributors

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

#include "oauthtoken.h"

class QNetworkAccessManager;
class QNetworkRequest;
class QOAuth2AuthorizationCodeFlow;

class OAuthManager : public QObject {
    Q_OBJECT
public:
    explicit OAuthManager(QNetworkAccessManager& network_manager);
    void authorize(QNetworkRequest& request);
    void setToken(const OAuthToken& token);
    OAuthToken token() const { return m_token; };
public slots:
    void requestAccess();
    void refreshAccess();
    void showStatus();
signals:
    void grant(const OAuthToken& token);
private:
    bool activate();
    void fixParameters(QAbstractOAuth::Stage stage, QMultiMap<QString, QVariant>* parameters);

    QOAuth2AuthorizationCodeFlow* m_oauth{ nullptr };
    OAuthToken m_token;
    int m_handler_port{ -1 };

    static void logRequestFailure(const QAbstractOAuth::Error error);
    static void logServerError(const QString& error, const QString& errorDescription, const QUrl& uri);

    static const QString SUCCESS_HTML;
    static const QString ERROR_HTML;
};
