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

#include "oauth.h"

#include <QByteArray>
#include <QCryptographicHash>
#include <QDesktopServices>
#include <QtHttpServer/QHttpServer>
#include <QNetworkAccessManager>
#include <QNetworkReply>
#include <QNetworkRequest>
#include <QString>
#include <QTimer>
#include <QUrl>
#include <QUrlQuery>
#include <QUuid>

#include "QsLog.h"

#include "rapidjson/document.h"
#include "network_info.h"
#include "util.h"

static const QString AUTHORIZE_URL = "https://www.pathofexile.com/oauth/authorize";
static const QString TOKEN_URL = "https://www.pathofexile.com/oauth/token";
static const QString CLIENT_ID = "acquisition";
static const QString SCOPE = "account:leagues account:stashes account:characters";
static const QString REDIRECT_URL = "http://127.0.0.1";
static const QString REDIRECT_PATH = "/auth/path-of-exile";

OAuthManager::OAuthManager(QNetworkAccessManager& network_manager, QObject* parent) :
	QObject(parent),
	the_manager_(network_manager)
{
	// Create the http server.
	the_server_ = std::make_unique<QHttpServer>(this);

	// Tell the server to ignore favicon requests.
	the_server_->route("/favicon.ico",
		[](const QHttpServerRequest& request, QHttpServerResponder&& responder) {
			Q_UNUSED(request);
			Q_UNUSED(responder);
			QLOG_TRACE() << "OAuth: ignoring favicon.ico request";
		});

	// Capture all unhandled requests for debugging.
	the_server_->setMissingHandler(
		[](const QHttpServerRequest& request, QHttpServerResponder&& responder) {
			Q_UNUSED(responder);
			QLOG_TRACE() << "OAuth: unhandled request:" << request.url().toString();
		});
}

void OAuthManager::requestAccess()
{
	// Build the state.
	const auto state_data = (
		QUuid::createUuid().toString(QUuid::WithoutBraces) +
		QUuid::createUuid().toString(QUuid::WithoutBraces)).toLatin1(); // 43 <= length <= 128
	const auto state_hash = QCryptographicHash::hash(state_data, QCryptographicHash::Sha256);
	const auto state = state_hash.toBase64(QByteArray::Base64UrlEncoding | QByteArray::OmitTrailingEquals);

	// Create the code challenge.
	code_verifier_ = (
		QUuid::createUuid().toString(QUuid::WithoutBraces) +
		QUuid::createUuid().toString(QUuid::WithoutBraces)).toLatin1(); // 43 <= length <= 128
	const auto code_hash = QCryptographicHash::hash(code_verifier_, QCryptographicHash::Sha256);
	const auto code_challenge = code_hash.toBase64(QByteArray::Base64UrlEncoding | QByteArray::OmitTrailingEquals);

	// Setup an http server so we know what port to listen on.
	the_server_->listen();
	const auto port = the_server_->serverPorts().at(0);

	// Build the callback URI.
	QUrl url(REDIRECT_URL);
	url.setPort(port);
	url.setPath(REDIRECT_PATH);
	redirect_uri_ = url.toString();
	
	// Make the authorization request.
	requestAuthorization(state, code_challenge);
}

const QString OAuthManager::access_token() const {
	return (the_token_ == nullptr) ? "" : the_token_->access_token;
};

void OAuthManager::addAuthorization(QNetworkRequest& request) {
	if (the_token_ != nullptr) {
		QString bearer = "Bearer " + the_token_->access_token;
		request.setRawHeader("Authorization", bearer.toUtf8());
	};
}

void OAuthManager::requestAuthorization(const QByteArray& state, const QByteArray& code_challenge)
{
	// Create the authorization query.
	const QUrlQuery query = Util::EncodeQueryItems({
		{"client_id", CLIENT_ID},
		{"response_type", "code"},
		{"scope", SCOPE},
		{"state", state},
		{"redirect_uri", redirect_uri_},
		{"code_challenge", code_challenge},
		{"code_challenge_method", "S256"} });

	// Prepare the url.
	QUrl authorization_url(AUTHORIZE_URL);
	authorization_url.setQuery(query);

	// Make sure the state is passed to the function that receives the authorization response.
	the_server_->route(REDIRECT_PATH,
		[=](const QHttpServerRequest& request) { return receiveAuthorization(request, state); });

	// Use the user's browser to open the authorization url.
	QLOG_DEBUG() << "Preparing OAuth authorization request:" << authorization_url.toString();
	QDesktopServices::openUrl(authorization_url);
}

QString authorizationError(const QString& message)
{
	QLOG_ERROR() << "OAuth authorization error:" << message;
	QString html =
		"<html>"
		"  <head>"
		"    <title>OAuth Authorization Error</title>"
		"  </head>"
		"  <body>"
		"    <p>%1</p>"
		"  </body>"
		"</html>";
	return html.arg(message);
}

