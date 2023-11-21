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

#include "network_info.h"
#include "util.h"

static const char* AUTHORIZE_URL = "https://www.pathofexile.com/oauth/authorize";
static const char* TOKEN_URL = "https://www.pathofexile.com/oauth/token";
static const char* CLIENT_ID = "acquisition";
static const char* SCOPE = "account:leagues account:stashes account:characters";
static const char* REDIRECT_URL = "http://127.0.0.1";
static const char* REDIRECT_PATH = "/auth/path-of-exile";

// Refresh a token an hour before it's due to expire.
static const int EXPIRATION_BUFFER_SECS = 3600;

QDateTime OAuthToken::getDate(const std::optional<std::string>& timestamp) {
	if (timestamp) {
		return QDateTime::fromString(QString::fromStdString(timestamp.value()), Qt::RFC2822Date);
	} else {
		return QDateTime();
	};
}

OAuthManager::OAuthManager(QNetworkAccessManager& network_manager, QObject* parent) :
	QObject(parent),
	the_manager_(network_manager)
{
	// Configure the refresh timer.
	refresh_timer_.setSingleShot(true);
	connect(&refresh_timer_, &QTimer::timeout, this, &OAuthManager::requestRefresh);

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

void OAuthManager::setToken(const OAuthToken& token) {
	if (!token.expiration) {
		QLOG_ERROR() << "Cannot set a token without an expiration.";
		return;
	};
	the_token_ = token;
	emit accessGranted(*the_token_);
	setRefreshTimer();
}

void OAuthManager::setRefreshTimer() {
	const QString expiration_timestamp = QString::fromStdString(*the_token_->expiration);
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
	the_server_->listen();
	const auto port = the_server_->serverPorts().at(0);

	// Build the callback URI.
	QUrl url(REDIRECT_URL);
	url.setPort(port);
	url.setPath(REDIRECT_PATH);
	redirect_uri_ = url.toString().toStdString();

	// Make the authorization request.
	requestAuthorization(state.toStdString(), code_challenge.toStdString());
}

void OAuthManager::requestAuthorization(const std::string& state, const std::string& code_challenge)
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
		[=](const QHttpServerRequest& request) {
			return QString::fromStdString(receiveAuthorization(request, state));
		});

	// Use the user's browser to open the authorization url.
	QDesktopServices::openUrl(authorization_url);
}

std::string OAuthManager::authorizationError(const std::string& message)
{
	QLOG_ERROR() << "OAuth: authorization error:" << message;
	QString html =
		"<html>"
		"  <head>"
		"    <title>OAuth Authorization Error</title>"
		"  </head>"
		"  <body>"
		"    <p>%1</p>"
		"  </body>"
		"</html>";
	return html.arg(QString::fromStdString(message)).toStdString();
}

std::string OAuthManager::receiveAuthorization(const QHttpServerRequest& request, const std::string& state)
{
	// Shut the server down now that an access token response has been received.
	// Don't do it immediately in case the browser wants to request a favicon.
	QTimer::singleShot(1000, this,
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
		return authorizationError(error_message.toStdString());
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
	return R"html(
		<html>
			<head>
				<title>Acquisition</title>
				<style>
					html, body, .container { height: 75%; }
					.container { display: flex; align-items: center; justify-content: center; }
				</style>
			</head>
			<body>
				<h1 class="container">Acquisition has been authorized.<br>You may close this page.</h1>
			</body>
		</html>";
	)html";
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
		{"code", code},
		{"redirect_uri", redirect_uri_},
		{"scope", SCOPE},
		{"code_verifier", code_verifier_} });
	const QByteArray data = query.toString(QUrl::FullyEncoded).toUtf8();

	QNetworkReply* reply = the_manager_.post(request, data);
	connect(reply, &QNetworkReply::finished, this, [=]() { receiveToken(reply); });
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
	JS::ParseContext context(bytes);
	JS::Error error = context.parseTo(the_token_);
	if (error != JS::Error::NoError) {
		QLOG_ERROR() << "OAuth: error parsing token:" << context.makeErrorString();
		return;
	};
	QLOG_TRACE() << "OAuth access token received.";

	// Determine birthday and expiration time.
	const QString token_timestamp = Util::FixTimezone(reply->rawHeader("Date"));
	const QDateTime token_birthday = QDateTime::fromString(token_timestamp, Qt::RFC2822Date);
	const QDateTime token_expiration = token_birthday.addSecs(the_token_->expires_in);

	// Add birthday and expiration then notify listeners there's a new token.
	the_token_->birthday = token_birthday.toString(Qt::RFC2822Date).toStdString();
	the_token_->expiration = token_expiration.toString(Qt::RFC2822Date).toStdString();
	emit accessGranted(*the_token_);

	// Setup the refresh timer.
	setRefreshTimer();
}

void OAuthManager::requestRefresh() {

	QLOG_TRACE() << "OAuth: refreshing access token.";

	QMessageBox* msgBox = new QMessageBox;
	msgBox->setText(
		"Your OAuth token is being refreshed.");
		//"This dialog box will close automatically after 5 minutes.");
	msgBox->exec();
	connect(msgBox, &QMessageBox::close, msgBox, &QMessageBox::deleteLater);
	//QTimer::singleShot(1000 * 60 * 5, [=]() { if (msgBox) msgBox->deleteLater(); });

	QNetworkRequest request;
	request.setUrl(QUrl(TOKEN_URL));
	request.setHeader(QNetworkRequest::ContentTypeHeader, "application/x-www-form-urlencoded");
	request.setHeader(QNetworkRequest::UserAgentHeader, USER_AGENT);

	const QUrlQuery query = Util::EncodeQueryItems({
		{"client_id", CLIENT_ID},
		{"grant_type", "refresh_token"},
		{"refresh_token", the_token_->refresh_token} });
	const QByteArray data = query.toString(QUrl::FullyEncoded).toUtf8();

	QNetworkReply* reply = the_manager_.post(request, data);
	connect(reply, &QNetworkReply::finished, this, [=]() { receiveToken(reply); });
}
