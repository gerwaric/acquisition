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

#include "application.h"

#include <QAction>
#include <QApplication>
#include <QDir>
#include <QFileDialog>
#include <QMessageBox>
#include <QNetworkCookieJar>
#include <QSettings>

#include <datastore/sqlitedatastore.h>
#include <datastore/userstore.h>
#include <ratelimit/ratelimiter.h>
#include <ratelimit/ratelimitmanager.h>
#include <repoe/repoe.h>
#include <ui/logindialog.h>
#include <ui/mainwindow.h>
#include <util/fatalerror.h>
#include <util/networkmanager.h>
#include <util/oauthmanager.h>
#include <util/oauthtoken.h>
#include <util/spdlog_qt.h>
#include <util/updatechecker.h>

#include "buyoutmanager.h"
#include "currencymanager.h"
#include "imagecache.h"
#include "itemsmanager.h"
#include "itemsmanagerworker.h"
#include "network_info.h"
#include "shop.h"
#include "version_defines.h"

Application::Application()
{
    spdlog::debug("Application: created");

    spdlog::trace("Application: creating NetworkManager");
    m_network_manager = std::make_unique<NetworkManager>();

    spdlog::trace("Application: creating RePoE");
    m_repoe = std::make_unique<RePoE>(network_manager());
}

Application::~Application()
{
    spdlog::info("Shutting down.");
    spdlog::shutdown();
}

void Application::Start(const QDir &appDataDir)
{
    spdlog::debug("Application: starting");
    InitUserDir(appDataDir.absolutePath());

    spdlog::trace("Application: creating login dialog");
    m_login = std::make_unique<LoginDialog>(m_data_dir,
                                            settings(),
                                            network_manager(),
                                            oauth_manager(),
                                            global_data());

    // Connect to the update signal in case an update is detected before the main window is open.
    connect(m_update_checker.get(),
            &UpdateChecker::UpdateAvailable,
            m_update_checker.get(),
            &UpdateChecker::AskUserToUpdate);

    // Connect signals from the login dialog.
    connect(m_login.get(), &LoginDialog::ChangeTheme, this, &Application::SetTheme);
    connect(m_login.get(), &LoginDialog::ChangeUserDir, this, &Application::SetUserDir);
    connect(m_login.get(), &LoginDialog::LoginComplete, this, &Application::OnLogin);

    // Look for an initial oauth token.

    // Start the initial check for updates.
    spdlog::trace("Application: checking for application updates");
    m_update_checker->CheckForUpdates();

    // Show the login dialog now.
    spdlog::trace("Application: showing the login dialog");
    m_login->show();
}

void Application::InitUserDir(const QString &dir)
{
    m_data_dir = QDir(dir);
    spdlog::trace("Application: data directory is {}", m_data_dir.absolutePath());

    const QString settings_path = m_data_dir.filePath("settings.ini");
    spdlog::trace("Application: creating the settings object: {}", settings_path);
    m_settings = std::make_unique<QSettings>(settings_path, QSettings::IniFormat);

    SaveDataOnNewVersion();

    const QDir user_dir(m_data_dir.filePath("data"));
    const QString global_data_file = user_dir.filePath(SqliteDataStore::MakeFilename("", ""));
    spdlog::trace("Application: opening global data file: {}", global_data_file);
    m_global_data = std::make_unique<SqliteDataStore>(global_data_file);

    const QString image_cache_dir = dir + QDir::separator() + "cache";
    m_image_cache = std::make_unique<ImageCache>(network_manager(), image_cache_dir);

    spdlog::trace("Application: loading theme");
    const QString theme = settings().value("theme", "default").toString();
    SetTheme(theme);

    if (settings().value("realm").toString().isEmpty()) {
        settings().setValue("realm", "pc");
    };
    spdlog::trace("Application: realm is {}", settings().value("realm"));

    spdlog::trace("Application: creating update checker");
    m_update_checker = std::make_unique<UpdateChecker>(settings(), network_manager());

    spdlog::trace("Application: creating OAuth manager");
    m_oauth_manager = std::make_unique<OAuthManager>(network_manager(), global_data());

    // Start the process of fetching RePoE data.
    spdlog::trace("Application: initializing RePoE");
    m_repoe->Init(dir);
}

void Application::Stop()
{
    // Delete things in the reverse order they were created, just in case
    // we might otherwise get an invalid point or reference.
    m_login = nullptr;
    m_oauth_manager = nullptr;
    m_update_checker = nullptr;
    m_global_data = nullptr;
    m_settings = nullptr;
}

