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
#include <QNetworkAccessManager>
#include <QNetworkReply>
#include <QNetworkRequest>
#include <QNetworkCookie>
#include <QNetworkCookieJar>
#include <QNetworkProxyFactory>
#include <QPushButton>
#include <QRegularExpression>
#include <QSettings>
#include <QSysInfo>
#include <QUrl>
#include <QUrlQuery>

#include "QsLog.h"
#include "rapidjson/error/en.h"

#include "crashpad.h"
#include "filesystem.h"
#include "mainwindow.h"
#include "network_info.h"
#include "replytimeout.h"
#include "util.h"
#include "updatechecker.h"
#include "oauthmanager.h"
#include "version_defines.h"

constexpr const char* POE_LEAGUE_LIST_URL = "https://api.pathofexile.com/leagues?type=main&compact=1";
constexpr const char* POE_MAIN_PAGE = "https://www.pathofexile.com/";
constexpr const char* POE_MY_ACCOUNT = "https://www.pathofexile.com/my-account";
constexpr const char* POE_LOGIN_CHECK_URL = POE_MY_ACCOUNT;

constexpr int CLOUDFLARE_RATE_LIMITED = 1015;

constexpr const char* OAUTH_TAB = "oauthTab";
constexpr const char* SESSIONID_TAB = "sessionIdTab";

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

LoginDialog::LoginDialog(
    QSettings& settings,
    QNetworkAccessManager& network_manager,
    OAuthManager& oauth_manager)
    :
    QDialog(nullptr),
    settings_(settings),
    network_manager_(network_manager),
    oauth_manager_(oauth_manager),
    ui(new Ui::LoginDialog)
{
    // Setup the dialog box.
    QLOG_TRACE() << "LoginDialog::LoginDialog() calling UI setup";
    ui->setupUi(this);

    // Set window properties.
    QLOG_TRACE() << "LoginDialog::LoginDialog() setting window properties";
    setWindowTitle(QString("Acquisition Login [") + APP_VERSION_STRING + "]");
    setWindowIcon(QIcon(":/icons/assets/icon.svg"));
    setWindowFlags(windowFlags() & ~Qt::WindowContextHelpButtonHint);

    // Hide the error message label by default.
    QLOG_TRACE() << "LoginDialog::LoginDialog() hiding the error message label";
    ui->errorLabel->hide();
    ui->errorLabel->setStyleSheet("QLabel { color : red; }");

    // Disable the login button until we are ready to login.
    QLOG_TRACE() << "LoginDialog::LoginDialog() disabling the login button";
    ui->loginButton->setEnabled(false);

    // Connect UI signals.
    connect(ui->sessionIDLineEdit, &QLineEdit::textChanged, this, &LoginDialog::OnSessionIDChanged);
    connect(ui->rememberMeCheckBox, &QCheckBox::clicked, this, &LoginDialog::OnRememberMeCheckBoxClicked);
    connect(ui->proxyCheckBox, &QCheckBox::clicked, this, &LoginDialog::OnProxyCheckBoxClicked);
    connect(ui->reportCrashesCheckBox, &QCheckBox::clicked, this, &LoginDialog::OnReportCrashesCheckBoxClicked);
    connect(ui->loginButton, &QPushButton::clicked, this, &LoginDialog::OnLoginButtonClicked);
    connect(ui->authenticateButton, &QPushButton::clicked, this, &LoginDialog::OnAuthenticateButtonClicked);
    connect(ui->loginTabs, &QTabWidget::currentChanged, this, &LoginDialog::OnLoginTabChanged);

    // Listen for access from the OAuth manager.
    connect(&oauth_manager_, &OAuthManager::accessGranted, this, &LoginDialog::OnOAuthAccessGranted);

    // Load saved settings.
    QLOG_TRACE() << "LoginDialog::LoginDialog() calling LoadSettings()";
    LoadSettings();

    // Load the OAuth token if one is already present.
    if (oauth_manager_.token().isValid()) {
        QLOG_TRACE() << "LoginDialog::LoginDialog() found a valid OAuth token";
        OnOAuthAccessGranted(oauth_manager_.token());
    };

    // Request the list of leagues.
    QLOG_TRACE() << "LoginDialog::LoginDialog() requesting leagues";
    RequestLeagues();
}

LoginDialog::~LoginDialog() {
    QLOG_TRACE() << "LoginDialog::~LoginDialog() destroying object";
    SaveSettings();
    delete ui;
}

