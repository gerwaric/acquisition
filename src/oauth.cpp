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
#include <QMessageBox>
#include <QNetworkAccessManager>
#include <QNetworkReply>
#include <QNetworkRequest>
#include <QString>
#include <QTimer>
#include <QUrl>
#include <QUrlQuery>
#include <QUuid>

#include "QsLog.h"
#include "rapidjson/error/en.h"
#include "rapidjson/stringbuffer.h"
#include "rapidjson/writer.h"

#include "network_info.h"
#include "util.h"

// Hard-code authorization stuff.
const char* AUTHORIZE_URL = "https://www.pathofexile.com/oauth/authorize";
const char* TOKEN_URL = "https://www.pathofexile.com/oauth/token";
const char* CLIENT_ID = "acquisition";
const char* SCOPE = "account:leagues account:stashes account:characters";
const char* REDIRECT_URL = "http://127.0.0.1";
const char* REDIRECT_PATH = "/auth/path-of-exile";

// Refresh a token an hour before it's due to expire.
const int EXPIRATION_BUFFER_SECS = 3600;

// Return this HTML to the browser after successful authentication,
// and try to avoid a favicon request.
const QString SUCCESS_HTML = QString(R"html(
		<html>
			<head>
				<link rel="icon" href="data:, ">
				<title>Acquisition</title>
				<style>
					html, body, .container { height: 75%; }
					.container { display: flex; align-items: center; justify-content: center; }
				</style>
			</head>
			<body>
				<h1 class="container">Acquisition has been authorized.<br>You may close this page.</h1>
			</body>
		</html>")html").simplified();

// Use this as a template to show authentication errors.
const QString ERROR_HTML = QString(R"html(
		<html>
			<head>
				<link rel="icon" href="data:, ">
				<title>OAuth Authorization Error</title>
			</head>
			<body>
				<p>%2</p>
			</body>
		</html>)html").simplified();

OAuthToken::OAuthToken() :
	expires_in_(-1)
{}

OAuthToken::OAuthToken(const std::string& json, const QDateTime& timestamp) :
	expires_in_(-1)
{
	rapidjson::Document doc;
	doc.Parse(json.c_str());
	if (doc.HasParseError()) {
		QLOG_ERROR() << "Error parsing OAuthToken from json:" << rapidjson::GetParseError_En(doc.GetParseError());
		return;
	};
	if (doc.IsObject() == false) {
		QLOG_ERROR() << "OAuthToken json is not an object.";
		return;
	};
	if (doc.HasMember("access_token") && doc["access_token"].IsString()) {
		access_token_ = doc["access_token"].GetString();
	};
	if (doc.HasMember("expires_in") && doc["expires_in"].IsInt64()) {
		expires_in_ = doc["expires_in"].GetInt64();
	};
	if (doc.HasMember("token_type") && doc["token_type"].IsString()) {
		token_type_ = doc["token_type"].GetString();
	};
	if (doc.HasMember("scope") && doc["scope"].IsString()) {
		scope_ = doc["scope"].GetString();
	};
	if (doc.HasMember("username") && doc["username"].IsString()) {
		username_ = doc["username"].GetString();
	};
	if (doc.HasMember("sub") && doc["sub"].IsString()) {
		sub_ = doc["sub"].GetString();
	};
	if (doc.HasMember("refresh_token") && doc["refresh_token"].IsString()) {
		refresh_token_ = doc["refresh_token"].GetString();
	};
	if (doc.HasMember("birthday") && doc["birthday"].IsString()) {
		birthday_ = doc["birthday"].GetString();
	};
	if (doc.HasMember("expiration") && doc["expiration"].IsString()) {
		expiration_ = doc["expiration"].GetString();
	};
	if (timestamp.isValid()) {
		if (birthday_) {
			QLOG_WARN() << "OAuthToken already has a birthday";
		};
		if (expiration_) {
			QLOG_WARN() << "OAuthToken already has an expiration";
		};
		const QDateTime token_expiration = timestamp.addSecs(expires_in_);
		birthday_ = timestamp.toString(Qt::RFC2822Date).toStdString();
		expiration_ = token_expiration.toString(Qt::RFC2822Date).toStdString();
	};
}