void Application::OnLogin()
{
    spdlog::debug("Application: login initiated");

    // Disconnect from the update signal so that only the main window gets it from now on.
    QObject::disconnect(&update_checker(), &UpdateChecker::UpdateAvailable, nullptr, nullptr);

    // Call init login to setup the shop, items manager, and other objects.
    spdlog::trace("Application: initializing login");
    InitLogin();

    // Prepare to show the main window now that everything is initialized.
    spdlog::trace("Application:creating main window");
    m_main_window = std::make_unique<MainWindow>(settings(),
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
    connect(m_main_window.get(),
            &MainWindow::UpdateCheckRequested,
            m_update_checker.get(),
            &UpdateChecker::CheckForUpdates);

    connect(m_items_manager.get(),
            &ItemsManager::ItemsRefreshed,
            m_main_window.get(),
            &MainWindow::OnItemsRefreshed);
    connect(m_items_manager.get(),
            &ItemsManager::StatusUpdate,
            m_main_window.get(),
            &MainWindow::OnStatusUpdate);

    connect(m_main_window.get(), &MainWindow::GetImage, m_image_cache.get(), &ImageCache::fetch);
    connect(m_image_cache.get(),
            &ImageCache::imageReady,
            m_main_window.get(),
            &MainWindow::OnImageFetched);

    connect(m_shop.get(), &Shop::StatusUpdate, m_main_window.get(), &MainWindow::OnStatusUpdate);

    m_main_window->prepare(*m_oauth_manager, *m_currency_manager);

    // Connect the update checker.
    connect(m_update_checker.get(),
            &UpdateChecker::UpdateAvailable,
            m_main_window.get(),
            &MainWindow::OnUpdateAvailable);

    spdlog::trace("Application::OnLogin() closing the login dialog");
    m_login->close();

    spdlog::trace("Application::OnLogin() showing the main window");
    m_main_window->show();
}

QSettings &Application::settings() const
{
    if (!m_settings) {
        FatalError("Application::settings() attempted to dereference a null pointer");
    }
    return *m_settings;
}

ItemsManager &Application::items_manager() const
{
    if (!m_items_manager) {
        FatalError("Application::items_manager() attempted to dereference a null pointer");
    }
    return *m_items_manager;
}

DataStore &Application::global_data() const
{
    if (!m_global_data) {
        FatalError("Application::global_data() attempted to dereference a null pointer");
    }
    return *m_global_data;
}

DataStore &Application::data() const
{
    if (!m_data) {
        FatalError("Application::data() attempted to dereference a null pointer");
    }
    return *m_data;
}

BuyoutManager &Application::buyout_manager() const
{
    if (!m_buyout_manager) {
        FatalError("Application::buyout_manager() attempted to dereference a null pointer");
    }
    return *m_buyout_manager;
}

NetworkManager &Application::network_manager() const
{
    if (!m_network_manager) {
        FatalError("Application::network_manager() attempted to dereference a null pointer");
    }
    return *m_network_manager;
}

RePoE &Application::repoe() const
{
    if (!m_repoe) {
        FatalError("Application::repoe() attempted to dereference a null pointer");
    }
    return *m_repoe;
}

Shop &Application::shop() const
{
    if (!m_shop) {
        FatalError("Application::shop() attempted to dereference a null pointer");
    }
    return *m_shop;
}

ImageCache &Application::image_cache() const
{
    if (!m_image_cache) {
        FatalError("Application::m_image_cache() attempted to dereference a null pointer");
    }
    return *m_image_cache;
}

CurrencyManager &Application::currency_manager() const
{
    if (!m_currency_manager) {
        FatalError("Application::currency_manager() attempted to dereference a null pointer");
    }
    return *m_currency_manager;
}

UpdateChecker &Application::update_checker() const
{
    if (!m_update_checker) {
        FatalError("Application::update_checker() attempted to dereference a null pointer");
    }
    return *m_update_checker;
}

OAuthManager &Application::oauth_manager() const
{
    if (!m_oauth_manager) {
        FatalError("Application::oauth_manager() attempted to dereference a null pointer");
    }
    return *m_oauth_manager;
}

RateLimiter &Application::rate_limiter() const
{
    if (!m_rate_limiter) {
        FatalError("Application::rate_limiter() attempted to dereference a null pointer");
    }
    return *m_rate_limiter;
}

void Application::InitCrashReporting()
{
    spdlog::trace("Application::InitCrashReporting() entered");

    // Make sure the settings object exists.
    if (!m_settings) {
        spdlog::error("Cannot init crash reporting because settings object is invalid");
        return;
    }

    // Enable crash reporting by default.
    bool report_crashes = true;

    if (!m_settings->contains("report_crashes")) {
        // Update the setting if it didn't exist before.
        spdlog::trace("Application::InitCrashReporting() setting 'report_crashes' to true");
        m_settings->setValue("report_crashes", true);
    } else {
        // Use the exiting setting.
        report_crashes = m_settings->value("report_crashes").toBool();
        spdlog::trace("Application::InitCrashReporting() 'report_crashes' is {}", report_crashes);
    }
}

void Application::SetSessionId(const QString &poesessid)
{
    if (poesessid.isEmpty()) {
        spdlog::error("Application: cannot update POESESSID: value is empty");
        return;
    }
    spdlog::info("Application: updating POESESSID");
    QNetworkCookie cookie(POE_COOKIE_NAME, poesessid.toUtf8());
    cookie.setPath(POE_COOKIE_PATH);
    cookie.setDomain(POE_COOKIE_DOMAIN);
    m_network_manager->cookieJar()->insertCookie(cookie);
    m_settings->setValue("session_id", poesessid);
}

void Application::SetTheme(const QString &theme)
{
    spdlog::trace("Application::OnSetTheme() entered");

    if (0 == theme.compare(m_active_theme, Qt::CaseInsensitive)) {
        spdlog::debug("Theme is already set: {}", theme);
        return;
    }

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
        spdlog::error("Invalid theme: {}", theme);
        return;
    }

    spdlog::trace("Application::OnSetTheme() setting theme: {}", theme);
    settings().setValue("theme", theme);

    QString style_data;
    if (!stylesheet.isEmpty()) {
        spdlog::trace("Application::OnSetTheme() loading stylesheet: {}", stylesheet);
        QFile f(stylesheet);
        if (f.exists()) {
            if (!f.open(QFile::ReadOnly | QFile::Text)) {
                spdlog::error("Error loading stylesheet ({}): {}", stylesheet, f.errorString());
            } else {
                style_data = f.readAll();
                f.close();
            }
        } else {
            spdlog::error("Style sheet not found: {}", stylesheet);
        }
    }

    spdlog::trace("Application::OnSetTheme() setting stylesheet");
    qApp->setStyleSheet(style_data);
}

