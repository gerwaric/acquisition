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

#include "logindialog.h"
#include "ui_logindialog.h"

#include <QDesktopServices>
#include <QFile>
#include <QFileDialog>
#include <QFileInfo>
#include <QMessageBox>
#include <QNetworkAccessManager>
#include <QNetworkCookie>
#include <QNetworkCookieJar>
#include <QNetworkProxyFactory>
#include <QNetworkReply>
#include <QNetworkRequest>
#include <QPushButton>
#include <QRegularExpression>
#include <QSettings>
#include <QSysInfo>
#include <QUrl>
#include <QUrlQuery>

#include <poe/types/league.h>

// #include "legacy/legacybuyoutvalidator.h" -- DISABLED as of v0.12.3.1
#include <datastore/datastore.h>
#include <util/crashpad.h>
#include <util/oauthmanager.h>
#include <util/spdlog_qt.h>
#include <util/updatechecker.h>
#include <util/util.h>
#include <version_defines.h>

#include <network_info.h>
#include <replytimeout.h>

#include "mainwindow.h"

constexpr const char *POE_LEAGUE_LIST_URL
    = "https://api.pathofexile.com/leagues?type=main&compact=1";
constexpr const char *POE_MAIN_PAGE = "https://www.pathofexile.com/";
constexpr const char *POE_MY_ACCOUNT = "https://www.pathofexile.com/my-account";
constexpr const char *POE_LOGIN_CHECK_URL = POE_MY_ACCOUNT;

constexpr int CLOUDFLARE_RATE_LIMITED = 1015;

constexpr const char *OAUTH_TAB = "oauthTab";
constexpr const char *SESSIONID_TAB = "sessionIdTab";
constexpr const char *OFFLINE_TAB = "offlineTab";

/**
 *
 * Possible login flows:
 *
 * Oauth:
 *   1. OnLoginButtonClicked()
 *   2. LoginWithOAuth()
 *
 * Session ID:
 *   1. OnLoginButtonClicked()
 *   2. LoginWithSessionID()
 *	 3. OnStartLegacyLogin()
 *   4. OnFinishLegacyLogin()
 *
 */

LoginDialog::LoginDialog(const QDir &app_data_dir,
                         QSettings &settings,
                         QNetworkAccessManager &network_manager,
                         OAuthManager &oauth_manager,
                         DataStore &datastore)
    : QDialog(nullptr)
    , m_app_data_dir(app_data_dir)
    , m_settings(settings)
    , m_network_manager(network_manager)
    , m_oauth_manager(oauth_manager)
    , m_datastore(datastore)
    , ui(new Ui::LoginDialog)
{
    // Setup the dialog box.
    spdlog::trace("LoginDialog::LoginDialog() calling UI setup");
    ui->setupUi(this);

    // Set window properties.
    spdlog::trace("LoginDialog::LoginDialog() setting window properties");
    setWindowTitle(QString("Acquisition Login [") + APP_VERSION_STRING + "]");
    setWindowIcon(QIcon(":/icons/assets/icon.svg"));
    setWindowFlags(windowFlags() & ~Qt::WindowContextHelpButtonHint);

    // Setup the realm options.
    ui->realmComboBox->addItems({"pc", "sony", "xbox"});

    // Setup theme.
    ui->themeComboBox->addItems({"default", "light", "dark"});

    // Setup logging levels.
    for (const auto &level_name : spdlog::level::level_string_views) {
        ui->loggingLevelComboBox->addItem(QString::fromUtf8(level_name));
    }

    // Display the data directory
    ui->userDirButton->setText(m_app_data_dir.absolutePath());

    // Hide the error message by default
    ui->errorLabel->setText("");
    ui->errorLabel->hide();
    ui->errorLabel->setStyleSheet("QLabel {color: red;}");

    // Disable the login button until we are ready to login.
    ui->loginButton->setEnabled(false);

    // Load saved settings.
    LoadSettings();

    // Set the proxy.
    QNetworkProxyFactory::setUseSystemConfiguration(ui->proxyCheckBox->isChecked());

    // Connect main UI buttons.
    connect(ui->loginTabs, &QTabWidget::currentChanged, this, &LoginDialog::OnLoginTabChanged);
    connect(ui->authenticateButton,
            &QPushButton::clicked,
            this,
            &LoginDialog::OnAuthenticateButtonClicked);
    connect(ui->sessionIDLineEdit, &QLineEdit::textChanged, this, &LoginDialog::OnSessionIDChanged);
    connect(ui->loginButton, &QPushButton::clicked, this, &LoginDialog::OnLoginButtonClicked);

    // Connects options UI elements.
    connect(ui->advancedCheckBox,
            &QCheckBox::checkStateChanged,
            this,
            &LoginDialog::OnAdvancedCheckBoxChanged);
    connect(ui->rememberMeCheckBox,
            &QCheckBox::checkStateChanged,
            this,
            &LoginDialog::OnRememberMeCheckBoxChanged);
    connect(ui->reportCrashesCheckBox,
            &QCheckBox::checkStateChanged,
            this,
            &LoginDialog::OnReportCrashesCheckBoxChanged);
    connect(ui->proxyCheckBox,
            &QCheckBox::checkStateChanged,
            this,
            &LoginDialog::OnProxyCheckBoxChanged);
    connect(ui->loggingLevelComboBox,
            &QComboBox::currentTextChanged,
            this,
            &LoginDialog::OnLoggingLevelChanged);
    connect(ui->userDirButton, &QPushButton::pressed, this, &LoginDialog::OnUserDirButtonPushed);
    connect(ui->themeComboBox, &QComboBox::currentTextChanged, this, &LoginDialog::OnThemeChanged);

    // Listen for access from the OAuth manager.
    connect(&m_oauth_manager,
            &OAuthManager::grantAccess,
            this,
            &LoginDialog::OnOAuthAccessGranted);

    // Use OnLoginTabChanged to do things like enable the login button.
    OnLoginTabChanged(ui->loginTabs->currentIndex());

    // Request the list of leagues.
    spdlog::trace("Login: requesting leagues");
    RequestLeagues();
}

