// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

#include "application.h"

#include <QAction>
#include <QApplication>
#include <QDir>
#include <QFileDialog>
#include <QMessageBox>
#include <QNetworkCookieJar>
#include <QSettings>

#include "buyoutmanager.h"
#include "currencymanager.h"
#include "datastore/characterrepo.h"
#include "datastore/sqlitedatastore.h"
#include "datastore/stashrepo.h"
#include "datastore/userstore.h"
#include "imagecache.h"
#include "itemsmanager.h"
#include "itemsmanagerworker.h"
#include "ratelimit/ratelimiter.h"
#include "repoe/repoe.h"
#include "shop.h"
#include "ui/logindialog.h"
#include "ui/mainwindow.h"
#include "util/fatalerror.h"
#include "util/networkmanager.h"
#include "util/oauthmanager.h"
#include "util/spdlog_qt.h" // IWYU pragma: keep
#include "util/updatechecker.h"
#include "version_defines.h"

Application::Application(const QDir &appDataDir)
{
    spdlog::debug("Application: created");

    spdlog::debug("Application: data directory is {}", appDataDir.absolutePath());
    m_data_dir = appDataDir;

    InitCoreServices();
}

Application::~Application()
{
    spdlog::info("Shutting down.");
    spdlog::shutdown();
}

void Application::InitCoreServices()
{
    m_core = std::make_unique<Application::CoreServices>(m_data_dir);

    // Connect to the update signal in case an update is detected before the main window is open.
    connect(&update_checker(),
            &UpdateChecker::UpdateAvailable,
            &update_checker(),
            &UpdateChecker::AskUserToUpdate);

    // Connect signals from the login dialog.
    connect(&login(), &LoginDialog::ChangeTheme, this, &Application::SetTheme);
    connect(&login(), &LoginDialog::ChangeUserDir, this, &Application::SetUserDir);
    connect(&login(), &LoginDialog::LoginComplete, this, &Application::OnLogin);

    if (settings().value("realm").toString().isEmpty()) {
        settings().setValue("realm", "pc");
    };
    spdlog::trace("Application: realm is {}", settings().value("realm"));

    SaveDataOnNewVersion();

    spdlog::trace("Application: loading theme");
    SetTheme(settings().value("theme", "default").toString());

    // Start the process of fetching RePoE data.
    spdlog::trace("Application: initializing RePoE");
    repoe().Init(m_data_dir.absolutePath());

    // Start the initial check for updates.
    spdlog::trace("Application: checking for application updates");
    update_checker().CheckForUpdates();

    // Show the login dialog now.
    spdlog::trace("Application: showing the login dialog");
    login().show();
}

void Application::InitUserSession()
{
    m_session = std::make_unique<Application::UserSession>(core());

    // Disconnect from the update signal so that only the main window gets it from now on.
    QObject::disconnect(&update_checker(), &UpdateChecker::UpdateAvailable, nullptr, nullptr);

    auto manager = &items_manager();
    auto worker = &items_worker();

    connect(manager, &ItemsManager::UpdateSignal, worker, &ItemsManagerWorker::Update);
    connect(worker, &ItemsManagerWorker::StatusUpdate, manager, &ItemsManager::OnStatusUpdate);
    connect(worker, &ItemsManagerWorker::ItemsRefreshed, manager, &ItemsManager::OnItemsRefreshed);
    connect(manager, &ItemsManager::ItemsRefreshed, this, &Application::OnItemsRefreshed);

    auto characters = &userstore().characters();
    auto stashes = &userstore().stashes();

    connect(worker,
            &ItemsManagerWorker::characterListReceived,
            characters,
            &CharacterRepo::saveCharacterList);
    connect(worker,
            &ItemsManagerWorker::characterReceived,
            characters,
            &CharacterRepo::saveCharacter);
    connect(worker, &ItemsManagerWorker::stashListReceived, stashes, &StashRepo::saveStashList);
    connect(worker, &ItemsManagerWorker::stashReceived, stashes, &StashRepo::saveStash);

    auto main = &main_window();
    auto updater = &update_checker();
    auto cache = &image_cache();

    // Connect UI signals.
    connect(main, &MainWindow::SetSessionId, this, &Application::SetSessionId);
    connect(main, &MainWindow::SetTheme, this, &Application::SetTheme);
    connect(main, &MainWindow::UpdateCheckRequested, updater, &UpdateChecker::CheckForUpdates);

    connect(manager, &ItemsManager::ItemsRefreshed, main, &MainWindow::OnItemsRefreshed);
    connect(manager, &ItemsManager::StatusUpdate, main, &MainWindow::OnStatusUpdate);

    connect(main, &MainWindow::GetImage, cache, &ImageCache::fetch);
    connect(cache, &ImageCache::imageReady, main, &MainWindow::OnImageFetched);

    connect(&shop(), &Shop::StatusUpdate, main, &MainWindow::OnStatusUpdate);

    // Connect the update checker.
    connect(updater, &UpdateChecker::UpdateAvailable, main, &MainWindow::OnUpdateAvailable);
}

