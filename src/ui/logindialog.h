// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

#pragma once

#include <QDialog>
#include <QDir>

#include "util/oauthtoken.h"

class QNetworkReply;
class QSettings;
class QString;

class NetworkManager;
class OAuthManager;

class DataStore;
namespace Ui {
    class LoginDialog;
}
namespace app {
    class UserSettings;
}

class LoginDialog : public QDialog
{
    Q_OBJECT
public:
    explicit LoginDialog(app::UserSettings &settings,
                         NetworkManager &network_manager,
                         OAuthManager &oauth_manager);
    ~LoginDialog();
signals:
    void ChangeTheme(const QString &theme);
    void ChangeUserDir(const QString &user_dir);
    void LoginComplete();
    void RemoveOAuthToken();
private slots:
    void OnLeaguesReceived();
    void OnAuthenticateButtonClicked();
    void OnLoginTabChanged(int index);
    void OnLoginButtonClicked();
    void OnLeagueChanged(const QString &league);
    void OnAdvancedCheckBoxChanged(Qt::CheckState state);
    void OnProxyCheckBoxChanged(Qt::CheckState state);
    void OnRememberMeCheckBoxChanged(Qt::CheckState state);
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
    void LeaguesRequestError(const QString &error, const QByteArray &reply);
    void DisplayError(const QString &error);

    app::UserSettings &m_settings;
    NetworkManager &m_network_manager;
    OAuthManager &m_oauth_manager;

    std::optional<OAuthToken> m_current_token;

    Ui::LoginDialog *ui;
};