void LoginDialog::LoadSettings() {

    QLOG_TRACE() << "LoginDialog::LoadSetting()";

    const QString session_id = settings_.value("session_id").toString();
    QLOG_TRACE() << "LoginDialog::LoadSettings() session_id has" << session_id.size() << "characters";

    const QString realm = settings_.value("realm").toString();
    QLOG_TRACE() << "LoginDialog::LoadSettings() realm =" << realm;

    const QString league = settings_.value("league").toString();
    QLOG_TRACE() << "LoginDialog::LoadSettings() league =" << league;

    const int login_tab = settings_.value("login_tab").toInt();
    QLOG_TRACE() << "LoginDialog::LoadSettings() login_tab =" << login_tab;

    const bool remember_me = settings_.value("remember_user").toBool();
    QLOG_TRACE() << "LoginDialog::LoadSettings() remember_me" << remember_me;

    const bool use_system_proxy = settings_.value("use_system_proxy").toBool();
    QLOG_TRACE() << "LoginDialog::LoadSettings() use_system_proxy" << use_system_proxy;

    const bool report_crashes = settings_.value("report_crashes").toBool();
    QLOG_TRACE() << "LoginDialog::LoadSettings() report_crashes" << report_crashes;

    QLOG_TRACE() << "LoginDialog::LoadSettings() passing remember_me to OAuth manager";
    oauth_manager_.RememberToken(remember_me);

    QLOG_TRACE() << "LoginDialog::LoadSettings() setting UI elements";
    ui->sessionIDLineEdit->setText(session_id);
    ui->rememberMeCheckBox->setChecked(remember_me);
    ui->proxyCheckBox->setChecked(use_system_proxy);
    ui->reportCrashesCheckBox->setChecked(report_crashes);
    ui->loginTabs->setCurrentIndex(login_tab);
    if (!realm.isEmpty()) {
        ui->realmComboBox->setCurrentText(realm);
    };
    if (!league.isEmpty()) {
        ui->leagueComboBox->setCurrentText(league);
    };

    QLOG_TRACE() << "LoginDialog::LoadSetting() setting proxy configuration";
    QNetworkProxyFactory::setUseSystemConfiguration(ui->proxyCheckBox->isChecked());
}

void LoginDialog::SaveSettings() {
    if (!ui->rememberMeCheckBox->isChecked()) {
        QLOG_TRACE() << "LoginDialog::SaveSettings() removing settings";
        settings_.remove("session_id");
        settings_.remove("account");
        settings_.remove("realm");
        settings_.remove("league");
        settings_.remove("login_tab");
        settings_.remove("remember_user");
        settings_.remove("use_system_proxy");
        settings_.remove("report_crashes");
    };
}

void LoginDialog::RequestLeagues() {

    // Make a non-API request to get the list of leagues. This currently uses
    // a legacy endpoint that is not rate limited and does not require authentication.
    QNetworkRequest request = QNetworkRequest(QUrl(QString(POE_LEAGUE_LIST_URL)));
    request.setHeader(QNetworkRequest::KnownHeaders::UserAgentHeader, USER_AGENT);
    request.setTransferTimeout(kPoeApiTimeout);

    // Send the request and handle errors.
    QLOG_TRACE() << "LoginDialog::RequestLeagues() sending request:" << request.url().toString();
    QNetworkReply* reply = network_manager_.get(request);
    connect(reply, &QNetworkReply::finished, this, &LoginDialog::OnLeaguesReceived);
    connect(reply, &QNetworkReply::errorOccurred, this,
        [=](QNetworkReply::NetworkError code) {
            Q_UNUSED(code);
            DisplayError("Error requesting leagues: " + reply->errorString(), true);
        });
    connect(reply, &QNetworkReply::sslErrors, this,
        [=](const QList<QSslError>& errors) {
            for (const auto& error : errors) {
                QLOG_ERROR() << "SSL Error requesting leagues:" << error.errorString();
            };
            DisplayError("SSL error fetching leagues", true);
        });
}