LoginDialog::~LoginDialog()
{
    if (!ui->rememberMeCheckBox->isChecked()) {
        if (auto lg = spdlog::default_logger(); lg) {
            lg->trace("Login: clearing settings");
        }
        m_settings.clear();
        m_datastore.Set("oauth_token", "");
    }
    delete ui;
}

void LoginDialog::LoadSettings()
{
    const QString realm = m_settings.value("realm").toString();
    ui->realmComboBox->setCurrentText(realm);

    const QString league = m_settings.value("league").toString();
    ui->leagueComboBox->setCurrentText(league);

    const spdlog::string_view_t sv = spdlog::level::to_string_view(spdlog::get_level());
    const QString logging_level = QString::fromUtf8(sv.data(), sv.size());
    ui->loggingLevelComboBox->setCurrentText(logging_level);

    const QString theme = m_settings.value("theme").toString();
    ui->themeComboBox->setCurrentText(theme);

    const QString session_id = m_settings.value("session_id").toString();
    ui->sessionIDLineEdit->setText(session_id);

    const bool show_advanced = m_settings.value("show_advanced_login_options").toBool();
    ui->advancedCheckBox->setChecked(show_advanced);

    const bool remember_user = m_settings.value("remember_user").toBool();
    ui->rememberMeCheckBox->setChecked(remember_user);

    const bool use_proxy = m_settings.value("use_system_proxy").toBool();
    ui->proxyCheckBox->setChecked(use_proxy);

    const bool report_crashes = m_settings.value("report_crashes").toBool();
    ui->reportCrashesCheckBox->setChecked(report_crashes);

    const QString login_tab = m_settings.value("login_tab").toString();
    spdlog::trace("LoginDialog::LoadSettings() login_tab = {}", login_tab);
    for (auto i = 0; i < ui->loginTabs->count(); ++i) {
        const QString tab_name = ui->loginTabs->widget(i)->objectName();
        if (0 == login_tab.compare(tab_name, Qt::CaseInsensitive)) {
            ui->loginTabs->setCurrentIndex(i);
            break;
        }
    }


}

