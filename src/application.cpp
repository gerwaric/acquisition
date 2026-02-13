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

#include "app/coreservices.h"
#include "app/sessionservices.h"
#include "app/usersettings.h"
#include "datastore/keychainstore.h"
#include "datastore/stashstore.h"
#include "imagecache.h"
#include "itemsmanager.h"
#include "itemsmanagerworker.h"
#include "repoe/repoe.h"
#include "shop.h"
#include "ui/logindialog.h"
#include "ui/mainwindow.h"
#include "util/fatalerror.h"
#include "util/networkmanager.h"
#include "util/spdlog_qt.h" // IWYU pragma: keep
#include "util/updatechecker.h"
#include "version_defines.h"

Application::Application(const QDir &dataDir)
    : m_data_dir(dataDir)
{
    spdlog::debug("Application: data directory is {}", m_data_dir.absolutePath());

    // Load user settings.
    m_settings = std::make_unique<app::UserSettings>(dataDir);

    // Setup core services.
    m_core = std::make_unique<app::CoreServices>(*m_settings);
    m_core->start();

    saveDataOnNewVersion();

    // Create the login dialog.
    m_login_dialog = std::make_unique<LoginDialog>(*m_settings,
                                                   m_core->network_manager(),
                                                   m_core->oauth_manager());

    // Connect signals from the login dialog.
    auto *login_dialog = m_login_dialog.get();
    connect(login_dialog, &LoginDialog::ChangeTheme, m_settings.get(), [this](const QString &theme) {
        m_settings->theme(theme);
    });
    connect(login_dialog, &LoginDialog::LoginComplete, this, &Application::startNewSession);
    connect(login_dialog, &LoginDialog::RemoveOAuthToken, this, [this] {
        const QString key = "oauth_token/" + m_settings->username();
        m_core->keychain().remove(key);
    });

    // Show the login dialog.
    spdlog::debug("Application: showing the login dialog");
    m_login_dialog->show();
}

Application::~Application() {}

void Application::startNewSession()
{
    spdlog::debug("Application: starting a user session.");

    if (!m_core) {
        FatalError("Application: CoreServices not initialized.");
    }

    m_session = std::make_unique<app::SessionServices>(*m_settings, *m_core);

    // Disconnect from the update signal so that only the main window gets it from now on.
    auto &updater = m_core->update_checker();
    disconnect(&updater, &UpdateChecker::UpdateAvailable, nullptr, nullptr);

    auto &repoe = m_core->repoe();
    auto &items_worker = m_session->items_worker();

    if (repoe.initialized()) {
        spdlog::debug("Application: RePoE data is available.");
        items_worker.OnRePoEReady();
        emit repoe.finished();
    } else {
        spdlog::debug("Application: Waiting for RePoE data.");
        connect(&repoe, &RePoE::finished, &items_worker, &ItemsManagerWorker::OnRePoEReady);
    }

    spdlog::trace("Application: closing the login dialog");
    m_login_dialog->close();

    createMainWindow();

    spdlog::trace("Application: showing the main window");
    m_main_window->show();
}

void Application::loadTheme(const QString &theme)
{
    spdlog::debug("Application: loading theme: '{}'", theme);

    QString stylesheet;
    if (0 == theme.compare("default", Qt::CaseInsensitive)) {
    } else if (0 == theme.compare("light", Qt::CaseInsensitive)) {
        stylesheet = ":qdarkstyle/light/lightstyle.qss";
    } else if (0 == theme.compare("dark", Qt::CaseInsensitive)) {
        stylesheet = ":qdarkstyle/dark/darkstyle.qss";
    } else {
        spdlog::error("Application: invalid theme: {}", theme);
        return;
    }

    QString style_data;
    if (!stylesheet.isEmpty()) {
        spdlog::trace("Application: loading stylesheet: {}", stylesheet);
        QFile f(stylesheet);
        if (f.exists()) {
            if (!f.open(QFile::ReadOnly | QFile::Text)) {
                spdlog::error("UserSettings: error loading stylesheet ({}): {}",
                              stylesheet,
                              f.errorString());
            } else {
                style_data = f.readAll();
                f.close();
            }
        } else {
            spdlog::error("Application: style sheet not found: {}", stylesheet);
        }
    }

    spdlog::trace("Application: setting stylesheet");
    qApp->setStyleSheet(style_data);
}

void Application::createMainWindow()
{
    spdlog::debug("Application: initializing main window.");

    if (!m_core) {
        FatalError("Application: cannot create main window: CoreServices is null.");
    }
    if (!m_session) {
        FatalError("Application: cannot create main window: UserSession is null.");
    }

    // Prepare to show the main window now that everything is initialized.
    m_main_window = std::make_unique<MainWindow>(*m_settings,
                                                 m_core->network_manager(),
                                                 m_session->rate_limiter(),
                                                 m_session->items_manager(),
                                                 m_session->buyout_manager(),
                                                 m_session->shop(),
                                                 m_core->image_cache());

    auto *settings = m_settings.get();
    auto *main_window = m_main_window.get();

    auto *network_manager = &m_core->network_manager();
    auto *update_checker = &m_core->update_checker();
    auto *image_cache = &m_core->image_cache();
    auto *shop = &m_session->shop();
    auto &items_mgr = m_session->items_manager();

    // Connect UI signals.
    connect(main_window, &MainWindow::SetSessionId, network_manager, &NetworkManager::setPoesessid);
    connect(main_window, &MainWindow::SetTheme, settings, [this](const QString &theme) {
        m_settings->theme(theme);
    });
    connect(main_window,
            &MainWindow::UpdateCheckRequested,
            update_checker,
            &UpdateChecker::CheckForUpdates);

    connect(&items_mgr, &ItemsManager::ItemsRefreshed, main_window, &MainWindow::OnItemsRefreshed);
    connect(&items_mgr, &ItemsManager::StatusUpdate, main_window, &MainWindow::OnStatusUpdate);

    connect(main_window, &MainWindow::GetImage, image_cache, &ImageCache::fetch);
    connect(image_cache, &ImageCache::imageReady, main_window, &MainWindow::OnImageFetched);

    connect(shop, &Shop::StatusUpdate, main_window, &MainWindow::OnStatusUpdate);

    // Connect the update checker.
    connect(update_checker,
            &UpdateChecker::UpdateAvailable,
            main_window,
            &MainWindow::OnUpdateAvailable);
}

void Application::saveDataOnNewVersion()
{
    spdlog::trace("Application::SaveDataOnNewVersion() entered");    
    //If user updated from a 0.5c db to a 0.5d, db exists but no "version" in it
    //QString version = m_data->Get("version", "0.5c");
    // We call this just after login, so we didn't pulled tabs for the first time ; so "tabs" shouldn't exist in the DB
    // This way we don't create an useless data_save_version folder on the first time you run acquisition

    if (!m_core) {
        FatalError("Application: cannot create main window: CoreServices is null.");
    }

    auto version = settings.value("version").toString();

    // The version setting was introduced in v0.16, so for prior versions we
    // use the global data store.
    if (version.isEmpty()) {
        spdlog::warn("SaveDataOnNewVersion: TBD");
        //version = global_data.Get("version", "UNKNOWN-VERSION");
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
    settings.setValue("version", APP_VERSION_STRING);
}
