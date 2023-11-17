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

#include "poe_api/poe_league.h"

#include "application.h"
#include "datastore.h"
#include "filesystem.h"
#include "mainwindow.h"
#include "updatechecker.h"
#include "oauth.h"
#include "version_defines.h"

LoginDialog::LoginDialog(std::unique_ptr<Application> app) :
	app_(std::move(app)),
	ui(new Ui::LoginDialog),
	mw(0),
	asked_to_update_(false)
{
	ui->setupUi(this);
	ui->loginButton->setEnabled(false);
	ui->errorLabel->hide();
	ui->errorLabel->setStyleSheet("QLabel { color : red; }");
	setWindowTitle("Login [" APP_VERSION_STRING "]");

#if defined(Q_OS_LINUX)
	setWindowIcon(QIcon(":/icons/assets/icon.svg"));
#endif
	setWindowFlags(windowFlags() & ~Qt::WindowContextHelpButtonHint);

	settings_path_ = Filesystem::UserDir() + "/settings.ini";
	LoadSettings();
	LoadTheme();

	const bool supports_ssl = QSslSocket::supportsSsl();
	QLOG_DEBUG() << "Supports SSL: " << supports_ssl;
	if (supports_ssl == false) {
#ifdef Q_OS_LINUX
		const QString msg = "OpenSSL 3.x was not found; check LD_LIBRARY_PATH if you have a custom installation.";
#else
		const QString msg = "SSL is not supported on" + QSysInfo::prettyProductName() + ". This is unexpected.";
#endif
		DisplayError(msg, true);
		QLOG_FATAL() << msg;
		ui->loginButton->setEnabled(false);
		return;
	};
	QLOG_DEBUG() << "SSL Library Build Version: " << QSslSocket::sslLibraryBuildVersionString();
	QLOG_DEBUG() << "SSL Library Version: " << QSslSocket::sslLibraryVersionString();

	connect(ui->authenticateButton, &QPushButton::clicked, this, &LoginDialog::OnAuthenticateButtonClicked);
	connect(ui->proxyCheckBox, &QCheckBox::clicked, this, &LoginDialog::OnProxyCheckBoxClicked);
	connect(ui->loginButton, &QPushButton::clicked, this, &LoginDialog::OnLoginButtonClicked);
	connect(&app_->update_checker(), &UpdateChecker::UpdateAvailable, this,
		[&]() {
			// Only annoy the user once at the login dialog window,
			// even if it's opened for more than an hour.
			if (asked_to_update_ == false) {
				asked_to_update_ = true;
				UpdateChecker::AskUserToUpdate(this);
			};
		});
}

void LoginDialog::LoadSettings() {
	QSettings settings(settings_path_, QSettings::IniFormat);
	ui->rembmeCheckBox->setChecked(settings.value("remember_me_checked").toBool());
	ui->proxyCheckBox->setChecked(settings.value("use_system_proxy_checked").toBool());
	saved_league_ = settings.value("league", "").toString();
	if (saved_league_.size() > 0) {
		ui->leagueComboBox->setCurrentText(saved_league_);
	};
	QNetworkProxyFactory::setUseSystemConfiguration(ui->proxyCheckBox->isChecked());
}

void LoginDialog::LoadTheme() {
	// Load the appropriate theme.
	const std::string theme = app_->global_data().Get("theme", "default");

	// Do nothing for the default theme.
	if (theme == "default") {
		return;
	};

	// Determine which qss file to use.
	QString stylesheet;
	if (theme == "dark") {
		stylesheet = ":qdarkstyle/dark/darkstyle.qss";
	} else if (theme == "light") {
		stylesheet = ":qdarkstyle/light/lightstyle.qss";
	} else {
		QLOG_ERROR() << "Invalid theme:" << theme;
		return;
	};

	// Load the theme.
	QFile f(stylesheet);
	if (!f.exists()) {
		QLOG_ERROR() << "Theme stylesheet not found:" << stylesheet;
	} else {
		f.open(QFile::ReadOnly | QFile::Text);
		QTextStream ts(&f);
		const QString stylesheet = ts.readAll();
		qApp->setStyleSheet(stylesheet);
		QPalette pal = QApplication::palette();
		pal.setColor(QPalette::WindowText, Qt::white);
		QApplication::setPalette(pal);
	};
}

void LoginDialog::OnAuthenticateButtonClicked() {
	ui->authenticateButton->setEnabled(false);
	ui->authenticateButton->setText("Authenticating...");
	connect(&app_->oauth_manager(), &OAuthManager::accessGranted, this, &LoginDialog::OnOAuthAccessGranted);
	app_->oauth_manager().requestAccess();
}

void LoginDialog::OnProxyCheckBoxClicked(bool checked) {
	QNetworkProxyFactory::setUseSystemConfiguration(checked);
}

void LoginDialog::OnOAuthAccessGranted(const AccessToken& token) {
	account_ = token.username.toStdString();
	ui->authenticateLabel->setText("You are authenticated as \"" + token.username + "\"");
	ui->authenticateButton->setText("Re-authenticate (as someone else).");
	ui->authenticateButton->setEnabled(true);
	PoE::GetLeagues(token, this,
		[=](const PoE::GetLeaguesResult& leagues) {
			OnLeaguesReceived(leagues);
		});
}

void LoginDialog::OnLeaguesReceived(const PoE::GetLeaguesResult& result) {
	ui->leagueComboBox->setEnabled(true);
	for (auto& league : result.leagues) {
		ui->leagueComboBox->addItem(QString::fromStdString(league.id));
	};
	ui->loginButton->setEnabled(true);
}

void LoginDialog::OnLoginButtonClicked() {
	const std::string league = ui->leagueComboBox->currentText().toStdString();
	const QString window_title = QString("Acquisition [%1] - %2 [%3]").arg(
		QString(APP_VERSION_STRING),
		QString::fromStdString(league),
		QString::fromStdString(account_));
	app_->InitLogin(league, account_);
	mw = new MainWindow(std::move(app_));
	mw->setWindowTitle(window_title);
	mw->show();
	close();
}

LoginDialog::~LoginDialog() {
	SaveSettings();
	delete ui;
	if (mw) {
		delete mw;
	};
}

void LoginDialog::SaveSettings() {
	QSettings settings(settings_path_, QSettings::IniFormat);
	const bool remember_me = ui->rembmeCheckBox->isChecked();
	settings.setValue("league", remember_me ? ui->leagueComboBox->currentText() : "");
	settings.setValue("remember_me_checked", ui->rembmeCheckBox->isChecked());
	settings.setValue("use_system_proxy_checked", ui->proxyCheckBox->isChecked());
}

void LoginDialog::DisplayError(const QString& error, bool disable_login) {
	ui->errorLabel->setText(error);
	ui->errorLabel->show();
	ui->loginButton->setEnabled(!disable_login);
	ui->loginButton->setText("Authenticate");
}

bool LoginDialog::event(QEvent* e) {
	if (e->type() == QEvent::LayoutRequest) {
		setFixedSize(sizeHint());
	};
	return QDialog::event(e);
}
