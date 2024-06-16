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

#include "rapidjson/document.h"

class QHttpServerRequest;
class QNetworkAccessManager;
class QNetworkReply;
class QNetworkRequest;

class Application;

class OAuthToken {
public:
	OAuthToken();
	OAuthToken(const std::string& json, const QDateTime& timestamp = QDateTime());
	std::string access_token() const { return access_token_; };
	int expires_in() const { return expires_in_; };
	std::string scope() const { return scope_; };
	std::string username() const { return username_; };
	std::string sub() const { return sub_; };
	std::string refresh_token() const { return refresh_token_; };
	std::optional<std::string> birthday() const { return birthday_; };
	std::optional<std::string> expiration() const { return expiration_; };
	std::string toJson() const;
	QDateTime getBirthday() const { return getDate(birthday_); };
	QDateTime getExpiration() const { return getDate(expiration_); };
private:
	static QDateTime getDate(const std::optional<std::string>& timestamp);
	std::string access_token_;
	long long int expires_in_;
	std::string token_type_;
	std::string scope_;
	std::string username_;
	std::string sub_;
	std::string refresh_token_;
	std::optional<std::string> birthday_;
	std::optional<std::string> expiration_;
};

class OAuthManager : public QObject {
	Q_OBJECT
public:
	OAuthManager(QObject* parent, QNetworkAccessManager& network_manager);
	void setAuthorization(QNetworkRequest& request);
	void setToken(const OAuthToken& token);
	const std::optional<OAuthToken> token() const { return token_; };
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
