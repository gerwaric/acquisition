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
#include <memory>
#include <string>
#include <vector>

class QNetworkReply;
class QString;

class Application;
class OAuthToken;
enum class PoeApiMode;

namespace Ui {
	class LoginDialog;
}

class LoginDialog : public QDialog {
	Q_OBJECT
public:
	explicit LoginDialog(Application& app);
	~LoginDialog();
signals:
	void LoginComplete(const QString& league, const QString& account, PoeApiMode mode);
public slots:
	void OnLeaguesRequestFinished();
	void OnLoginButtonClicked();
	void OnLoggedIn();
	void LoggedInCheck(); // checks login is successful
	void OnMainPageFinished();
	void OnProxyCheckBoxClicked(bool);
	void OnOAuthAccessGranted(const OAuthToken& token);
	void errorOccurred();
	void sslErrorOccurred();
protected:
	bool event(QEvent* e);
private:
	void SaveSettings();
	void LoadSettings();
	void DisplayError(const QString& error, bool disable_login = false);
	void LoginWithOAuth();
	void LoginWithCookie(const QString& cookie);
	void LeaguesApiError(const QString& error, const QByteArray& reply);
	// Retrieves session cookie for a successful login; proceeds to OnMainPageFinished
	void FinishLogin(QNetworkReply* reply);
	Application& app_;
	Ui::LoginDialog* ui;
	QString saved_league_;
	QString session_id_;
	std::vector<std::string> leagues_;
};