void Application::SetUserDir(const QString &dir)
{
    Stop();
    Start(dir);
}

void Application::InitLogin()
{
    spdlog::trace("Application::InitLogin() entered");

    const QString league = m_settings->value("league").toString();
    const QString account = m_settings->value("account").toString();
    if (league.isEmpty()) {
        FatalError("Login failure: the league has not been set.");
    }
    if (account.isEmpty()) {
        FatalError("Login failure: the account has not been set.");
    }
    QDir data_dir(m_data_dir.filePath("data"));
    spdlog::trace("Application::InitLogin() league = {}", league);
    spdlog::trace("Application::InitLogin() account = {}", account);
    spdlog::trace("Application::InitLogin() data_dir = {}", data_dir.absolutePath());
    const QString data_file = SqliteDataStore::MakeFilename(account, league);
    const QString data_path = data_dir.absoluteFilePath(data_file);
    spdlog::trace("Application::InitLogin() data_path = {}", data_path);

    m_data = std::make_unique<SqliteDataStore>(data_path);

    spdlog::trace("Application::InitLogin() creating user datastore");
    m_userstore = std::make_unique<UserStore>(data_dir, account);

    spdlog::trace("Application::InitLogin() creating rate limiter");
    m_rate_limiter = std::make_unique<RateLimiter>(network_manager());

    spdlog::trace("Application::InitLogin() creating buyout manager");
    m_buyout_manager = std::make_unique<BuyoutManager>(data());

    spdlog::trace("Application::InitLogin() creating items manager");
    m_items_manager = std::make_unique<ItemsManager>(settings(), buyout_manager(), data());

    spdlog::trace("Application::InitLogin() creating items worker");
    m_items_worker = std::make_unique<ItemsManagerWorker>(settings(),
                                                          buyout_manager(),
                                                          rate_limiter());

    spdlog::trace("Application::InitLogin() creating shop");
    m_shop = std::make_unique<Shop>(settings(),
                                    network_manager(),
                                    rate_limiter(),
                                    data(),
                                    items_manager(),
                                    buyout_manager());

    spdlog::trace("Application::InitLogin() creating currency manager");
    m_currency_manager = std::make_unique<CurrencyManager>(settings(), data(), items_manager());

    auto repoe = m_repoe.get();
    auto manager = m_items_manager.get();
    auto worker = m_items_worker.get();
    auto userstore = m_userstore.get();

    connect(manager, &ItemsManager::UpdateSignal, worker, &ItemsManagerWorker::Update);
    connect(worker, &ItemsManagerWorker::StatusUpdate, manager, &ItemsManager::OnStatusUpdate);
    connect(worker, &ItemsManagerWorker::ItemsRefreshed, manager, &ItemsManager::OnItemsRefreshed);
    connect(manager, &ItemsManager::ItemsRefreshed, this, &Application::OnItemsRefreshed);

    connect(worker,
            &ItemsManagerWorker::characterListReceived,
            userstore,
            &UserStore::saveCharacterList);
    connect(worker, &ItemsManagerWorker::characterReceived, userstore, &UserStore::saveCharacter);
    connect(worker, &ItemsManagerWorker::stashListReceived, userstore, &UserStore::saveStashList);
    connect(worker, &ItemsManagerWorker::stashReceived, userstore, &UserStore::saveStash);

    if (m_repoe->IsInitialized()) {
        spdlog::debug("Application: RePoE data is available.");
        m_items_worker->OnRePoEReady();
    } else {
        spdlog::debug("Application: Waiting for RePoE data.");
        connect(repoe, &RePoE::finished, worker, &ItemsManagerWorker::OnRePoEReady);
    }
}