void LoginDialog::OnLeaguesReceived() {

    QLOG_TRACE() << "LoginDialog::OnLeaguesReceived() reply recieved";
    QNetworkReply* reply = qobject_cast<QNetworkReply*>(QObject::sender());
    const QByteArray bytes = reply->readAll();
    reply->deleteLater();

    // Check for network errors.
    if (reply->error()) {
        QLOG_TRACE() << "LoginDialog::OnLeaguesReceived() reply error" << reply->error();
        return LeaguesRequestError(reply->errorString(), bytes);
    };

    rapidjson::Document doc;
    doc.Parse(bytes.constData());

    // Check the document for basic error.
    if (doc.HasParseError()) {
        const QString parse_error = rapidjson::GetParseError_En(doc.GetParseError());
        return LeaguesRequestError("json error: " + parse_error, bytes);
    };
    if (!doc.IsArray()) {
        return LeaguesRequestError("object is not an array", bytes);
    };

    QStringList leagues;

    // Parse leagues from the json object.
    for (auto& league : doc) {
        if (!league.IsObject()) {
            return LeaguesRequestError("object expected", bytes);
        };
        if (!league.HasMember("id")) {
            return LeaguesRequestError("missing league 'id'", bytes);
        };
        if (!league["id"].IsString()) {
            return LeaguesRequestError("league 'id' is not a string", bytes);
        };
        leagues.append(league["id"].GetString());
    };

    // Get the league from settings.ini
    const QString saved_league = settings_.value("league").toString();
    QLOG_TRACE() << "LoginDialog::OnLeaguesReceived() loaded leage from settings:" << saved_league;

    bool use_saved_league = false;

    ui->leagueComboBox->clear();
    for (auto& league : doc) {

        // Make sure the league object is well-formed.
        if (!league.IsObject()) {
            return LeaguesRequestError("object expected", bytes);
        };
        if (!league.HasMember("id")) {
            return LeaguesRequestError("missing league 'id'", bytes);
        };
        if (!league["id"].IsString()) {
            return LeaguesRequestError("league 'id' is not a string", bytes);
        };

        // Get the league name.
        const QString league_name = league["id"].GetString();
        QLOG_TRACE() << "LoginDialog::OnLeaguesReceived() found league" << league_name;

        // Add the league to the combo box.
        ui->leagueComboBox->addItem(league_name);

        // Set the current league if it matches the saved league.
        if (0 == saved_league.compare(league_name, Qt::CaseInsensitive)) {
            use_saved_league = true;
        };
    };
    ui->leagueComboBox->setEnabled(true);

    // If we found a match for the save league use it. If we didn't, then
    // we need to clear the setting, since the list of leagues may have
    // changed since the last time acquisition was run.
    if (use_saved_league) {
        QLOG_TRACE() << "LoginDialog::OnLeaguesReceived() setting current league to" << saved_league;
        ui->leagueComboBox->setCurrentText(saved_league);
    } else {
        QLOG_TRACE() << "LoginDialog::OnLeaguesReceived() clearing the saved league";
        settings_.setValue("league", "");
    };

    // Now that leagues have been received, start listening for changes.
    connect(ui->leagueComboBox, &QComboBox::currentTextChanged, this, &LoginDialog::OnLeagueChanged);

    // Now we can let the user login.
    ui->loginButton->setEnabled(true);
}

void LoginDialog::LeaguesRequestError(const QString& error, const QByteArray& reply) {
    QLOG_ERROR() << "League reply was:" << reply;
    DisplayError("Error requesting leagues: " + error, true);
}

void LoginDialog::OnAuthenticateButtonClicked() {
    QLOG_TRACE() << "LoginDialog::OnAuthenticateButtonClicked() entered";
    ui->errorLabel->hide();
    ui->errorLabel->setText("");
    ui->authenticateButton->setEnabled(false);
    ui->authenticateButton->setText("Authenticating...");
    oauth_manager_.requestAccess();
}

void LoginDialog::OnLoginButtonClicked() {
    QLOG_TRACE() << "LoginDialog::OnLoginButtonClicked() entered";
    ui->errorLabel->hide();
    ui->errorLabel->setText("");
    ui->loginButton->setEnabled(false);
    ui->loginButton->setText("Logging in...");

    const QString realm = ui->realmComboBox->currentText();
    const QString league = ui->leagueComboBox->currentText();
    const QString session_id = ui->sessionIDLineEdit->text();
    settings_.setValue("realm", realm);
    settings_.setValue("league", league);
    settings_.setValue("session_id", session_id);
    if (!session_id.isEmpty()) {
        QNetworkCookie poesessid(POE_COOKIE_NAME, session_id.toUtf8());
        poesessid.setPath(POE_COOKIE_PATH);
        poesessid.setDomain(POE_COOKIE_DOMAIN);
        network_manager_.cookieJar()->insertCookie(poesessid);
    };

    const QString tab_name = ui->loginTabs->currentWidget()->objectName();
    if (tab_name == OAUTH_TAB) {
        LoginWithOAuth();
    } else if (tab_name == SESSIONID_TAB) {
        if (!session_id.isEmpty()) {
            LoginWithSessionID();
        } else {
            QLOG_ERROR() << "POESESSID is empty";
            DisplayError("POESESSID cannot be blank");
            ui->loginButton->setEnabled(true);
            ui->loginButton->setText("Log in");
        };
    } else {
        DisplayError("Invalid tab selected: " + tab_name);
    };
}