void LoginDialog::RequestLeagues()
{
    // Make a non-API request to get the list of leagues. This currently uses
    // a legacy endpoint that is not rate limited and does not require authentication.
    QNetworkRequest request = QNetworkRequest(QUrl(QString(POE_LEAGUE_LIST_URL)));
    request.setTransferTimeout(kPoeApiTimeout);

    // Send the request and handle errors.
    spdlog::trace("LoginDialog::RequestLeagues() sending request: {}", request.url().toString());
    QNetworkReply *reply = m_network_manager.get(request);
    connect(reply, &QNetworkReply::finished, this, &LoginDialog::OnLeaguesReceived);
    connect(reply, &QNetworkReply::errorOccurred, this, [=, this](QNetworkReply::NetworkError code) {
        Q_UNUSED(code);
        DisplayError("Error requesting leagues: " + reply->errorString());
        ui->loginButton->setEnabled(false);
    });
    connect(reply, &QNetworkReply::sslErrors, this, [=, this](const QList<QSslError> &errors) {
        for (const auto &error : errors) {
            spdlog::error("SSL Error requesting leagues: {}", error.errorString());
        }
        DisplayError("SSL error fetching leagues");
        ui->loginButton->setEnabled(false);
    });
}

void LoginDialog::OnLeaguesReceived()
{
    spdlog::trace("LoginDialog::OnLeaguesReceived() reply recieved");
    QNetworkReply *reply = qobject_cast<QNetworkReply *>(QObject::sender());
    const QByteArray bytes = reply->readAll();
    reply->deleteLater();

    // Check for network errors.
    if (reply->error()) {
        spdlog::trace("LoginDialog::OnLeaguesReceived() reply error {}", reply->error());
        return LeaguesRequestError(reply->errorString(), bytes);
    }

    // Parse the leagues.
    const auto leagues = Util::parseJson<std::vector<poe::League>>(bytes);

    // Get the league from settings.ini
    const QString saved_league = m_settings.value("league").toString();
    spdlog::trace("LoginDialog::OnLeaguesReceived() loaded leage from settings: {}", saved_league);

    bool use_saved_league = false;

    ui->leagueComboBox->clear();
    for (auto &league : leagues) {
        // Add the league to the combo box.
        ui->leagueComboBox->addItem(league.id);

        // Set the current league if it matches the saved league.
        if (0 == saved_league.compare(league.id, Qt::CaseInsensitive)) {
            use_saved_league = true;
        }
    }
    ui->leagueComboBox->setEnabled(true);

    // If we found a match for the save league use it. If we didn't, then
    // we need to clear the setting, since the list of leagues may have
    // changed since the last time acquisition was run.
    if (use_saved_league) {
        spdlog::trace("LoginDialog::OnLeaguesReceived() setting current league to {}", saved_league);
        ui->leagueComboBox->setCurrentText(saved_league);
    } else {
        spdlog::trace("LoginDialog::OnLeaguesReceived() clearing the saved league");
        m_settings.setValue("league", "");
    }

    // Now that leagues have been received, start listening for changes.
    connect(ui->leagueComboBox, &QComboBox::currentTextChanged, this, &LoginDialog::OnLeagueChanged);

    // Now we can let the user login.
    //ui->loginButton->setEnabled(true);
}

void LoginDialog::LeaguesRequestError(const QString &error, const QByteArray &reply)
{
    spdlog::error("League reply was: {}", reply);
    DisplayError("Error requesting leagues: " + error);
    ui->loginButton->setEnabled(false);
}

void LoginDialog::OnAuthenticateButtonClicked()
{
    spdlog::trace("LoginDialog::OnAuthenticateButtonClicked() entered");
    ui->errorLabel->hide();
    ui->errorLabel->setText("");
    ui->authenticateButton->setEnabled(false);
    ui->authenticateButton->setText("Authenticating...");
    m_oauth_manager.initLogin();
}

