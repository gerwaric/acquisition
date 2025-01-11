/*
    Copyright (C) 2014-2024 Acquisition Contributors

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

#include "application.h"

#include <QAction>
#include <QApplication>
#include <QDir>
#include <QFileDialog>
#include <QMessageBox>
#include <QNetworkAccessManager>
#include <QNetworkCookieJar>
#include <QSettings>
#include <QtHttpServer/QHttpServer>

#include <QsLog/QsLog.h>

#include "datastore/sqlitedatastore.h"
#include "legacy/legacybuyoutvalidator.h"
#include "ratelimit/ratelimiter.h"
#include "ratelimit/ratelimitmanager.h"
#include "ui/logindialog.h"
#include "ui/mainwindow.h"
#include "util/crashpad.h"
#include "util/fatalerror.h"
#include "util/oauthmanager.h"
#include "util/repoe.h"
#include "util/updatechecker.h"

#include "buyoutmanager.h"
#include "currencymanager.h"
#include "imagecache.h"
#include "itemsmanager.h"
#include "network_info.h"
#include "shop.h"
#include "testmain.h"
#include "version_defines.h"

Application::Application(const QDir& appDataDir) {
    QLOG_TRACE() << "Application::Application() entered";

    QLOG_TRACE() << "Application::Application() initializing crashpad";
    InitCrashReporting();

    QLOG_TRACE() << "Application::Application() creating QNetworkAccessManager";
    m_network_manager = std::make_unique<QNetworkAccessManager>();

    QLOG_TRACE() << "Application::Application() creating RePoE";
    m_repoe = std::make_unique<RePoE>(network_manager());

    InitUserDir(appDataDir.absolutePath());
}

void Application::InitUserDir(const QString& dir) {

    m_data_dir = QDir(dir);
    QLOG_TRACE() << "Application::Application() data directory is" << m_data_dir.absolutePath();

    const QString settings_path = m_data_dir.filePath("settings.ini");
    QLOG_TRACE() << "Application::Application() creating the settings object:" << settings_path;
    m_settings = std::make_unique<QSettings>(settings_path, QSettings::IniFormat);

    const QDir user_dir(m_data_dir.filePath("data"));
    const QString global_data_file = user_dir.filePath(SqliteDataStore::MakeFilename("", ""));
    QLOG_TRACE() << "Application::Application() opening global data file:" << global_data_file;
    m_global_data = std::make_unique<SqliteDataStore>(global_data_file);

    const QString image_cache_dir = dir + QDir::separator() + "cache";
    m_image_cache = std::make_unique<ImageCache>(network_manager(), image_cache_dir);

    QLOG_TRACE() << "Application::Application() loading theme";
    const QString theme = settings().value("theme", "default").toString();
    SetTheme(theme);

    if (settings().value("realm").toString().isEmpty()) {
        settings().setValue("realm", "pc");
    };
    QLOG_TRACE() << "Application::Application() realm is" << settings().value("realm");

    QLOG_TRACE() << "Application::Application() creating update checker";
    m_update_checker = std::make_unique<UpdateChecker>(settings(), network_manager());

    QLOG_TRACE() << "Application::Application() creating OAuth manager";
    m_oauth_manager = std::make_unique<OAuthManager>(network_manager(), global_data());

    // Start the process of fetching RePoE data.
    QLOG_TRACE() << "Application::Application() initializing RePoE";
    m_repoe->Init(dir);
}

Application::~Application() {}

void Application::Start() {

    QLOG_TRACE() << "Application::Start() entered";

    QLOG_TRACE() << "Application::Start() creating login dialog";
    m_login = std::make_unique<LoginDialog>(
        m_data_dir,
        settings(),
        network_manager(),
        oauth_manager());

    // Connect to the update signal in case an update is detected before the main window is open.
    connect(m_update_checker.get(), &UpdateChecker::UpdateAvailable, m_update_checker.get(), &UpdateChecker::AskUserToUpdate);

    // Connect signals from the login dialog.
    connect(m_login.get(), &LoginDialog::ChangeTheme, this, &Application::SetTheme);
    connect(m_login.get(), &LoginDialog::ChangeUserDir, this, &Application::SetUserDir);
    connect(m_login.get(), &LoginDialog::LoginComplete, this, &Application::OnLogin);

    // Start the initial check for updates.
    QLOG_TRACE() << "Application::Start() starting a check for application updates";
    m_update_checker->CheckForUpdates();

    // Show the login dialog now.
    QLOG_TRACE() << "Application::Start() showing the login dialog";
    m_login->show();
}

void Application::Stop() {
    // Delete things in the reverse order they were created, just in case
    // we might otherwise get an invalid point or reference.
    m_login = nullptr;
    m_oauth_manager = nullptr;
    m_update_checker = nullptr;
    m_global_data = nullptr;
    m_settings = nullptr;
}

void Application::OnLogin(POE_API api) {

    QLOG_TRACE() << "Application::OnLogin() entered";

    // Disconnect from the update signal so that only the main window gets it from now on.
    QObject::disconnect(&update_checker(), &UpdateChecker::UpdateAvailable, nullptr, nullptr);

    // Call init login to setup the shop, items manager, and other objects.
    QLOG_TRACE() << "Application::OnLogin() initializing login";
    InitLogin(api);

    // Prepare to show the main window now that everything is initialized.
    QLOG_TRACE() << "Application::OnLogin() creating main window";
    m_main_window = std::make_unique<MainWindow>(
        settings(),
        network_manager(),
        rate_limiter(),
        data(),
        items_manager(),
        buyout_manager(),
        shop(),
        image_cache());

    // Connect UI signals.
    connect(m_main_window.get(), &MainWindow::SetSessionId, this, &Application::SetSessionId);
    connect(m_main_window.get(), &MainWindow::SetTheme, this, &Application::SetTheme);
    connect(m_main_window.get(), &MainWindow::UpdateCheckRequested, m_update_checker.get(), &UpdateChecker::CheckForUpdates);

    connect(m_items_manager.get(), &ItemsManager::ItemsRefreshed, m_main_window.get(), &MainWindow::OnItemsRefreshed);
    connect(m_items_manager.get(), &ItemsManager::StatusUpdate, m_main_window.get(), &MainWindow::OnStatusUpdate);

    connect(m_main_window.get(), &MainWindow::GetImage, m_image_cache.get(), &ImageCache::fetch);
    connect(m_image_cache.get(), &ImageCache::imageReady, m_main_window.get(), &MainWindow::OnImageFetched);

    connect(m_shop.get(), &Shop::StatusUpdate, m_main_window.get(), &MainWindow::OnStatusUpdate);

    m_main_window->prepare(
        *m_oauth_manager,
        *m_currency_manager,
        *m_shop);

    // Connect the update checker.
    connect(m_update_checker.get(), &UpdateChecker::UpdateAvailable, m_main_window.get(), &MainWindow::OnUpdateAvailable);

    QLOG_TRACE() << "Application::OnLogin() closing the login dialog";
    m_login->close();

    QLOG_TRACE() << "Application::OnLogin() showing the main window";
    m_main_window->show();
}

QSettings& Application::settings() const {
    if (!m_settings) {
        FatalError("Application::settings() attempted to dereference a null pointer");
    };
    return *m_settings;
};

ItemsManager& Application::items_manager() const {
    if (!m_items_manager) {
        FatalError("Application::items_manager() attempted to dereference a null pointer");
    };
    return *m_items_manager;
};

DataStore& Application::global_data() const {
    if (!m_global_data) {
        FatalError("Application::global_data() attempted to dereference a null pointer");
    };
    return *m_global_data;
};

DataStore& Application::data() const {
    if (!m_data) {
        FatalError("Application::data() attempted to dereference a null pointer");
    };
    return *m_data;
};

BuyoutManager& Application::buyout_manager() const {
    if (!m_buyout_manager) {
        FatalError("Application::buyout_manager() attempted to dereference a null pointer");
    };
    return *m_buyout_manager;
};

QNetworkAccessManager& Application::network_manager() const {
    if (!m_network_manager) {
        FatalError("Application::network_manager() attempted to dereference a null pointer");
    };
    return *m_network_manager;
}

RePoE& Application::repoe() const {
    if (!m_repoe) {
        FatalError("Application::repoe() attempted to dereference a null pointer");
    };
    return *m_repoe;
}

Shop& Application::shop() const {
    if (!m_shop) {
        FatalError("Application::shop() attempted to dereference a null pointer");
    };
    return *m_shop;
}

ImageCache& Application::image_cache() const {
    if (!m_image_cache) {
        FatalError("Application::m_image_cache() attempted to dereference a null pointer");
    };
    return *m_image_cache;
}


CurrencyManager& Application::currency_manager() const {
    if (!m_currency_manager) {
        FatalError("Application::currency_manager() attempted to dereference a null pointer");
    };
    return *m_currency_manager;
}

UpdateChecker& Application::update_checker() const {
    if (!m_update_checker) {
        FatalError("Application::update_checker() attempted to dereference a null pointer");
    };
    return *m_update_checker;
}

OAuthManager& Application::oauth_manager() const {
    if (!m_oauth_manager) {
        FatalError("Application::oauth_manager() attempted to dereference a null pointer");
    };
    return *m_oauth_manager;
}

RateLimiter& Application::rate_limiter() const {
    if (!m_rate_limiter) {
        FatalError("Application::rate_limiter() attempted to dereference a null pointer");
    };
    return *m_rate_limiter;
}

void Application::InitCrashReporting() {
    QLOG_TRACE() << "Application::InitCrashReporting() entered";

    // Make sure the settings object exists.
    if (!m_settings) {
        QLOG_ERROR() << "Cannot init crash reporting because settings object is invalid";
        return;
    };

    // Enable crash reporting by default.
    bool report_crashes = true;

    if (!m_settings->contains("report_crashes")) {
        // Update the setting if it didn't exist before.
        QLOG_TRACE() << "Application::InitCrashReporting() setting 'report_crashes' to true";
        m_settings->setValue("report_crashes", true);
    } else {
        // Use the exiting setting.
        report_crashes = m_settings->value("report_crashes").toBool();
        QLOG_TRACE() << "Application::InitCrashReporting() 'report_crashes' is" << report_crashes;
    };

    // Initialize crash reporting with crashpad.
    if (report_crashes) {
        QLOG_TRACE() << "Application::InitCrashReporting() initializing crashpad";
        initializeCrashpad(m_data_dir.absolutePath(), APP_PUBLISHER, APP_NAME, APP_VERSION_STRING);
    };
}

void Application::SetSessionId(const QString& poesessid) {
    if (poesessid.isEmpty()) {
        QLOG_ERROR() << "Application: cannot update POESESSID: value is empty";
        return;
    };
    QLOG_INFO() << "Application: updating POESESSID";
    QNetworkCookie cookie(POE_COOKIE_NAME, poesessid.toUtf8());
    cookie.setPath(POE_COOKIE_PATH);
    cookie.setDomain(POE_COOKIE_DOMAIN);
    m_network_manager->cookieJar()->insertCookie(cookie);
    m_settings->setValue("session_id", poesessid);
}

void Application::SetTheme(const QString& theme) {
    QLOG_TRACE() << "Application::OnSetTheme() entered";

    if (0 == theme.compare(m_active_theme, Qt::CaseInsensitive)) {
        QLOG_DEBUG() << "Theme is already set:" << theme;
        return;
    };

    QString stylesheet;
    QColor text_color;

    if (0 == theme.compare("default", Qt::CaseInsensitive)) {
        stylesheet.clear();
        text_color = Qt::black;
    } else if (0 == theme.compare("light", Qt::CaseInsensitive)) {
        stylesheet = ":qdarkstyle/light/lightstyle.qss";
        text_color = Qt::black;
    } else if (0 == theme.compare("dark", Qt::CaseInsensitive)) {
        stylesheet = ":qdarkstyle/dark/darkstyle.qss";
        text_color = Qt::white;
    } else {
        QLOG_ERROR() << "Invalid theme:" << theme;
        return;
    };

    QLOG_TRACE() << "Application::OnSetTheme() setting theme:" << theme;
    settings().setValue("theme", theme);

    QString style_data;
    if (!stylesheet.isEmpty()) {
        QLOG_TRACE() << "Application::OnSetTheme() loading stylesheet:" << stylesheet;
        QFile f(stylesheet);
        if (f.exists()) {
            if (!f.open(QFile::ReadOnly | QFile::Text)) {
                QLOG_ERROR() << "Error loading stylesheet (" + stylesheet + "):" << f.errorString();
            } else {
                style_data = f.readAll();
                f.close();
            };
        } else {
            QLOG_ERROR() << "Style sheet not found:" << stylesheet;
        };
    };

    QLOG_TRACE() << "Application::OnSetTheme() setting stylesheet";
    qApp->setStyleSheet(style_data);
}

void Application::SetUserDir(const QString& dir) {
    Stop();
    InitUserDir(dir);
    Start();
}

void Application::InitLogin(POE_API mode)
{
    QLOG_TRACE() << "Application::InitLogin() entered";

    const QString league = m_settings->value("league").toString();
    const QString account = m_settings->value("account").toString();
    const QDir user_dir(m_data_dir.filePath("data"));
    if (league.isEmpty()) {
        FatalError("Login failure: the league has not been set.");
    };
    if (account.isEmpty()) {
        FatalError("Login failure: the account has not been set.");
    };
    QLOG_TRACE() << "Application::InitLogin() league =" << league;
    QLOG_TRACE() << "Application::InitLogin() account =" << account;
    QLOG_TRACE() << "Application::InitLogin() data_dir =" << user_dir.absolutePath();
    const QString data_file = SqliteDataStore::MakeFilename(account, league);
    const QString data_path = user_dir.absoluteFilePath(data_file);
    QLOG_TRACE() << "Application::InitLogin() data_path =" << data_path;

    QLOG_INFO() << "Validating stored buyouts";
    LegacyBuyoutValidator legacy(data_path);
    //legacy.validate();

    m_data = std::make_unique<SqliteDataStore>(data_path);
    SaveDbOnNewVersion();

    QLOG_TRACE() << "Application::InitLogin() creating rate limiter";
    m_rate_limiter = std::make_unique<RateLimiter>(
        network_manager(),
        oauth_manager(), mode);

    QLOG_TRACE() << "Application::InitLogin() creating buyout manager";
    m_buyout_manager = std::make_unique<BuyoutManager>(
        data());

    QLOG_TRACE() << "Application::InitLogin() creating items manager";
    m_items_manager = std::make_unique<ItemsManager>(
        settings(),
        network_manager(),
        repoe(),
        buyout_manager(),
        data(),
        rate_limiter());

    QLOG_TRACE() << "Application::InitLogin() creating shop";
    m_shop = std::make_unique<Shop>(
        settings(),
        network_manager(),
        rate_limiter(),
        data(),
        items_manager(),
        buyout_manager());

    QLOG_TRACE() << "Application::InitLogin() creating currency manager";
    m_currency_manager = std::make_unique<CurrencyManager>(
        settings(),
        data(),
        items_manager());

    connect(m_items_manager.get(), &ItemsManager::ItemsRefreshed, this, &Application::OnItemsRefreshed);

    QLOG_TRACE() << "Application::InitLogin() starting items manager";
    m_items_manager->Start(mode);
}

void Application::OnItemsRefreshed(bool initial_refresh) {
    QLOG_TRACE() << "Application::OnItemsRefreshed() entered";
    QLOG_TRACE() << "Application::OnItemsRefreshed() initial_refresh =" << initial_refresh;
    m_currency_manager->Update();
    m_shop->Update();
    if (!initial_refresh && m_shop->auto_update()) {
        QLOG_TRACE() << "Application::OnItemsRefreshed() submitting shops";
        m_shop->SubmitShopToForum();
    };
}

void Application::OnRunTests() {
    QLOG_TRACE() << "Application::OnRunTests() entered";
    if (!m_repoe->IsInitialized()) {
        QLOG_TRACE() << "Application::OnRunTests() RePoE is not initialized";
        QMessageBox::information(nullptr,
            "Acquisition", "RePoE is not initialized yet. Try again later",
            QMessageBox::StandardButton::Ok,
            QMessageBox::StandardButton::Ok);
        return;
    }
    QLOG_TRACE() << "Application::OnRunTests() hiding login and running tests";
    m_login->hide();
    const int result = test_main(m_data_dir.absolutePath());
    QLOG_TRACE() << "Application::OnRunTests(): test_main returned" << result;
    QMessageBox::information(nullptr,
        "Acquisition", "Testing returned " + QString::number(result),
        QMessageBox::StandardButton::Ok,
        QMessageBox::StandardButton::Ok);
    m_login->show();
}

void Application::SaveDbOnNewVersion() {

    QLOG_TRACE() << "Application::SaveDbOnNewVersion() entered";
    //If user updated from a 0.5c db to a 0.5d, db exists but no "version" in it
    QString version = m_data->Get("version", "0.5c");
    // We call this just after login, so we didn't pulled tabs for the first time ; so "tabs" shouldn't exist in the DB
    // This way we don't create an useless data_save_version folder on the first time you run acquisition

    bool first_start = m_data->Get("tabs", "first_time") == "first_time" &&
        m_data->GetTabs(ItemLocationType::STASH).size() == 0 &&
        m_data->GetTabs(ItemLocationType::CHARACTER).size() == 0;
    QLOG_TRACE() << "Application::SaveDbOnNewVersion() first_start =" << first_start;

    if (version != APP_VERSION_STRING && !first_start) {
        const QString data_path = m_data_dir.filePath("data");
        const QString save_path = m_data_dir.filePath("m_data_save" + version);
        QLOG_TRACE() << "Application::SaveDbOnNewVersion() data_path =" << data_path;
        QLOG_TRACE() << "Application::SaveDbOnNewVersion() save_path =" << save_path;
        QDir src(data_path);
        QDir dst(save_path);
        if (!dst.exists()) {
            QLOG_TRACE() << "Application::SaveDbOnNewVersion() creating save_path";
            QDir().mkpath(dst.path());
        };
        for (const auto& name : src.entryList()) {
            const QString a = QDir(data_path).filePath(name);
            const QString b = QDir(save_path).filePath(name);
            QLOG_TRACE() << "Application::SaveDbOnNewVersion() copying" << a << "to" << b;
            QFile::copy(a, b);
        }
        QLOG_INFO() << "I've created the folder " << save_path << "in your acquisition folder, containing a save of all your data";
    }

    QLOG_TRACE() << "Application::SaveDbOnNewVersion() setting 'version' to" << APP_VERSION_STRING;
    m_data->Set("version", APP_VERSION_STRING);
}