void LoginDialog::LoginWithOAuth() {
    QLOG_INFO() << "Starting OAuth authentication";
    if (oauth_manager_.token().isValid()) {
        const OAuthToken& token = oauth_manager_.token();
        const QString account = QString::fromStdString(token.username());
        settings_.setValue("account", account);
        emit LoginComplete(POE_API::OAUTH);
    } else {
        DisplayError("You are not authenticated.");
    };
}

void LoginDialog::LoginWithSessionID() {
    QLOG_INFO() << "Starting legacy login with POESESSID";
    QNetworkRequest request = QNetworkRequest(QUrl(POE_LOGIN_CHECK_URL));
    request.setHeader(QNetworkRequest::KnownHeaders::UserAgentHeader, USER_AGENT);
    QNetworkReply* reply = network_manager_.get(request);

    connect(reply, &QNetworkReply::finished, this, &LoginDialog::OnStartLegacyLogin);
    connect(reply, &QNetworkReply::errorOccurred, this,
        [=](QNetworkReply::NetworkError code) {
            const int error_code = static_cast<int>(code);
            if (error_code == CLOUDFLARE_RATE_LIMITED) {
                DisplayError("Rate limited by Cloudflare! Please report to gerwaric@gmail.com");
            } else {
                DisplayError("Error during legacy login: " + reply->errorString(), true);
            };
        });
    connect(reply, &QNetworkReply::sslErrors, this,
        [=](const QList<QSslError>& errors) {
            for (const auto& error : errors) {
                QLOG_ERROR() << "SSL error during legacy login:" << error.errorString();
            };
            DisplayError("SSL error during session id login", true);
        });
}

// Need a separate check since it's just the /login URL that's filtered
void LoginDialog::OnStartLegacyLogin() {
    QLOG_TRACE() << "LoginDialog::OnStartLegacyLogin() entered";

    QNetworkReply* reply = qobject_cast<QNetworkReply*>(QObject::sender());
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
            msg = QString("Network error %1 during legacy login: %2").arg(
                QString::number(reply->error()),
                reply->errorString());
        };
        DisplayError(msg);
        return;
    };

    // Check the session id cookie.
    const QString session_id = settings_.value("session_id").toString();
    for (const QNetworkCookie& cookie : cookies) {
        if (QString(cookie.name()) == POE_COOKIE_NAME) {
            if (cookie.value() != session_id) {
                QLOG_WARN() << "POESESSID mismatch";
            };
            break;
        };
    };

    // we need one more request to get account name
    QNetworkRequest request = QNetworkRequest(QUrl(POE_MY_ACCOUNT));
    request.setHeader(QNetworkRequest::KnownHeaders::UserAgentHeader, USER_AGENT);
    QNetworkReply* next_reply = network_manager_.get(request);

    connect(next_reply, &QNetworkReply::finished, this, &LoginDialog::OnFinishLegacyLogin);
    connect(reply, &QNetworkReply::errorOccurred, this,
        [=](QNetworkReply::NetworkError code) {
            const int error_code = static_cast<int>(code);
            if (error_code == CLOUDFLARE_RATE_LIMITED) {
                DisplayError("Blocked by Cloudflare! Please tell gerwaric@gmail.com. You may need to contact GGG support :-(");
            } else {
                DisplayError("Error finishing legacy login: " + reply->errorString(), true);
            };
        });
    connect(reply, &QNetworkReply::sslErrors, this,
        [=](const QList<QSslError>& errors) {
            for (const auto& error : errors) {
                QLOG_ERROR() << "SSL finishing legacy login:" << error.errorString();
            };
            DisplayError("SSL error finishing legacy login", true);
        });
}

void LoginDialog::OnFinishLegacyLogin() {
    QLOG_TRACE() << "LoginDialog::OnFinishLegacyLogin() entered";

    QNetworkReply* reply = qobject_cast<QNetworkReply*>(QObject::sender());
    const QString html(reply->readAll());
    reply->deleteLater();
    if (reply->error()) {
        DisplayError("Network error finishing legacy loging " + reply->errorString());
        return;
    };

    static const QRegularExpression regexp("/account/view-profile/.*?>(.*?)<");
    QRegularExpressionMatch match = regexp.match(html, 0);
    if (match.hasMatch() == false) {
        DisplayError("Failed to find account name.");
        return;
    };

    const QString account = match.captured(1);
    const QString realm = settings_.value("realm").toString();
    const QString league = settings_.value("league").toString();
    settings_.setValue("account", account);

    QLOG_DEBUG() << "Logged in as" << account << "to" << league << "league in" << realm << "realm";

    emit LoginComplete(POE_API::LEGACY);
}

