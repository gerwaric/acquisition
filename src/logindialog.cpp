/*
	Copyright 2014 Ilya Zhuravlev

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

#include "logindialog.h"
#include "ui_logindialog.h"

#include <QDesktopServices>
#include <QFile>
#include <QMessageBox>
#include <QNetworkReply>
#include <QNetworkRequest>
#include <QNetworkCookie>
#include <QNetworkCookieJar>
#include <QNetworkProxyFactory>
#include <QRegularExpression>
#include <QSettings>
#include <QSysInfo>
#include <QUrl>
#include <QUrlQuery>
#include <iostream>
#include "QsLog.h"

#include "application.h"
#include "datastore.h"
#include "filesystem.h"
#include "mainwindow.h"
#include "network_info.h"
#include "replytimeout.h"
#include "util.h"
#include "updatechecker.h"
#include "oauth.h"
#include "version_defines.h"

const char* POE_LEAGUE_LIST_URL = "https://api.pathofexile.com/leagues?type=main&compact=1";
const char* POE_LOGIN_URL = "https://www.pathofexile.com/login";
const char* POE_MAIN_PAGE = "https://www.pathofexile.com/";
const char* POE_MY_ACCOUNT = "https://www.pathofexile.com/my-account";
const char* POE_LOGIN_CHECK_URL = POE_MY_ACCOUNT;
const char* POE_COOKIE_NAME = "POESESSID";

const char* LOGIN_CHECK_ERROR = "Failed to log in. Try copying your session ID again, or try OAuth";

const char* OAUTH_TAB = "oauthTab";
const char* SESSIONID_TAB = "sessionIdTab";

/**
 * Possible login flows:

 * OAuth
	=> Point browser to OAuth login page
	=> OnSteamCookieReceived() -> LoginWithCookie()
	=> Retrieve POE_LOGIN_CHECK_URL
	=> LoggedInCheck()
	=> Retrieve /my-account to get account name
	=> OnMainPageFinished()
	=> done

  * Session ID
	=> LoginWithCookie()
	=> Retrieve POE_LOGIN_CHECK_URL
	=> LoggedInCheck()
	=> Retrieve /my-account to get account name
	=> OnMainPageFinished()
	=> done
 */

LoginDialog::LoginDialog(Application& app) :
	app_(app),
	ui(new Ui::LoginDialog)
{
	ui->setupUi(this);
	ui->errorLabel->hide();
	ui->errorLabel->setStyleSheet("QLabel { color : red; }");
	setWindowTitle(QString("Login [") + APP_VERSION_STRING + "]");
	setWindowIcon(QIcon(":/icons/assets/icon.svg"));
	setWindowFlags(windowFlags() & ~Qt::WindowContextHelpButtonHint);

	LoadSettings();

	connect(ui->proxyCheckBox, &QCheckBox::clicked, this, &LoginDialog::OnProxyCheckBoxClicked);
	connect(ui->loginButton, &QPushButton::clicked, this, &LoginDialog::OnLoginButtonClicked);

	QNetworkRequest leagues_request = QNetworkRequest(QUrl(QString(POE_LEAGUE_LIST_URL)));
	leagues_request.setHeader(QNetworkRequest::KnownHeaders::UserAgentHeader, USER_AGENT);
	leagues_request.setTransferTimeout(kPoeApiTimeout);
	QNetworkReply* leagues_reply = app_.network_manager().get(leagues_request);

	connect(leagues_reply, &QNetworkReply::errorOccurred, this, &LoginDialog::errorOccurred);
	connect(leagues_reply, &QNetworkReply::sslErrors, this, &LoginDialog::sslErrorOccurred);
	connect(leagues_reply, &QNetworkReply::finished, this, &LoginDialog::OnLeaguesRequestFinished);
}

LoginDialog::~LoginDialog() {
	SaveSettings();
	delete ui;
}

void LoginDialog::errorOccurred() {
	QLOG_ERROR() << "League List errorOccured";
}

void LoginDialog::sslErrorOccurred() {
	QLOG_ERROR() << "League List sslErrorOccured";
}

