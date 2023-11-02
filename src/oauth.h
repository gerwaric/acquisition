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

#ifndef ACQUISITION_OAUTH_H
#define ACQUISITION_OAUTH_H

#include <QObject>
#include <QString>

#include "rapidjson/document.h"

class QHttpServerRequest;
class QHttpServer;
class QNetworkAccessManager;
class QNetworkReply;
class QNetworkRequest;

struct AccessToken {
	AccessToken(const rapidjson::Value& json);
	QString access_token;
	long int expires_in;
	QString token_type;
	QString scope;
	QString username;
	QString sub;
	QString refresh_token;
};

class OAuthManager : public QObject {
	Q_OBJECT
public:
	OAuthManager(QNetworkAccessManager& network_manager, QObject* parent = nullptr);
	void requestAccess();
    const QString access_token() const;
	void addAuthorization(QNetworkRequest& request);
signals:
	void accessGranted(const AccessToken& token);
private:
	void requestAuthorization(const QByteArray& state, const QByteArray& code_challenge);
	QString receiveAuthorization(const QHttpServerRequest& request, const QByteArray& state);
	void requestToken(const QString& code);
	void receiveToken(QNetworkReply* reply);
	void requestRefresh(const QString& refresh_token);

	QNetworkAccessManager& the_manager_;
	std::unique_ptr<QHttpServer> the_server_;
	std::unique_ptr<AccessToken> the_token_;

	QByteArray code_verifier_;
	QString redirect_uri_;
};

#endif