std::string OAuthToken::toJson() const {
	rapidjson::Document doc;
	doc.SetObject();
	auto& allocator = doc.GetAllocator();
	Util::RapidjsonAddConstString(&doc, "access_token", access_token_, allocator);
	Util::RapidjsonAddInt64(&doc, "expires_in", expires_in_, allocator);
	Util::RapidjsonAddConstString(&doc, "token_type", token_type_, allocator);
	Util::RapidjsonAddConstString(&doc, "scope", scope_, allocator);
	Util::RapidjsonAddConstString(&doc, "username", username_, allocator);
	Util::RapidjsonAddConstString(&doc, "sub", sub_, allocator);
	Util::RapidjsonAddConstString(&doc, "refresh_token", refresh_token_, allocator);
	if (birthday_) {
		Util::RapidjsonAddConstString(&doc, "birthday", birthday_.value(), allocator);
	};
	if (expiration_) {
		Util::RapidjsonAddConstString(&doc, "expiration", expiration_.value(), allocator);
	};
	return Util::RapidjsonSerialize(doc);
}

QDateTime OAuthToken::getDate(const std::optional<std::string>& timestamp) {
	QByteArray value = QByteArray::fromStdString(timestamp.value_or(""));
	value = Util::FixTimezone(value);
	return QDateTime::fromString(value, Qt::RFC2822Date);
}

//---------------------------------------------------------------------

OAuthManager::OAuthManager(QNetworkAccessManager& network_manager, QObject* parent) :
	QObject(parent),
	network_manager_(network_manager),
	refresh_timer_(this)
{
	// Configure the refresh timer.
	refresh_timer_.setSingleShot(true);
	connect(&refresh_timer_, &QTimer::timeout, this, &OAuthManager::requestRefresh);
}

void OAuthManager::setToken(const OAuthToken& token) {
	if (!token.expiration()) {
		QLOG_ERROR() << "Cannot set an OAuth token without an expiration.";
	} else if (token.getExpiration() < QDateTime::currentDateTime()) {
		QLOG_ERROR() << "Cannot set an OAuth token that has already expired.";
	} else {
		token_ = token;
		emit accessGranted(*token_);
		setRefreshTimer();
	};
}

void OAuthManager::setRefreshTimer() {
	const QString expiration_timestamp = QString::fromStdString(token_->expiration().value_or(""));
	const QDateTime expiration_date = QDateTime::fromString(expiration_timestamp, Qt::RFC2822Date);
	const QDateTime refresh_date = expiration_date.addSecs(-EXPIRATION_BUFFER_SECS);
	const unsigned long interval = QDateTime::currentDateTime().msecsTo(refresh_date);
	refresh_timer_.setInterval(interval);
	refresh_timer_.start();
	QLOG_INFO() << "OAuth: refreshing token at" << refresh_date.toString();
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
	createHttpServer();
	if (http_server_ == nullptr) {
		QLOG_ERROR() << "Unable to create the http server for OAuth authorization.";
		return;
	};

	// Get the port for the callback.
	const auto port = http_server_->listen();
	if (port == 0) {
		QLOG_ERROR() << "Unable to bind the http server for OAuth authorization.";
		return;
	};

	// Build the callback URI.
	QUrl url(REDIRECT_URL);
	url.setPort(port);
	url.setPath(REDIRECT_PATH);
	redirect_uri_ = url.toString().toStdString();

	// Make the authorization request.
	requestAuthorization(state.toStdString(), code_challenge.toStdString());
}