void LoginDialog::OnLoginButtonClicked() {
	ui->loginButton->setEnabled(false);
	ui->loginButton->setText("Logging in...");

	const QString tab_name = ui->loginTabs->currentWidget()->objectName();

	if (tab_name == OAUTH_TAB) {
		LoginWithOAuth();
	} else if (tab_name == SESSIONID_TAB) {
		LoginWithCookie(ui->sessionIDLineEdit->text());
	} else {
		QLOG_ERROR() << "Invalid login tab name:" << tab_name;
	};
}

void LoginDialog::LeaguesApiError(const QString& error, const QByteArray& reply) {
	DisplayError("Leagues API returned malformed data: " + error, true);
	QLOG_ERROR() << "Leagues API says: " << reply;
}

void LoginDialog::OnLeaguesRequestFinished() {
	QNetworkReply* reply = qobject_cast<QNetworkReply*>(QObject::sender());
	QByteArray bytes = reply->readAll();
	reply->deleteLater();

	if (reply->error())
		return LeaguesApiError(reply->errorString(), bytes);

	rapidjson::Document doc;
	doc.Parse(bytes.constData());

	if (doc.HasParseError() || !doc.IsArray())
		return LeaguesApiError("Failed to parse the document", bytes);

	ui->leagueComboBox->clear();
	for (auto& league : doc) {
		if (!league.IsObject())
			return LeaguesApiError("Object expected", bytes);
		if (!league.HasMember("id"))
			return LeaguesApiError("Missing league 'id'", bytes);
		if (!league["id"].IsString())
			return LeaguesApiError("String expected", bytes);
		ui->leagueComboBox->addItem(league["id"].GetString());
	}
	ui->leagueComboBox->setEnabled(true);

	if (saved_league_.size() > 0)
		ui->leagueComboBox->setCurrentText(saved_league_);
}

// All characters except + should be handled by QUrlQuery
// See https://doc.qt.io/qt-6/qurlquery.html#encoding
static QString EncodeSpecialCharacters(QString s) {
	s.replace("+", "%2b");
	return s;
}

void LoginDialog::FinishLogin(QNetworkReply* reply) {
	QList<QNetworkCookie> cookies = reply->manager()->cookieJar()->cookiesForUrl(QUrl(POE_MAIN_PAGE));
	for (QNetworkCookie& cookie : cookies)
		if (QString(cookie.name()) == POE_COOKIE_NAME)
			session_id_ = cookie.value();

	// we need one more request to get account name
	QNetworkRequest main_page_request = QNetworkRequest(QUrl(POE_MY_ACCOUNT));
	main_page_request.setHeader(QNetworkRequest::KnownHeaders::UserAgentHeader, USER_AGENT);
	QNetworkReply* main_page = app_.network_manager().get(main_page_request);
	connect(main_page, &QNetworkReply::finished, this, &LoginDialog::OnMainPageFinished);
}

void LoginDialog::OnLoggedIn() {
	QNetworkReply* reply = qobject_cast<QNetworkReply*>(QObject::sender());
	QByteArray bytes = reply->readAll();
	int status = reply->attribute(QNetworkRequest::HttpStatusCodeAttribute).toInt();
	if (status != 302) {
		DisplayError(LOGIN_CHECK_ERROR);
		reply->deleteLater();
		return;
	}
	FinishLogin(reply);
	reply->deleteLater();
}

// Need a separate check since it's just the /login URL that's filtered
void LoginDialog::LoggedInCheck() {
	QNetworkReply* reply = qobject_cast<QNetworkReply*>(QObject::sender());
	QByteArray bytes = reply->readAll();
	int status = reply->attribute(QNetworkRequest::HttpStatusCodeAttribute).toInt();
	switch (status) {
	case 302:
		DisplayError(LOGIN_CHECK_ERROR);
		reply->deleteLater();
		return;
	case 401:
		DisplayError(LOGIN_CHECK_ERROR);
		reply->deleteLater();
		return;
	}
	FinishLogin(reply);
	reply->deleteLater();
}

