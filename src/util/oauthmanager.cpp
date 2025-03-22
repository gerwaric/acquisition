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

#include "oauthmanager.h"

#include <QByteArray>
#include <QDesktopServices>
#include <QMessageBox>
#include <QNetworkAccessManager>
#include <QNetworkRequest>
#include <QOAuth2AuthorizationCodeFlow>
#include <QOAuthHttpServerReplyHandler>
#include <QSettings>
#include <QUrl>

#include <QsLog/QsLog.h>

#include "datastore/datastore.h"

#include "network_info.h"
#include "util.h"

namespace {
    constexpr const char* AUTHORIZE_URL = "https://www.pathofexile.com/oauth/authorize";
    constexpr const char* TOKEN_URL = "https://www.pathofexile.com/oauth/token";
    constexpr const char* CLIENT_ID = "acquisition";
    constexpr const char* SCOPE = "account:leagues account:stashes account:characters";
    constexpr const char* REDIRECT_URL = "http://127.0.0.1";
    constexpr const char* REDIRECT_PATH = "/auth/path-of-exile";
}

OAuthManager::OAuthManager(QNetworkAccessManager& network_manager) {
    QLOG_DEBUG() << "OAuth: setting up OAuth";

    m_oauth = new QOAuth2AuthorizationCodeFlow(this);
    m_oauth->setNetworkAccessManager(&network_manager);
    m_oauth->setAccessTokenUrl(QUrl(TOKEN_URL));
    m_oauth->setAuthorizationUrl(QUrl(AUTHORIZE_URL));
    m_oauth->setClientIdentifier(CLIENT_ID);
    m_oauth->setScope(SCOPE);
    m_oauth->setPkceMethod(QOAuth2AuthorizationCodeFlow::PkceMethod::S256);
    m_oauth->setNetworkRequestModifier(this,
        [](QNetworkRequest& request, QAbstractOAuth::Stage) {
            request.setHeader(QNetworkRequest::UserAgentHeader, USER_AGENT);
        });
    m_oauth->setModifyParametersFunction(
        [this](QAbstractOAuth::Stage stage, QMultiMap<QString, QVariant>* parameters) {
            fixParameters(stage, parameters);
        });
    m_oauth->setAutoRefresh(true);

    connect(m_oauth, &QAbstractOAuth::authorizeWithBrowser, this, &QDesktopServices::openUrl);
    connect(m_oauth, &QAbstractOAuth::granted, this, [this]() { emit grant(m_token); });
    connect(m_oauth, &QAbstractOAuth::requestFailed, this, &OAuthManager::logRequestFailure);
    connect(m_oauth, &QAbstractOAuth2::serverReportedErrorOccurred, this, &OAuthManager::logServerError);
}

void OAuthManager::authorize(QNetworkRequest& request) {
    QLOG_TRACE() << "OAuth: authorizing a network request:" << request.url().toDisplayString();
    if (m_token.access_token.isEmpty()) {
        QLOG_ERROR() << "OAuth: cannot authorize request because there is no token.";
        return;
    };
    const QByteArray bearer_token = "Bearer " + m_token.access_token;
    request.setRawHeader("Authorization", bearer_token);
}

void OAuthManager::setToken(const OAuthToken& token) {
    QLOG_DEBUG() << "OAuth: setting token";
    m_token = token;
    m_oauth->setRefreshToken(m_token.refresh_token);
    if (m_token.access_expiration > QDateTime::currentDateTime()) {
        QLOG_WARN() << "OAuth: the access token needs to be refreshed";
    };
}

void OAuthManager::requestAccess() {
    QLOG_DEBUG() << "OAuth: requesting an access token";
    if (activate()) {
        m_oauth->grant();
    };
}

void OAuthManager::refreshAccess() {
    QLOG_DEBUG() << "OAuth: refreshing access token";
    if (m_token.refresh_token.isEmpty()) {
        QLOG_ERROR() << "OAuth: cannot refresh access because the refresh token is empty";
        return;
    };
    if (activate()) {
        m_oauth->refreshTokens();
    };
};

bool OAuthManager::activate() {

    if (m_handler_port > -1) {
        QLOG_WARN() << "OAuth: handler port is non-negative:" << m_handler_port;
    };

    auto handler = new QOAuthHttpServerReplyHandler(0, this);
    handler->setCallbackPath(REDIRECT_PATH);
    handler->setCallbackText(SUCCESS_HTML);

    connect(handler, &QOAuthHttpServerReplyHandler::tokensReceived, this,
        [=](const QVariantMap& tokens) {
            QLOG_DEBUG() << "OAuth: the handler has received tokens";
            m_token = OAuthToken(tokens);
            handler->close();
            handler->deleteLater();
            m_oauth->setReplyHandler(nullptr);
            m_handler_port = -1;
        });

    connect(handler, &QAbstractOAuthReplyHandler::tokenRequestErrorOccurred,
        [=](QAbstractOAuth::Error error, const QString& errorString) {
            QLOG_ERROR() << "OAuth: token request error" << QString::number(int(error)) + ":" << errorString;
            handler->close();
            handler->deleteLater();
            m_oauth->setReplyHandler(nullptr);
            m_handler_port = -1;
        });

    if (!handler->isListening()) {
        QLOG_ERROR() << "OAuth: unable to start the reply handler";
        handler->deleteLater();
        m_handler_port = -1;
        return false;
    };

    m_oauth->setReplyHandler(handler);
    m_handler_port = handler->port();
    QLOG_DEBUG() << "OAuth: reply handler is listening on port" << m_handler_port;
    return true;
}