Application::CoreServices &Application::core() const
{
    if (m_core) {
        return *m_core;
    }
    FatalError("Application: core services not initialized");
}

Application::UserSession &Application::session() const
{
    if (m_session) {
        return *m_session;
    }
    FatalError("Application: user session not initialized");
}

Application::CoreServices::CoreServices(const QDir &appDataDir)
    : dir(appDataDir)
{
    spdlog::trace("Application: creating NetworkManager");
    network_manager = std::make_unique<NetworkManager>();

    spdlog::trace("Application: creating RePoE");
    repoe = std::make_unique<RePoE>(*network_manager);

    const QString settings_path = dir.filePath("settings.ini");
    spdlog::trace("Application: creating the settings object: {}", settings_path);
    settings = std::make_unique<QSettings>(settings_path, QSettings::IniFormat);

    const QDir user_dir(dir.filePath("data"));
    const QString global_data_file = user_dir.filePath(SqliteDataStore::MakeFilename("", ""));
    spdlog::trace("Application: opening global data file: {}", global_data_file);
    global_data = std::make_unique<SqliteDataStore>(global_data_file);

    const QString image_cache_dir = dir.absoluteFilePath("cache");
    image_cache = std::make_unique<ImageCache>(*network_manager, image_cache_dir);

    spdlog::trace("Application: creating update checker");
    update_checker = std::make_unique<UpdateChecker>(*settings, *network_manager);

    spdlog::trace("Application: creating OAuth manager");
    oauth_manager = std::make_unique<OAuthManager>(*network_manager, *global_data);

    spdlog::trace("Application: creating login dialog");
    login = std::make_unique<LoginDialog>(dir.absolutePath(),
                                          *settings,
                                          *network_manager,
                                          *oauth_manager,
                                          *global_data);
}

Application::UserSession::UserSession(const Application::CoreServices &core)
{
    spdlog::trace("Application::InitLogin() entered");

    const auto &dir = core.dir;
    auto &settings = *core.settings;
    auto &network_manager = *core.network_manager;
    auto &image_cache = *core.image_cache;

    const QString league = settings.value("league").toString();
    const QString account = settings.value("account").toString();
    if (league.isEmpty()) {
        FatalError("Login failure: the league has not been set.");
    }
    if (account.isEmpty()) {
        FatalError("Login failure: the account has not been set.");
    }
    spdlog::trace("Application::InitLogin() league = {}", league);
    spdlog::trace("Application::InitLogin() account = {}", account);

    QDir data_dir = dir.filePath("data");
    const QString data_file = SqliteDataStore::MakeFilename(account, league);
    const QString data_path = data_dir.absoluteFilePath(data_file);
    spdlog::trace("Application::InitLogin() data_path = {}", data_path);
    data = std::make_unique<SqliteDataStore>(data_path);

    spdlog::trace("Application::InitLogin() creating user datastore");
    userstore = std::make_unique<UserStore>(data_dir, account);

    spdlog::trace("Application::InitLogin() creating rate limiter");
    rate_limiter = std::make_unique<RateLimiter>(network_manager);

    spdlog::trace("Application::InitLogin() creating buyout manager");
    buyout_manager = std::make_unique<BuyoutManager>(*data);

    spdlog::trace("Application::InitLogin() creating items manager");
    items_manager = std::make_unique<ItemsManager>(settings, *buyout_manager, *data);

    spdlog::trace("Application::InitLogin() creating items worker");
    items_worker = std::make_unique<ItemsManagerWorker>(settings, *buyout_manager, *rate_limiter);

    spdlog::trace("Application::InitLogin() creating shop");
    shop = std::make_unique<Shop>(settings,
                                  network_manager,
                                  *rate_limiter,
                                  *data,
                                  *items_manager,
                                  *buyout_manager);

    spdlog::trace("Application::InitLogin() creating currency manager");
    currency_manager = std::make_unique<CurrencyManager>(settings, *data, *items_manager);

    // Prepare to show the main window now that everything is initialized.
    spdlog::trace("Application:creating main window");
    main_window = std::make_unique<MainWindow>(settings,
                                               network_manager,
                                               *rate_limiter,
                                               *data,
                                               *items_manager,
                                               *buyout_manager,
                                               *currency_manager,
                                               *shop,
                                               image_cache);
}

