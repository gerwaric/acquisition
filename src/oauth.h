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
#include <QtHttpServer/QHttpServer>
#include <QTimer>

#include <optional>
#include <string>

#include "json_struct/json_struct.h"

class QHttpServerRequest;
class QNetworkAccessManager;
class QNetworkReply;
class QNetworkRequest;

struct OAuthToken {
	static QDateTime getDate(const std::optional<std::string>& timestamp);
	QDateTime getBirthday() const { return getDate(birthday); };
	QDateTime getExpiration() const { return getDate(expiration); };
	std::string access_token;
	long long int expires_in;
	std::string token_type;
	std::string scope;
	std::string username;
	std::string sub;
	std::string refresh_token;
	std::optional<std::string> birthday;
	std::optional<std::string> expiration;
	JS_OBJ(access_token, expires_in, token_type, scope, username, sub, refresh_token, birthday, expiration);
};

class OAuthManager : public QObject {
	Q_OBJECT
public:
	OAuthManager(QNetworkAccessManager& network_manager, QObject* parent = nullptr);
	void setToken(const OAuthToken& token);
	void requestAccess();
	void showStatusMessage();
	const std::optional<OAuthToken> token() const { return token_; };
signals:
	void accessGranted(const OAuthToken& token);
private:
	void createHttpServer();
	void requestAuthorization(const std::string& state, const std::string& code_challenge);
	QString receiveAuthorization(const QHttpServerRequest& request, const std::string& state);
	void requestToken(const std::string& code);
	void receiveToken(QNetworkReply* reply);
	void requestRefresh();
	void setRefreshTimer();

	static QString authorizationError(const QString& message);

	QNetworkAccessManager& network_manager_;

	// I can't find a way to shutdown a QHttpServer once it's started
	// listening, so use a unique pointer so that we can destory the
	// server once authentication is complete, so it won't stay
	// running in the background.
	std::unique_ptr<QHttpServer> http_server_;

	std::optional<OAuthToken> token_;
	std::string code_verifier_;
	std::string redirect_uri_;

	QTimer refresh_timer_;
};