void LoginDialog::OnLoginButtonClicked()
{
    spdlog::trace("LoginDialog::OnLoginButtonClicked() entered");
    ui->errorLabel->hide();
    ui->errorLabel->setText("");
    ui->loginButton->setEnabled(false);
    ui->loginButton->setText("Logging in...");

    const QString realm = ui->realmComboBox->currentText();
    const QString league = ui->leagueComboBox->currentText();
    const QString session_id = ui->sessionIDLineEdit->text();
    m_settings.setValue("realm", realm);
    m_settings.setValue("league", league);
    m_settings.setValue("session_id", session_id);
    if (!session_id.isEmpty()) {
        QNetworkCookie poesessid(POE_COOKIE_NAME, session_id.toUtf8());
        poesessid.setPath(POE_COOKIE_PATH);
        poesessid.setDomain(POE_COOKIE_DOMAIN);
        m_network_manager.cookieJar()->insertCookie(poesessid);
    }

    const QString tab_name = ui->loginTabs->currentWidget()->objectName();
    if (tab_name == OAUTH_TAB) {
        LoginWithOAuth();
    } else if (tab_name == SESSIONID_TAB) {
        if (session_id.isEmpty()) {
            spdlog::error("POESESSID is empty");
            DisplayError("POESESSID cannot be blank");
            ui->loginButton->setEnabled(true);
            ui->loginButton->setText("Log in");
        } else {
            LoginWithSessionID();
            ui->loginButton->setEnabled(false);
        }
    } else if (tab_name == OFFLINE_TAB) {
    } else {
        DisplayError("Invalid tab selected: " + tab_name);
    }
}

void LoginDialog::OnOfflineButtonClicked() {}

void LoginDialog::LoginWithOAuth()
{
    spdlog::info("Starting OAuth authentication");
    const QDateTime now = QDateTime::currentDateTime();
    if (m_current_token.has_value()) {
        const OAuthToken &token = m_current_token.value();
        if (token.access_expiration && (now < *token.access_expiration)) {
            m_settings.setValue("account", token.username);
            emit LoginComplete(POE_API::OAUTH);
        } else if (token.refresh_expiration && (now < *token.refresh_expiration)) {
            DisplayError("The OAuth token needs to be refreshed");
        } else {
            DisplayError("The OAuth token is not valid.");
        }
    } else {
        DisplayError("You are not authenticated.");
    }
}

void LoginDialog::LoginWithSessionID()
{
    spdlog::info("Starting legacy login with POESESSID");
    QNetworkRequest request = QNetworkRequest(QUrl(POE_LOGIN_CHECK_URL));
    QNetworkReply *reply = m_network_manager.get(request);

    connect(reply, &QNetworkReply::finished, this, &LoginDialog::OnStartLegacyLogin);
    connect(reply, &QNetworkReply::errorOccurred, this, [=, this](QNetworkReply::NetworkError code) {
        const int error_code = static_cast<int>(code);
        if (error_code == CLOUDFLARE_RATE_LIMITED) {
            DisplayError("Rate limited by Cloudflare! Please report to gerwaric@gmail.com");
            ui->loginButton->setEnabled(false);
        } else {
            DisplayError("Error during legacy login: " + reply->errorString());
            ui->loginButton->setEnabled(false);
        }
    });
    connect(reply, &QNetworkReply::sslErrors, this, [=, this](const QList<QSslError> &errors) {
        for (const auto &error : errors) {
            spdlog::error("SSL error during legacy login: {}", error.errorString());
        }
        DisplayError("SSL error during session id login");
        ui->loginButton->setEnabled(false);
    });
}

// Need a separate check since it's just the /login URL that's filtered
void LoginDialog::OnStartLegacyLogin()
{
    spdlog::trace("LoginDialog::OnStartLegacyLogin() entered");

    QNetworkReply *reply = qobject_cast<QNetworkReply *>(QObject::sender());
    const auto cookies = reply->manager()->cookieJar()->cookiesForUrl(QUrl(POE_MAIN_PAGE));
    reply->deleteLater();

    // Check for HTTP errors.
    if (reply->error() != QNetworkReply::NoError) {
        const int error_code = static_cast<int>(reply->error());
        QString msg;
        if (error_code == 204) {
            msg = "You appear to be logged out. Please try updating your POESESSID.";
        } else if (error_code == 1015) {
            msg = "Your account or ip seems to have been blocked by Cloudflare!";
        } else {
            msg = QString("Network error %1 during legacy login: %2")
                      .arg(QString::number(reply->error()), reply->errorString());
        }
        DisplayError(msg);
        return;
    }

    // Check the session id cookie.
    const QString session_id = m_settings.value("session_id").toString();
    for (const QNetworkCookie &cookie : cookies) {
        if (QString(cookie.name()) == POE_COOKIE_NAME) {
            if (cookie.value() != session_id) {
                spdlog::warn("POESESSID mismatch");
            }
            break;
        }
    }

    // we need one more request to get account name
    QNetworkRequest request = QNetworkRequest(QUrl(POE_MY_ACCOUNT));
    QNetworkReply *next_reply = m_network_manager.get(request);

    connect(next_reply, &QNetworkReply::finished, this, &LoginDialog::OnFinishLegacyLogin);
    connect(reply, &QNetworkReply::errorOccurred, this, [=, this](QNetworkReply::NetworkError code) {
        const int error_code = static_cast<int>(code);
        if (error_code == CLOUDFLARE_RATE_LIMITED) {
            DisplayError("Blocked by Cloudflare! Please tell gerwaric@gmail.com. You may need to "
                         "contact GGG support :-(");
        } else {
            DisplayError("Error finishing legacy login: " + reply->errorString());
            ui->loginButton->setEnabled(false);
        }
    });
    connect(reply, &QNetworkReply::sslErrors, this, [=, this](const QList<QSslError> &errors) {
        for (const auto &error : errors) {
            spdlog::error("SSL finishing legacy login: {}", error.errorString());
        }
        DisplayError("SSL error finishing legacy login");
        ui->loginButton->setEnabled(false);
    });
}