void LoginDialog::LoginWithOAuth() {
	connect(&app_.oauth_manager(), &OAuthManager::accessGranted, this, &LoginDialog::OnOAuthAccessGranted);
	app_.oauth_manager().requestAccess();
}

void LoginDialog::OnOAuthAccessGranted(const OAuthToken& token) {
	const QString account = QString::fromStdString(token.username());
	const QString league = ui->leagueComboBox->currentText();
	emit LoginComplete(league, account, PoeApiMode::OAUTH);
}

void LoginDialog::LoginWithCookie(const QString& cookie) {
	QNetworkCookie poeCookie(POE_COOKIE_NAME, cookie.toUtf8());
	poeCookie.setPath("/");
	poeCookie.setDomain(".pathofexile.com");

	app_.network_manager().cookieJar()->insertCookie(poeCookie);

	QNetworkRequest login_page_request = QNetworkRequest(QUrl(POE_LOGIN_CHECK_URL));
	login_page_request.setHeader(QNetworkRequest::KnownHeaders::UserAgentHeader, USER_AGENT);
	QNetworkReply* login_page = app_.network_manager().get(login_page_request);
	connect(login_page, &QNetworkReply::finished, this, &LoginDialog::LoggedInCheck);
}

void LoginDialog::OnMainPageFinished() {
	QNetworkReply* reply = qobject_cast<QNetworkReply*>(QObject::sender());
	QString html(reply->readAll());
	reply->deleteLater();

	QRegularExpression regexp("/account/view-profile/(.*?)\"");
	QRegularExpressionMatch match = regexp.match(html, 0);
	if (match.hasMatch() == false) {
		DisplayError("Failed to find account name.");
		return;
	}
	QString account = match.captured(1);
	QLOG_DEBUG() << "Logged in as:" << account;

	const QString league = ui->leagueComboBox->currentText();

	emit LoginComplete(league, account, PoeApiMode::LEGACY);
}

void LoginDialog::OnProxyCheckBoxClicked(bool checked) {
	QNetworkProxyFactory::setUseSystemConfiguration(checked);
}

void LoginDialog::LoadSettings() {
	QSettings& settings = app_.settings();
	session_id_ = settings.value("session_id", "").toString();
	ui->sessionIDLineEdit->setText(session_id_);
	ui->rembmeCheckBox->setChecked(settings.value("remember_me_checked").toBool());
	ui->proxyCheckBox->setChecked(settings.value("use_system_proxy_checked").toBool());

	if (ui->rembmeCheckBox->isChecked()) {
		for (auto i = 0; i < ui->loginTabs->count(); ++i) {
			if (ui->loginTabs->widget(i)->objectName() == SESSIONID_TAB) {
				ui->loginTabs->setCurrentIndex(i);
				break;
			};
		};
	};

	saved_league_ = settings.value("league", "").toString();
	if (saved_league_.size() > 0)
		ui->leagueComboBox->setCurrentText(saved_league_);

	QNetworkProxyFactory::setUseSystemConfiguration(ui->proxyCheckBox->isChecked());
}

void LoginDialog::SaveSettings() {
	QSettings& settings = app_.settings();
	if (ui->rembmeCheckBox->isChecked()) {
		settings.setValue("session_id", session_id_);
		settings.setValue("league", ui->leagueComboBox->currentText());
	} else {
		settings.setValue("session_id", "");
		settings.setValue("league", "");
	}
	settings.setValue("remember_me_checked", ui->rembmeCheckBox->isChecked() && !session_id_.isEmpty());
	settings.setValue("use_system_proxy_checked", ui->proxyCheckBox->isChecked());
}

void LoginDialog::DisplayError(const QString& error, bool disable_login) {
	ui->errorLabel->setText(error);
	ui->errorLabel->show();
	ui->loginButton->setEnabled(!disable_login);
	ui->loginButton->setText("Login");
}


bool LoginDialog::event(QEvent* e) {
	if (e->type() == QEvent::LayoutRequest)
		setFixedSize(sizeHint());
	return QDialog::event(e);
}
