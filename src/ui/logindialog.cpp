// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

#include "ui/logindialog.h"
#include "ui_logindialog.h"

#include <QDesktopServices>
#include <QFile>
#include <QFileDialog>
#include <QFileInfo>
#include <QMessageBox>
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

#include "app/usersettings.h"
#include "poe/types/league.h"
#include "replytimeout.h"
#include "util/json_readers.h"
#include "util/networkmanager.h"
#include "util/oauthmanager.h"
#include "util/spdlog_qt.h" // IWYU pragma: keep
#include "util/updatechecker.h"
#include "util/util.h"
#include "version_defines.h"

#include "mainwindow.h"

constexpr const char *POE_LEAGUE_LIST_URL
    = "https://api.pathofexile.com/leagues?type=main&compact=1";

constexpr int CLOUDFLARE_RATE_LIMITED = 1015;

LoginDialog::LoginDialog(app::UserSettings &settings,
                         NetworkManager &network_manager,
                         OAuthManager &oauth_manager)
    : m_settings(settings)
    , m_network_manager(network_manager)
    , m_oauth_manager(oauth_manager)
    , ui(new Ui::LoginDialog)
{
    // Setup the dialog box.
    spdlog::trace("LoginDialog::LoginDialog() calling UI setup");
    ui->setupUi(this);

    // Set window properties.
    spdlog::trace("LoginDialog::LoginDialog() setting window properties");
    setWindowTitle(QString("Acquisition Login [") + APP_VERSION_STRING + "]");
    setWindowIcon(QIcon(":/icons/icon.svg"));
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
    ui->userDirButton->setText(m_settings.userDir().absolutePath());

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
        emit RemoveOAuthToken();
    }
    delete ui;
}

void LoginDialog::LoadSettings()
{
    ui->realmComboBox->setCurrentText(m_settings.realm());
    ui->leagueComboBox->setCurrentText(m_settings.league());
    ui->themeComboBox->setCurrentText(m_settings.theme());
    ui->advancedCheckBox->setChecked(m_settings.showStartupOptions());
    ui->rememberMeCheckBox->setChecked(m_settings.rememberUser());
    ui->proxyCheckBox->setChecked(m_settings.useSystemProxy());

    const spdlog::string_view_t sv = spdlog::level::to_string_view(spdlog::get_level());
    const QString logging_level = QString::fromUtf8(sv.data(), sv.size());
    ui->loggingLevelComboBox->setCurrentText(logging_level);
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
        LeaguesRequestError(reply->errorString(), bytes);
        return;
    }

    // Parse the leagues.
    const auto leagues = json::readLeagueList(bytes);
    if (!leagues) {
        return;
    }

    // Get the league from settings.ini
    const QString saved_league = m_settings.league();
    spdlog::trace("LoginDialog::OnLeaguesReceived() loaded leage from settings: {}", saved_league);

    bool use_saved_league = false;

    ui->leagueComboBox->clear();
    for (auto &league : *leagues) {
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
        m_settings.league.clear();
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
    m_settings.realm(realm);

    LoginWithOAuth();
}

void LoginDialog::LoginWithOAuth()
{
    spdlog::info("Starting OAuth authentication");
    const QDateTime now = QDateTime::currentDateTime();
    if (m_current_token.has_value()) {
        const OAuthToken &token = m_current_token.value();
        if (token.access_expiration && (now < *token.access_expiration)) {
            m_settings.username(token.username);
            emit LoginComplete();
        } else if (token.refresh_expiration && (now < *token.refresh_expiration)) {
            DisplayError("The OAuth token needs to be refreshed");
        } else {
            DisplayError("The OAuth token is not valid.");
        }
    } else {
        DisplayError("You are not authenticated.");
    }
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
    ui->loginButton->setEnabled(m_current_token.has_value());
}

void LoginDialog::OnLeagueChanged(const QString &league)
{
    spdlog::trace("LoginDialog::OnLeagueChanged() entered");
    m_settings.league(league);
}

void LoginDialog::OnAdvancedCheckBoxChanged(Qt::CheckState state)
{
    spdlog::trace("LoginDialog: advanced options checkbox changed to {}", state);
    const bool checked = (state == Qt::Checked);
    const bool hide_options = false;
    ui->advancedOptionsFrame->setHidden(!checked || hide_options);
    m_settings.showStartupOptions(checked);
}

void LoginDialog::OnProxyCheckBoxChanged(Qt::CheckState state)
{
    spdlog::trace("LoginDialog: proxy checkbox changed to {}", state);
    const bool checked = (state == Qt::Checked);
    QNetworkProxyFactory::setUseSystemConfiguration(checked);
    m_settings.useSystemProxy(checked);
}

void LoginDialog::OnRememberMeCheckBoxChanged(Qt::CheckState state)
{
    spdlog::trace("LoginDialog: remember me checkbox changed to {}", state);
    m_settings.rememberUser(state == Qt::Checked);
}

void LoginDialog::OnUserDirButtonPushed()
{
    // NOTE: TBD: re-implement this.
}

void LoginDialog::OnLoggingLevelChanged(const QString &level_name)
{
    const auto level = spdlog::level::from_str(level_name.toStdString());
    spdlog::set_level(level);
    m_settings.logLevel(level);
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
