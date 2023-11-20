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

#include "poe_api/poe_league.h"

class QString;

class Application;
class MainWindow;
struct OAuthToken;
namespace Ui {
	class LoginDialog;
}

class LoginDialog : public QDialog {
	Q_OBJECT
public:
	explicit LoginDialog(std::unique_ptr<Application> app);
	~LoginDialog();
public slots:
	void OnAuthenticateButtonClicked();
	void OnProxyCheckBoxClicked(bool);
	void OnOAuthAccessGranted(const OAuthToken& token);
	void OnLeaguesReceived(const PoE::GetLeaguesResult& leagues);
	void OnLoginButtonClicked();
protected:
	bool event(QEvent* e);
private:
	void SaveSettings();
	void LoadSettings();
	void DisplayError(const QString& error, bool disable_login = false);
	std::unique_ptr<Application> app_;
	Ui::LoginDialog* ui;
	MainWindow* mw;
	QString settings_path_;
	QString saved_league_;
	std::string account_;
	std::vector<std::string> leagues_;
	bool asked_to_update_;
};
