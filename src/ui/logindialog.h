/*
    Copyright (C) 2014-2025 Acquisition Contributors

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
#include <QDir>

#include "network_info.h"
#include "util/oauthtoken.h"

class QNetworkReply;
class QSettings;
class QString;
class QNetworkAccessManager;

class OAuthManager;

class DataStore;
namespace Ui {
    class LoginDialog;
}

class LoginDialog : public QDialog
{
    Q_OBJECT
public:
    explicit LoginDialog(const QDir &app_data_dir,
                         QSettings &settings,
                         QNetworkAccessManager &network_manager,
                         OAuthManager &oauth_manager,
                         DataStore &data_store);
    ~LoginDialog();
signals:
    void ChangeTheme(const QString &theme);
    void ChangeUserDir(const QString &user_dir);
    void LoginComplete(POE_API mode);
private slots:
    void OnLeaguesReceived();
    void OnAuthenticateButtonClicked();
    void OnLoginTabChanged(int index);
    void OnLoginButtonClicked();
    void OnOfflineButtonClicked();
    void OnStartLegacyLogin();
    void OnFinishLegacyLogin();
    void OnSessionIDChanged(const QString &session_id);
    void OnLeagueChanged(const QString &league);
    void OnAdvancedCheckBoxChanged(Qt::CheckState state);
    void OnProxyCheckBoxChanged(Qt::CheckState state);
    void OnRememberMeCheckBoxChanged(Qt::CheckState state);
    void OnReportCrashesCheckBoxChanged(Qt::CheckState state);
    void OnLoggingLevelChanged(const QString &level);
    void OnThemeChanged(const QString &theme);
    void OnUserDirButtonPushed();
    void OnOAuthAccessGranted(const OAuthToken &token);

protected:
    bool event(QEvent *e);

private:
    void LoadSettings();
    void RequestLeagues();
    void LoginWithOAuth();
    void LoginWithSessionID();
    void LeaguesRequestError(const QString &error, const QByteArray &reply);
    void DisplayError(const QString &error);

    const QDir m_app_data_dir;
    QSettings &m_settings;
    QNetworkAccessManager &m_network_manager;
    OAuthManager &m_oauth_manager;
    DataStore &m_datastore;

    std::optional<OAuthToken> m_current_token;

    Ui::LoginDialog *ui;
};