void OAuthManager::fixParameters(QAbstractOAuth::Stage stage, QMultiMap<QString, QVariant>* parameters) {
    if (stage == QAbstractOAuth::Stage::RequestingAuthorization) {
        // Qt uses "loopback" but GGG requires that acquisition use "127.0.0.1"
        QUrl url(REDIRECT_URL);
        url.setPort(m_handler_port);
        url.setPath(REDIRECT_PATH);
        QLOG_TRACE() << "OAuth: replacing redirect_uri with" << url.toDisplayString();
        parameters->replace("redirect_uri", url);
    }
    else if (stage == QAbstractOAuth::Stage::RefreshingAccessToken) {
        // Qt inserts a client secret, but GGG's oauth implementation
        // doesn't like this.
        if (parameters->contains("client_secret")) {
            QLOG_TRACE() << "OAuth: removing client secret from refresh parameters";
            parameters->remove("client_secret");
        };
    };
    if (QsLogging::Logger::instance().loggingLevel() >= QsLogging::DebugLevel) {
        for (const auto& key : parameters->keys()) {
            QLOG_TRACE() << "OAuth: stage" << int(stage) << "parameter:" << key << "=" << parameters->value(key);
        };
    };
}

void OAuthManager::logRequestFailure(const QAbstractOAuth::Error error) {
    QString message;
    switch (error) {
    case QAbstractOAuth::Error::NoError:
        message = "No error has ocurred.";
        break;
    case QAbstractOAuth::Error::NetworkError:
        message = "Failed to connect to the server.";
        break;
    case QAbstractOAuth::Error::ServerError:
        message = "The server answered the request with an error, or its response was not successfully received(for example, due to a state mismatch).";
        break;
    case QAbstractOAuth::Error::OAuthTokenNotFoundError:
        message = "The server's response to a token request provided no token identifier.";
        break;
    case QAbstractOAuth::Error::OAuthTokenSecretNotFoundError:
        message = "The server's response to a token request provided no token secret.";
        break;
    case QAbstractOAuth::Error::OAuthCallbackNotVerified:
        message = "The authorization server has not verified the supplied callback URI in the request.This usually happens when the provided callback does not match with the callback supplied during client registration.";
        break;
    case QAbstractOAuth::Error::ClientError:
        message = "An error that is attributable to the client application(e.g.missing configuration or attempting a request in a state where it's not allowed). Currently used by QOAuth2DeviceAuthorizationFlow.";
        break;
    case QAbstractOAuth::Error::ExpiredError:
        message = "A token has expired.Currently used by QOAuth2DeviceAuthorizationFlow.";
        break;
    default:
        message = "An unknown OAuth error occured (" + QString::number(int(error)) + ")";
        break;
    };
    QLOG_ERROR() << "OAuth: request failed:" << message;
}

void OAuthManager::logServerError(const QString& error, const QString& errorDescription, const QUrl& uri) {
    const QString message = QString("%1: %2 (%3)").arg(error, errorDescription, uri.toDisplayString());
    QLOG_ERROR() << "OAuth: server error:" << message;
}

void OAuthManager::showStatus() {

    QMessageBox* msgBox = new QMessageBox;
    msgBox->setWindowTitle("OAuth Status - " APP_NAME " - OAuth Token Status");
    msgBox->setModal(false);
    msgBox->setAttribute(Qt::WA_DeleteOnClose);

    const QDateTime now = QDateTime::currentDateTime();
    const QString json = QString::fromUtf8(JS::serializeStruct(m_token, JS::SerializerOptions::Pretty));
    QStringList message = { "Your current OAuth token:", json };

    if (!m_token.access_expiration) {
        message += "The token's access_expiration is missing";
    };
    if (!m_token.refresh_expiration) {
        message += "The token's refresh_expiration is missing";
    };

    if (m_token.access_expiration && m_token.refresh_expiration) {
        if (now < m_token.access_expiration.value_or(QDateTime())) {
            //const QDateTime refresh_time = now.addMSecs(m_refresh_timer.remainingTime());
            //const QString refresh_timestamp = refresh_time.toString("MMM d 'at' h:m ap");
            //message.append("This token will be automatically refreshed on " + refresh_timestamp);
        }
        else if (now < m_token.refresh_expiration) {
            message.append("This token needs to be refreshed now");
        }
        else {
            message.append("No valid token. You are not authenticated.");
        };
    }
    else {
        message.append("The token's expiration dates are missing");
    };
    QLOG_TRACE() << "OAuth: status:" << message.join("; ");
    msgBox->setText(message.join("\n\n"));
    msgBox->show();
    msgBox->raise();
}

// Return this HTML to the browser after successful authentication,
// and try to avoid a favicon request.
const QString OAuthManager::SUCCESS_HTML = QString(
    R"html(
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
		</html>"
	)html").simplified();

// Use this as a template to show authentication errors.
const QString OAuthManager::ERROR_HTML = QString(
    R"html(
		<html>
			<head>
				<link rel="icon" href="data:, ">
				<title>OAuth Authorization Error</title>
			</head>
			<body>
				<p>%2</p>
			</body>
		</html>
	)html").simplified();
