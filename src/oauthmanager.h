/*
    Copyright 2023 Gerwaric

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
#include <QTimer>

#include <string>

#include "oauthtoken.h"

class QHttpServer;
class QHttpServerRequest;
class QNetworkAccessManager;
class QNetworkReply;
class QNetworkRequest;
class QTcpServer;

class DataStore;

class OAuthManager : public QObject {
    Q_OBJECT
public:
    explicit OAuthManager(QObject* parent,
        QNetworkAccessManager& network_manager,
        DataStore& datastore);
    ~OAuthManager();
    void setAuthorization(QNetworkRequest& request);
    void RememberToken(bool remember);
    const OAuthToken& token() const { return token_; };
public slots:
    void requestAccess();
    void requestRefresh();
    void showStatus();
signals:
    void accessGranted(const OAuthToken& token);
private:
    void createHttpServer();
    void requestAuthorization(const std::string& state, const std::string& code_challenge);
    QString receiveAuthorization(const QHttpServerRequest& request, const std::string& state);
    void requestToken(const std::string& code);
    void receiveToken(QNetworkReply* reply);
    void setRefreshTimer();

    static QString authorizationError(const QString& message);

    QNetworkAccessManager& network_manager_;
    DataStore& datastore_;

    // I can't find a way to shutdown a QHttpServer once it's started
    // listening, so use a unique pointer so that we can destory the
    // server once authentication is complete, so it won't stay
    // running in the background.
    QHttpServer* http_server_;
    QTcpServer* tcp_server_;

    bool remember_token_;
    OAuthToken token_;
    std::string code_verifier_;
    std::string redirect_uri_;

    QTimer refresh_timer_;

    static const QString SUCCESS_HTML;
    static const QString ERROR_HTML;
};