void LoginDialog::OnOAuthAccessGranted(const OAuthToken& token) {
    QLOG_TRACE() << "LoginDialog::OnOAuthAccessGranted() entered";
    const QString username = QString::fromStdString(token.username());
    const QString expiration = token.expiration().toString();
    ui->authenticateLabel->setText("You are authenticated as \"" + username + "\" until " + expiration);
    ui->authenticateButton->setText("Re-authenticate (as someone else).");
    ui->authenticateButton->setEnabled(true);
}

void LoginDialog::OnLoginTabChanged(int index) {
    QLOG_TRACE() << "LoginDialog::OnLoginTabChanged() entered";
    settings_.setValue("login_tab", index);
};

void LoginDialog::OnSessionIDChanged(const QString& session_id) {
    QLOG_TRACE() << "LoginDialog::OnSessionIDChanged() entered";
    settings_.setValue("session_id", session_id);
}

void LoginDialog::OnLeagueChanged(const QString& league) {
    QLOG_TRACE() << "LoginDialog::OnLeagueChanged() entered";
    settings_.setValue("league", league);
}

void LoginDialog::OnProxyCheckBoxClicked(bool checked) {
    QLOG_TRACE() << "LoginDialog::OnProxyCheckBoxClicked() entered";
    QNetworkProxyFactory::setUseSystemConfiguration(checked);
    settings_.setValue("use_system_proxy", checked);
}

void LoginDialog::OnRememberMeCheckBoxClicked(bool checked) {
    QLOG_TRACE() << "LoginDialog::OnRememberMeCheckBoxClicked() entered";
    oauth_manager_.RememberToken(checked);
    settings_.setValue("remember_user", checked);
}

void LoginDialog::OnReportCrashesCheckBoxClicked(bool checked) {

    QLOG_TRACE() << "LoginDialog::OnCrashReportCheckBoxClicked() entered";
    QMessageBox msgbox(this);
    msgbox.setWindowTitle("Acquisition Crash Reporting");

    if (checked) {

        // Before enabling crash reporting, make sure the user
        // understands and accepts that crash reporting cannot be
        // disabled without restarting acquistion.
        msgbox.setText("Once crash reporting is enabled, it cannot be "
            "disabled without restarting Acquistion.\n\nDo you want to "
            "enable crash reporting?");
        auto yes = msgbox.addButton("  Yes, enable crash reporting  ", msgbox.YesRole);
        msgbox.addButton("  No  ", msgbox.NoRole);
        msgbox.exec();
        if (msgbox.clickedButton() == yes) {
            settings_.setValue("report_crashes", true);
            initializeCrashpad(Filesystem::UserDir(), APP_PUBLISHER, APP_NAME, APP_VERSION_STRING);
        } else {
            settings_.setValue("report_crashes", false);
            ui->reportCrashesCheckBox->setChecked(false);
        };

    } else {

        // Since crashpad cannot be stopped once it is started, acquistion
        // will have to be exited and restarted to disable crash reporting,
        // so make sure the user accepts and agrees to this.
        msgbox.setText("Acquisition will have to restart to disable crash "
            "reporting.\n\nDo you want Acquisition to exit now and disable "
            "crash reporting the next time it runs?");
        auto yes = msgbox.addButton("  Yes, exit now  ", msgbox.YesRole);
        msgbox.addButton("  No, continue running  ", msgbox.NoRole);
        msgbox.exec();
        if (msgbox.clickedButton() == yes) {
            settings_.setValue("report_crashes", false);
            close();
        } else {
            settings_.setValue("report_crashes", true);
            ui->reportCrashesCheckBox->setChecked(true);
            initializeCrashpad(Filesystem::UserDir(), APP_PUBLISHER, APP_NAME, APP_VERSION_STRING);
        };

    };
}

void LoginDialog::DisplayError(const QString& error, bool disable_login) {
    QLOG_ERROR() << "LoginDialog:" << error;
    ui->errorLabel->setText(error);
    ui->errorLabel->show();
    ui->loginButton->setEnabled(!disable_login);
    ui->loginButton->setText("Login");
}

bool LoginDialog::event(QEvent* e) {
    if (e->type() == QEvent::LayoutRequest) {
        setFixedSize(sizeHint());
    };
    return QDialog::event(e);
}