void LoginDialog::OnFinishLegacyLogin()
{
    spdlog::trace("LoginDialog::OnFinishLegacyLogin() entered");

    QNetworkReply *reply = qobject_cast<QNetworkReply *>(QObject::sender());
    const QString html(reply->readAll());
    reply->deleteLater();
    if (reply->error()) {
        DisplayError("Network error finishing legacy loging " + reply->errorString());
        return;
    }

    static const QRegularExpression regexp("/account/view-profile/.*?>(.*?)<");
    QRegularExpressionMatch match = regexp.match(html, 0);
    if (match.hasMatch() == false) {
        DisplayError("Failed to find account name.");
        return;
    }

    const QString account = match.captured(1);
    const QString realm = m_settings.value("realm").toString();
    const QString league = m_settings.value("league").toString();
    m_settings.setValue("account", account);

    spdlog::debug("Logged in as {} to {} league in {} realm", account, league, realm);

    emit LoginComplete(POE_API::LEGACY);
}

void LoginDialog::OnOAuthAccessGranted(const OAuthToken &token)
{
    spdlog::trace("LoginDialog::OnOAuthAccessGranted() entered");
    ui->authenticateLabel->setText("You are authenticated as \"" + token.username + "\"");
    ui->authenticateButton->setText("Re-authenticate (as someone else).");
    ui->authenticateButton->setEnabled(true);
    if (ui->loginTabs->currentWidget() == ui->oauthTab) {
        ui->loginButton->setEnabled(true);
    }
    m_current_token = token;
}

void LoginDialog::OnLoginTabChanged(int index)
{
    spdlog::trace("LoginDialog::OnLoginTabChanged() entered");
    const QWidget *tab = ui->loginTabs->widget(index);
    if (tab == nullptr) {
        spdlog::error("LoginDialog: current tab widget is null");
        return;
    }
    const bool hide_options = false;
    const bool hide_advanced = !ui->advancedCheckBox->isChecked();
    const bool hide_error = ui->errorLabel->text().isEmpty();
    ui->optionsWidget->setHidden(hide_options);
    ui->advancedOptionsFrame->setHidden(hide_options || hide_advanced);
    ui->errorLabel->setHidden(hide_options || hide_error);
    ui->loginButton->setHidden(hide_options);
    if (tab == ui->oauthTab) {
        ui->loginButton->setEnabled(m_current_token.has_value());
    } else if (tab == ui->sessionIdTab) {
        ui->loginButton->setEnabled(!ui->sessionIDLineEdit->text().isEmpty());
    }
    m_settings.setValue("login_tab", tab->objectName());
};

void LoginDialog::OnSessionIDChanged(const QString &session_id)
{
    // Save the new session and make sure the login button is enabled.
    spdlog::trace("LoginDialog::OnSessionIDChanged() entered");
    m_settings.setValue("session_id", session_id);
    ui->loginButton->setEnabled(!session_id.isEmpty());
}

void LoginDialog::OnLeagueChanged(const QString &league)
{
    spdlog::trace("LoginDialog::OnLeagueChanged() entered");
    m_settings.setValue("league", league);
}