QString OAuthManager::receiveAuthorization(const QHttpServerRequest& request, const QByteArray& state)
{
	// Shut the server down now that an access token response has been received.
	// Don't do it immediately in case the browser wants to request a favicon.
	QTimer::singleShot(1000,
		[=]() {
			the_server_->disconnect();
			the_server_->deleteLater();
			the_server_ = nullptr;
		});

	const QUrlQuery& query = request.query();

	// Check for errors.
	if (query.hasQueryItem("error")) {
		QString error_message = query.queryItemValue("error");
		const QString error_desription = query.queryItemValue("error_description");
		const QString error_uri = query.queryItemValue("error_uri");
		if (error_desription.isEmpty() == false) {
			error_message += " : " + error_desription;
		};
		if (error_uri.isEmpty() == false) {
			error_message += " : " + error_uri;
		};
		return authorizationError(error_message);
	};

	const QString auth_code = query.queryItemValue("code");
	const QString auth_state = query.queryItemValue("state");

	// Make sure the code and state look valid.
	if (auth_code.isEmpty()) {
		return authorizationError("Invalid authorization response: 'code' is missing.");
	};
	if (auth_state.isEmpty()) {
		return authorizationError("Invalid authorization response: 'state' is missing.");
	};
	if (auth_state != state) {
		return authorizationError("Invalid authorization repsonse: 'state' is invalid!");
	};

	// Use the code to request an access token.
	requestToken(auth_code);

	// Update the user.
	return
		"<html>"
		"  <head>"
		"    <title>OAuth Authorization Response</title>"
		"  </head>"
		"  <body>"
		"    <p>Authorization received; you can close this page.</p>"
		"  </body>"
		"</html>";
};

void OAuthManager::requestToken(const QString& code)
{
	QNetworkRequest request(TOKEN_URL);
	request.setHeader(QNetworkRequest::ContentTypeHeader, "application/x-www-form-urlencoded");
	request.setHeader(QNetworkRequest::UserAgentHeader, USER_AGENT);

	const QUrlQuery query = Util::EncodeQueryItems({
		{"client_id", CLIENT_ID},
		{"grant_type", "authorization_code"},
		{"code", code},
		{"redirect_uri", redirect_uri_},
		{"scope", SCOPE},
		{"code_verifier", code_verifier_} });
	
	const QByteArray data = query.toString(QUrl::FullyEncoded).toUtf8();

	QLOG_TRACE() << "Requesting OAuth access token.";
	QNetworkReply* reply = the_manager_.post(request, data);
	connect(reply, &QNetworkReply::finished, this, [=]() { receiveToken(reply); });
}

void OAuthManager::receiveToken(QNetworkReply* reply)
{
	// Extract the response so we can dispose of the reply.
	const QByteArray bytes = reply->readAll();
	reply->deleteLater();

	// Parse the response into a token.
	rapidjson::Document json;
	json.Parse(bytes.constData());
	if (json.HasParseError()) {
		QLOG_ERROR() << "Oauth: error parsing access token response:" << Util::RapidjsonSerialize(json);
		return;
	};

	// Check for response errors.
	if (json.HasMember("error")) {
		QLOG_ERROR() << "OAuth: access token response error:" << Util::RapidjsonSerialize(json);
		return;
	};

	// Parse the token and emit it.
	QLOG_TRACE() << "OAuth access token received.";
	the_token_ = std::make_unique<AccessToken>(json);
	emit accessGranted(*the_token_);

	// Use a timer to trigger a refresh.
	const long int refresh_sec = (the_token_->expires_in - 60);
	if (refresh_sec < 60) {
		QLOG_ERROR() << "Token refresh is too soon:" << refresh_sec << "s";
		return;
	};
	QLOG_TRACE() << "Refreshing OAuth token in" << refresh_sec << "seconds.";
	const long int refresh_msec = refresh_sec * 1000;
	QTimer::singleShot(refresh_msec, [=]() { requestRefresh(the_token_->refresh_token); });
}

void OAuthManager::requestRefresh(const QString& refresh_token) {

	QNetworkRequest request(TOKEN_URL);
	request.setHeader(QNetworkRequest::ContentTypeHeader, "application/x-www-form-urlencoded");
	request.setHeader(QNetworkRequest::UserAgentHeader, USER_AGENT);

	const QUrlQuery query = Util::EncodeQueryItems({
		{"client_id", CLIENT_ID},
		{"grant_type", "refresh_token"},
		{"refresh_token", refresh_token} });

	QLOG_TRACE() << "Refreshing OAuth access token.";
	const QByteArray data = query.toString(QUrl::FullyEncoded).toUtf8();
	QNetworkReply* reply = the_manager_.post(request, data);
	connect(reply, &QNetworkReply::finished, this, [=]() { receiveToken(reply); });
}

QString getString(const rapidjson::Value& json, const char* name)
{
	return (json.HasMember(name) && json[name].IsString()) ? json[name].GetString() : "";
}

int getInt(const rapidjson::Value& json, const char* name)
{
	return (json.HasMember(name) && json[name].IsInt()) ? json[name].GetInt() : 0;
}

AccessToken::AccessToken(const rapidjson::Value& json)
{
	access_token = getString(json, "access_token");
	expires_in = getInt(json, "expires_in");
	token_type = getString(json, "token_type");
	scope = getString(json, "scope");
	username = getString(json, "username");
	sub = getString(json, "sub");
	refresh_token = getString(json, "refresh_token");
}
