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

#pragma once

#include <QDialog>

class QNetworkReply;
class QSettings;
class QString;
class QNetworkAccessManager;

class OAuthManager;
class OAuthToken;
enum class PoeApiMode;

namespace Ui {
	class LoginDialog;
}

class LoginDialog : public QDialog {
	Q_OBJECT
public:
	explicit LoginDialog(
		QSettings& settings,
		QNetworkAccessManager& network_manager,
		OAuthManager& oauth_manager);
	~LoginDialog();
signals:
	void LoginComplete(PoeApiMode mode);
private slots:
	void OnLeaguesReceived();
	void OnAuthenticateButtonClicked();
	void OnLoginButtonClicked();
	void OnStartLegacyLogin();
	void OnFinishLegacyLogin();
	void OnSessionIDChanged(const QString& session_id);
	void OnLeagueChanged(const QString& league);
	void OnProxyCheckBoxClicked(bool checked);
	void OnRememberMeCheckBoxClicked(bool checked);
	void OnOAuthAccessGranted(const OAuthToken& token);
protected:
	bool event(QEvent* e);
private:
	void LoadSettings();
	void SaveSettings();
	void RequestLeagues();
	void LoginWithOAuth();
	void LoginWithSessionID(const QString& session_id);
	void LeaguesRequestError(const QString& error, const QByteArray& reply);
	void DisplayError(const QString& error, bool disable_login = false);

	QSettings& settings_;
	QNetworkAccessManager& network_manager_;
	OAuthManager& oauth_manager_;

	Ui::LoginDialog* ui;
};