void Application::OnLogin()
{
    spdlog::debug("Application: login initiated");

    InitUserSession();

    if (repoe().IsInitialized()) {
        spdlog::debug("Application: RePoE data is available.");
        items_worker().OnRePoEReady();
        emit repoe().finished();
    } else {
        spdlog::debug("Application: Waiting for RePoE data.");
        connect(&repoe(), &RePoE::finished, &items_worker(), &ItemsManagerWorker::OnRePoEReady);
    }

    spdlog::trace("Application::OnLogin() closing the login dialog");
    login().close();

    spdlog::trace("Application::OnLogin() showing the main window");
    main_window().show();
}

void Application::InitCrashReporting()
{
    spdlog::trace("Application::InitCrashReporting() entered");

    // Enable crash reporting by default.
    bool report_crashes = true;

    if (!settings().contains("report_crashes")) {
        // Update the setting if it didn't exist before.
        spdlog::trace("Application::InitCrashReporting() setting 'report_crashes' to true");
        settings().setValue("report_crashes", true);
    } else {
        // Use the exiting setting.
        report_crashes = settings().value("report_crashes").toBool();
        spdlog::trace("Application::InitCrashReporting() 'report_crashes' is {}", report_crashes);
    }
}

void Application::SetSessionId(const QString &poesessid)
{
    if (poesessid.isEmpty()) {
        spdlog::error("Application: cannot update POESESSID: value is empty");
        return;
    }
    network_manager().setPoeSessionId(poesessid);
    settings().setValue("session_id", poesessid);
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
    m_session.reset();
    m_core.reset();
    m_data_dir = QDir(dir);

    InitCoreServices();
}

void Application::OnItemsRefreshed(bool initial_refresh)
{
    spdlog::trace("Application::OnItemsRefreshed() entered");
    spdlog::trace("Application::OnItemsRefreshed() initial_refresh = {}", initial_refresh);
    currency_manager().Update();
    shop().ExpireShopData();
    if (!initial_refresh && shop().auto_update()) {
        spdlog::trace("Application::OnItemsRefreshed() submitting shops");
        shop().SubmitShopToForum();
    }
}

void Application::SaveDataOnNewVersion()
{
    spdlog::trace("Application::SaveDataOnNewVersion() entered");
    //If user updated from a 0.5c db to a 0.5d, db exists but no "version" in it
    //QString version = m_data->Get("version", "0.5c");
    // We call this just after login, so we didn't pulled tabs for the first time ; so "tabs" shouldn't exist in the DB
    // This way we don't create an useless data_save_version folder on the first time you run acquisition

    auto version = settings().value("version").toString();

    // The version setting was introduced in v0.16, so for prior versions we
    // use the global data store.
    if (version.isEmpty()) {
        version = global_data().Get("version", "UNKNOWN-VERSION");
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
    core().settings->setValue("version", APP_VERSION_STRING);
}