void LoginDialog::OnAdvancedCheckBoxChanged(Qt::CheckState state)
{
    spdlog::trace("LoginDialog: advanced options checkbox changed to {}", state);
    const bool checked = (state == Qt::Checked);
    const bool hide_options = false;
    ui->advancedOptionsFrame->setHidden(!checked || hide_options);
    m_settings.setValue("show_advanced_login_options", checked);
}

void LoginDialog::OnProxyCheckBoxChanged(Qt::CheckState state)
{
    spdlog::trace("LoginDialog: proxy checkbox changed to {}", state);
    const bool checked = (state == Qt::Checked);
    QNetworkProxyFactory::setUseSystemConfiguration(checked);
    m_settings.setValue("use_system_proxy", checked);
}

void LoginDialog::OnRememberMeCheckBoxChanged(Qt::CheckState state)
{
    spdlog::trace("LoginDialog: remember me checkbox changed to {}", state);
    const bool checked = (state == Qt::Checked);
    m_settings.setValue("remember_user", checked);
}

void LoginDialog::OnReportCrashesCheckBoxChanged(Qt::CheckState state)
{
    spdlog::trace("LoginDialog: crash reporting checkbox changed to {}", state);
    const bool checked = (state == Qt::Checked);
    QMessageBox msgbox(this);
    msgbox.setWindowTitle("Acquisition Crash Reporting");

    if (checked) {
        // Before enabling crash reporting, make sure the user
        // understands and accepts that crash reporting cannot be
        // disabled without restarting acquisition.
        msgbox.setText("Once crash reporting is enabled, it cannot be "
                       "disabled without restarting Acquisition.\n\nDo you want to "
                       "enable crash reporting?");
        auto yes = msgbox.addButton("  Enable crash reports  ", msgbox.YesRole);
        msgbox.addButton("  Cancel  ", msgbox.NoRole);
        msgbox.exec();
        const bool enable_reporting = (msgbox.clickedButton() == yes);
        m_settings.setValue("report_crashes", enable_reporting);
        if (enable_reporting) {
            initializeCrashpad(m_app_data_dir.absolutePath(),
                               APP_PUBLISHER,
                               APP_NAME,
                               APP_VERSION_STRING);
        } else {
            ui->reportCrashesCheckBox->setChecked(false);
        }

    } else {
        // Since crashpad cannot be stopped once it is started, acquisition
        // will have to be exited and restarted to disable crash reporting,
        // so make sure the user accepts and agrees to this.
        msgbox.setText("Acquisition will have to restart to disable crash "
                       "reporting.\n\nDo you want Acquisition to exit now and disable "
                       "crash reporting the next time it runs?");
        auto yes = msgbox.addButton("  Yes, exit now  ", msgbox.YesRole);
        msgbox.addButton("  No, continue without crash reporting  ", msgbox.NoRole);
        msgbox.exec();
        const bool disable_reporting = (msgbox.clickedButton() == yes);
        m_settings.setValue("report_crashes", !disable_reporting);
        if (disable_reporting) {
            close();
        } else {
            ui->reportCrashesCheckBox->setChecked(true);
        }
    }
}

void LoginDialog::OnUserDirButtonPushed()
{
    const QString current_dir = QFileInfo(m_settings.fileName()).absolutePath();
    const QString parent_dir = QFileInfo(current_dir).absolutePath();
    const QString new_dir = QFileDialog::getExistingDirectory(this,
                                                              "Select the user directory",
                                                              parent_dir);
    if (new_dir != current_dir) {
        ui->userDirButton->setText(new_dir);
        emit ChangeUserDir(new_dir);
    }
}

void LoginDialog::OnLoggingLevelChanged(const QString &level_name)
{
    const spdlog::level::level_enum level = spdlog::level::from_str(level_name.toStdString());
    spdlog::set_level(level);
    m_settings.setValue("log_level", level_name);
}

void LoginDialog::OnThemeChanged(const QString &theme)
{
    emit ChangeTheme(theme);
}

void LoginDialog::DisplayError(const QString &error)
{
    spdlog::error("LoginDialog: {}", error);
    ui->errorLabel->setText(error);
    ui->errorLabel->show();
    ui->loginButton->setText("Login");
}

bool LoginDialog::event(QEvent *e)
{
    if (e->type() == QEvent::LayoutRequest) {
        setFixedSize(sizeHint());
    }
    return QDialog::event(e);
}