void Application::OnItemsRefreshed(bool initial_refresh)
{
    spdlog::trace("Application::OnItemsRefreshed() entered");
    spdlog::trace("Application::OnItemsRefreshed() initial_refresh = {}", initial_refresh);
    m_currency_manager->Update();
    m_shop->ExpireShopData();
    if (!initial_refresh && m_shop->auto_update()) {
        spdlog::trace("Application::OnItemsRefreshed() submitting shops");
        m_shop->SubmitShopToForum();
    }
}

void Application::SaveDataOnNewVersion()
{
    spdlog::trace("Application::SaveDataOnNewVersion() entered");
    //If user updated from a 0.5c db to a 0.5d, db exists but no "version" in it
    //QString version = m_data->Get("version", "0.5c");
    // We call this just after login, so we didn't pulled tabs for the first time ; so "tabs" shouldn't exist in the DB
    // This way we don't create an useless data_save_version folder on the first time you run acquisition

    auto version = m_settings->value("version").toString();

    // The version setting was introduced in v0.16, so for prior versions we
    // use the global data store.
    if (version.isEmpty()) {
        version = m_data->Get("version", "UNKNOWN-VERSION");
    }

    // Do nothing if the version is current.
    if (version == APP_VERSION_STRING) {
        spdlog::debug("Application: skipping backup: version is current");
        return;
    }

    const auto src_path = m_data_dir.filePath("data");
    QDir src{src_path};

    // Do nothing if there's no data directory.
    if (!src.exists()) {
        spdlog::debug("Application: skipping backup: directory does not exist: {}", src_path);
        return;
    }

    // Do nothing if the data directory is empty.
    if (src.entryList().isEmpty()) {
        spdlog::debug("Application: skipping backup: directory is empty: {}", src_path);
        return;
    }

    // Find a backup directory we can use.
    const auto dst_base = m_data_dir.filePath("data-backup-" + version);
    auto dst_path = dst_base;
    int n{0};
    while (m_data_dir.exists(dst_path)) {
        dst_path = dst_base + QString("-%1").arg(++n);
        if (n > 20) {
            spdlog::error("Application: skipping backup: too many backups!");
            return;
        }
    }

    QDir dst{dst_path};
    if (!dst.exists()) {
        spdlog::debug("Application: creating backup in '{}'", dst_path);
        dst.mkpath(dst_path);
    }

    spdlog::info("Application: backup up data from version '{}' into '{}'", version, dst_path);

    for (const auto &filename : src.entryList()) {
        const QString a = src.filePath(filename);
        const QString b = dst.filePath(filename);
        spdlog::debug("Application: backing up {} to {}", a, b);
        QFile::copy(a, b);
    }
    spdlog::info("Your data is backed up into '{}'", dst_path);

    spdlog::debug("Application: updating 'version' setting to {}", APP_VERSION_STRING);
    m_settings->setValue("version", APP_VERSION_STRING);
}