void OAuthManager::createHttpServer() {

	// Create a new HTTP server.
	http_server_ = std::make_unique<QHttpServer>(this);

	// Tell the server to ignore favicon requests, even though these
	// should be disabled based on the HTML we are returning.
	http_server_->route("/favicon.ico",
		[](const QHttpServerRequest& request, QHttpServerResponder&& responder) {
			Q_UNUSED(request);
			Q_UNUSED(responder);
			QLOG_TRACE() << "OAuth: ignoring favicon.ico request";
		});

	// Capture all unhandled requests for debugging.
	http_server_->setMissingHandler(
		[](const QHttpServerRequest& request, QHttpServerResponder&& responder) {
			Q_UNUSED(responder);
			QLOG_TRACE() << "OAuth: unhandled request:" << request.url().toString();
		});
}

void OAuthManager::requestAuthorization(const std::string& state, const std::string& code_challenge)
{
	// Create the authorization query.
	const QUrlQuery query = Util::EncodeQueryItems({
		{"client_id", CLIENT_ID},
		{"response_type", "code"},
		{"scope", SCOPE},
		{"state", QString::fromStdString(state)},
		{"redirect_uri", QString::fromStdString(redirect_uri_)},
		{"code_challenge", QString::fromStdString(code_challenge)},
		{"code_challenge_method", "S256"} });

	// Prepare the url.
	QUrl authorization_url(AUTHORIZE_URL);
	authorization_url.setQuery(query);

	// Make sure the state is passed to the function that receives the authorization response.
	http_server_->route(REDIRECT_PATH,
		[=](const QHttpServerRequest& request) {
			return receiveAuthorization(request, state);
		});

	// Use the user's browser to open the authorization url.
	QDesktopServices::openUrl(authorization_url);
}

QString OAuthManager::authorizationError(const QString& message)
{
	QLOG_ERROR() << "OAuth: authorization error:" << message;
	return ERROR_HTML.arg(message);
}

QString OAuthManager::receiveAuthorization(const QHttpServerRequest& request, const std::string& state)
{
	// Shut the server down now that an access token response has been received.
	// Don't do it immediately in case the browser wants to request a favicon, even
	// though I've tried to disable that by including icon links in HTML.
	QTimer::singleShot(1000, this,
		[=]() {
			http_server_ = nullptr;
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
	if (auth_state != QString::fromStdString(state)) {
		return authorizationError("Invalid authorization repsonse: 'state' is invalid!");
	};

	// Use the code to request an access token.
	requestToken(auth_code.toStdString());

	// Update the user.
	return SUCCESS_HTML;
};

void OAuthManager::requestToken(const std::string& code)
{
	QLOG_TRACE() << "OAuth: requesting access token.";

	QNetworkRequest request;
	request.setUrl(QUrl(TOKEN_URL));
	request.setHeader(QNetworkRequest::ContentTypeHeader, "application/x-www-form-urlencoded");
	request.setHeader(QNetworkRequest::UserAgentHeader, USER_AGENT);

	const QUrlQuery query = Util::EncodeQueryItems({
		{"client_id", CLIENT_ID},
		{"grant_type", "authorization_code"},
		{"code", QString::fromStdString(code)},
		{"redirect_uri", QString::fromStdString(redirect_uri_)},
		{"scope", SCOPE},
		{"code_verifier", QString::fromStdString(code_verifier_)} });
	const QByteArray data = query.toString(QUrl::FullyEncoded).toUtf8();
	QNetworkReply* reply = network_manager_.post(request, data);

	connect(reply, &QNetworkReply::finished, this,
		[=]() {
			receiveToken(reply);
			reply->deleteLater();
		});
	connect(reply, &QNetworkReply::errorOccurred, this,
		[=]() {
			QLOG_ERROR() << "Error requesting OAuth access token:" << reply->errorString();
			reply->deleteLater();
		});
}

void OAuthManager::receiveToken(QNetworkReply* reply)
{
	QLOG_TRACE() << "OAuth: receiving access token.";

	if (reply->error() != QNetworkReply::NoError) {
		QLOG_ERROR() << "OAuth: http error"
			<< reply->attribute(QNetworkRequest::HttpStatusCodeAttribute) << ":"
			<< reply->attribute(QNetworkRequest::HttpReasonPhraseAttribute);
		return;
	};

	// Extract the response so we can dispose of the reply.
	const QByteArray bytes = reply->readAll();
	reply->deleteLater();

	// Determine birthday and expiration time.
	const QString token_timestamp = Util::FixTimezone(reply->rawHeader("Date"));
	const QDateTime token_birthday = QDateTime::fromString(token_timestamp, Qt::RFC2822Date).toLocalTime();

	// Parse the token and emit it.
	token_ = OAuthToken(bytes.toStdString(), token_birthday);
	QLOG_TRACE() << "OAuth access token received.";

	emit accessGranted(*token_);

	// Setup the refresh timer.
	setRefreshTimer();
}

void OAuthManager::requestRefresh() {

	QLOG_TRACE() << "OAuth: refreshing access token.";

	// Update the user.
	static std::unique_ptr<QMessageBox> msgBox = nullptr;
	if (!msgBox) {
		msgBox = std::make_unique<QMessageBox>();
		msgBox->setWindowTitle(APP_NAME " - OAuth Token Refresh");
		msgBox->setModal(false);
	};

	msgBox->setText("Your OAuth token is being refreshed.");

	// Setup the refresh query.
	const QUrlQuery query = Util::EncodeQueryItems({
		{"client_id", CLIENT_ID},
		{"grant_type", "refresh_token"},
		{"refresh_token", QString::fromStdString(token_->refresh_token())} });
	const QByteArray data = query.toString(QUrl::FullyEncoded).toUtf8();

	// Make and submit the POST request.
	QNetworkRequest request;
	request.setUrl(QUrl(TOKEN_URL));
	request.setHeader(QNetworkRequest::ContentTypeHeader, "application/x-www-form-urlencoded");
	request.setHeader(QNetworkRequest::UserAgentHeader, USER_AGENT);
	QNetworkReply* reply = network_manager_.post(request, data);

	connect(reply, &QNetworkReply::finished, this,
		[=]() {
			// Update the user again after the token has been received.
			receiveToken(reply);
			const QStringList message = {
				"Your OAuth token was refreshed on " + token_->getBirthday().toString(),
				"",
				"The new token expires on " + token_->getExpiration().toString(),
			};
			msgBox->setText(message.join("\n"));
			msgBox->show();
			msgBox->raise();
			reply->deleteLater();
		});

	connect(reply, &QNetworkReply::errorOccurred, this,
		[=]() {
			// Let the user know if there was an error.
			const int status = reply->attribute(QNetworkRequest::HttpStatusCodeAttribute).toInt();
			const auto reason = reply->attribute(QNetworkRequest::HttpReasonPhraseAttribute).toString();
			const QStringList message = {
				"OAuth refresh failed: " + reply->errorString(),
				"",
				"HTTP status " + QString::number(status) + " (" + reason + ")"
			};
			msgBox->setText(message.join("\n"));
			msgBox->show();
			msgBox->raise();
			reply->deleteLater();
		});
}

void OAuthManager::showStatus() {

	static std::unique_ptr<QMessageBox> msgBox = nullptr;
	if (!msgBox) {
		msgBox = std::make_unique<QMessageBox>();
		msgBox->setWindowTitle("OAuth Status - " APP_NAME " - OAuth Token Status");
		msgBox->setModal(false);
	};

	if (token_) {
		const std::string json = token_.value().toJson();
		const QDateTime now = QDateTime::currentDateTime();
		const QDateTime refresh_time = now.addMSecs(refresh_timer_.remainingTime());
		const QString refresh_timestamp = refresh_time.toString("MMM d 'at' h:m ap");
		const QStringList message = {
			"Your current OAuth token:",
			"",
			QString::fromStdString(json),
			"",
			"This token will be automatically refreshed on " + refresh_timestamp
		};
		msgBox->setText(message.join("\n"));
	} else {
		msgBox->setText("No valid token. You are not authenticated.");
	}
	msgBox->show();
	msgBox->raise();
}
