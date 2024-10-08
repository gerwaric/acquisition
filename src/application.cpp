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

#include "application.h"

#include <QAction>
#include <QApplication>
#include <QDir>
#include <QMessageBox>
#include <QNetworkAccessManager>
#include <QSettings>
#include <QTemporaryFile>
#include <QtHttpServer/QHttpServer>

#include "boost/current_function.hpp"
#include "QsLog.h"

#include "buyoutmanager.h"
#include "crashpad.h"
#include "currencymanager.h"
#include "fatalerror.h"
#include "filesystem.h"
#include "itemsmanager.h"
#include "logindialog.h"
#include "mainwindow.h"
#include "memorydatastore.h"
#include "network_info.h"
#include "oauthmanager.h"
#include "ratelimiter.h"
#include "ratelimitmanager.h"
#include "repoe.h"
#include "shop.h"
#include "sqlitedatastore.h"
#include "testmain.h"
#include "testsettings.h"
#include "updatechecker.h"
#include "version_defines.h"

Application::Application(bool test_mode) :
    test_mode_(test_mode)
{
    QLOG_TRACE() << "Application::Application() entered";

    if (test_mode_) {

        QLOG_TRACE() << "Application::Application() preparing settings and data for test mode";
        settings_ = TestSettings::NewInstance();
        global_data_ = std::make_unique<MemoryDataStore>();

    } else {

        QLOG_TRACE() << "Application::Application() preparing settings and data";
        const QString user_dir = Filesystem::UserDir();
        QLOG_TRACE() << "Application::Application() data directory is" << user_dir;

        const QString settings_path = user_dir + "/settings.ini";
        QLOG_TRACE() << "Application::Application() creating the settings object:" << settings_path;
        settings_ = std::make_unique<QSettings>(settings_path, QSettings::IniFormat);

        QLOG_TRACE() << "Application::Application() initializing crash reporting";
        InitCrashReporting();

        const QString global_data_file = user_dir + "/data/" + SqliteDataStore::MakeFilename("", "");
        QLOG_TRACE() << "Application::Application() opening global data file:" << global_data_file;
        global_data_ = std::make_unique<SqliteDataStore>(global_data_file);

        QLOG_TRACE() << "Application::Application() loading theme";
        const QString theme = settings().value("theme", "default").toString();
        OnSetTheme(theme);
    };

    QLOG_TRACE() << "Application::Application() creating QNetworkAccessManager";
    network_manager_ = std::make_unique<QNetworkAccessManager>(this);

    QLOG_TRACE() << "Application::Application() creating RePoE";
    repoe_ = std::make_unique<RePoE>(this, network_manager());

    QLOG_TRACE() << "Application::Application() creating update checker";
    update_checker_ = std::make_unique<UpdateChecker>(this, settings(), network_manager());

    QLOG_TRACE() << "Application::Application() creating OAuth manager";
    oauth_manager_ = std::make_unique<OAuthManager>(this, network_manager(), global_data());

    // Start the process of fetching RePoE data.
    QLOG_TRACE() << "Application::Application() initializing RePoE";
    repoe_->Init();

    if (test_mode) {
        QLOG_TRACE() << "Application::Application() calling InitLogin for test mode";
        InitLogin(POE_API::LEGACY);
    };
}

Application::~Application() {
    QLOG_TRACE() << "Application::~Application() entered";
    if (buyout_manager_) {
        QLOG_TRACE() << "Application::~Application() saving buyout manager";
        buyout_manager_->Save();
    };
}

void Application::Start() {

    QLOG_TRACE() << "Application::Start() entered";

    QLOG_TRACE() << "Application::Start() creaating login dialog";
    login_ = std::make_unique<LoginDialog>(
        settings(),
        network_manager(),
        oauth_manager());

    // Connect to the update signal in case an update is detected before the main window is open.
    QLOG_TRACE() << "Application::Start() connecting update checker";
    QObject::connect(update_checker_.get(), &UpdateChecker::UpdateAvailable, update_checker_.get(), &UpdateChecker::AskUserToUpdate);

    // Use the login complete signal to setup the main window.
    QLOG_TRACE() << "Application::Start() connecting login";
    QObject::connect(login_.get(), &LoginDialog::LoginComplete, this, &Application::OnLogin);

    // Setup the ability to trigger testing from the UI
    QLOG_TRACE() << "Application::Start() installing keyboard shortcut for forced crashes";
    connect(&test_action_, &QAction::triggered, this, &Application::OnRunTests);
    test_action_.setShortcut(Qt::Key_T | Qt::CTRL);
    login_->addAction(&test_action_);

    // Start the initial check for updates.
    QLOG_TRACE() << "Application::Start() starting a check for application updates";
    update_checker_->CheckForUpdates();

    // Show the login dialog now.
    QLOG_TRACE() << "Application::Start() showing the login dialog";
    login_->show();
}

void Application::OnLogin(POE_API api) {

    QLOG_TRACE() << "Application::OnLogin() entered";

    // Stop listening for CTRL+T
    QLOG_TRACE() << "Application::OnLogin() uninstalling the keyboard shortcut for forced crashes";
    login_->removeAction(&test_action_);

    // Disconnect from the update signal so that only the main window gets it from now on.
    QObject::disconnect(&update_checker(), &UpdateChecker::UpdateAvailable, nullptr, nullptr);

    // Call init login to setup the shop, items manager, and other objects.
    QLOG_TRACE() << "Application::OnLogin() initializing login";
    InitLogin(api);

    // Prepare to show the main window now that everything is initialized.
    QLOG_TRACE() << "Application::OnLogin() creating main window";
    main_window_ = std::make_unique<MainWindow>(
        settings(),
        network_manager(),
        rate_limiter(),
        data(),
        oauth_manager(),
        items_manager(),
        buyout_manager(),
        currency_manager(),
        update_checker(),
        shop());

    // Connect the theme signal.
    connect(main_window_.get(), &MainWindow::SetTheme, this, &Application::OnSetTheme);

    QLOG_TRACE() << "Application::OnLogin() closing the login dialog";
    login_->close();

    QLOG_TRACE() << "Application::OnLogin() showing the main window";
    main_window_->show();
}

QSettings& Application::settings() const {
    if (!settings_) {
        FatalError("Application::settings() attempted to dereference a null pointer");
    };
    return *settings_;
};

ItemsManager& Application::items_manager() const {
    if (!items_manager_) {
        FatalError("Application::items_manager() attempted to dereference a null pointer");
    };
    return *items_manager_;
};

DataStore& Application::global_data() const {
    if (!global_data_) {
        FatalError("Application::global_data() attempted to dereference a null pointer");
    };
    return *global_data_;
};

DataStore& Application::data() const {
    if (!data_) {
        FatalError("Application::data() attempted to dereference a null pointer");
    };
    return *data_;
};

BuyoutManager& Application::buyout_manager() const {
    if (!buyout_manager_) {
        FatalError("Application::buyout_manager() attempted to dereference a null pointer");
    };
    return *buyout_manager_;
};

QNetworkAccessManager& Application::network_manager() const {
    if (!network_manager_) {
        FatalError("Application::network_manager() attempted to dereference a null pointer");
    };
    return *network_manager_;
}

RePoE& Application::repoe() const {
    if (!repoe_) {
        FatalError("Application::repoe() attempted to dereference a null pointer");
    };
    return *repoe_;
}

Shop& Application::shop() const {
    if (!shop_) {
        FatalError("Application::shop() attempted to dereference a null pointer");
    };
    return *shop_;
}

CurrencyManager& Application::currency_manager() const {
    if (!currency_manager_) {
        FatalError("Application::currency_manager() attempted to dereference a null pointer");
    };
    return *currency_manager_;
}

UpdateChecker& Application::update_checker() const {
    if (!update_checker_) {
        FatalError("Application::update_checker() attempted to dereference a null pointer");
    };
    return *update_checker_;
}

OAuthManager& Application::oauth_manager() const {
    if (!oauth_manager_) {
        FatalError("Application::oauth_manager() attempted to dereference a null pointer");
    };
    return *oauth_manager_;
}

RateLimiter& Application::rate_limiter() const {
    if (!rate_limiter_) {
        FatalError("Application::rate_limiter() attempted to dereference a null pointer");
    };
    return *rate_limiter_;
}

void Application::InitCrashReporting() {
    QLOG_TRACE() << "Application::InitCrashReporting() entered";

    // Make sure the settings object exists.
    if (!settings_) {
        QLOG_ERROR() << "Cannot init crash reporting because settings object is invalid";
        return;
    };

    // Enable crash reporting by default.
    bool report_crashes = true;

    if (!settings_->contains("report_crashes")) {
        // Update the setting if it didn't exist before.
        QLOG_TRACE() << "Application::InitCrashReporting() setting 'report_crashes' to true";
        settings_->setValue("report_crashes", true);
    } else {
        // Use the exiting setting.
        report_crashes = settings_->value("report_crashes").toBool();
        QLOG_TRACE() << "Application::InitCrashReporting() 'report_crashes' is" << report_crashes;
    };

    // Initialize crash reporting with crashpad.
    if (report_crashes) {
        QLOG_TRACE() << "Application::InitCrashReporting() initializing crashpad";
        initializeCrashpad(Filesystem::UserDir(), APP_PUBLISHER, APP_NAME, APP_VERSION_STRING);
    };
}

void Application::OnSetTheme(const QString& theme) {
    QLOG_TRACE() << "Application::OnSetTheme() entered";

    if (0 == theme.compare(active_theme_, Qt::CaseInsensitive)) {
        QLOG_DEBUG() << "Theme is already set:" << theme;
        return;
    };

    QString stylesheet;
    QColor text_color;

    if (0 == theme.compare("default", Qt::CaseInsensitive)) {
        stylesheet = "";
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

    QString style_data;
    if (!stylesheet.isEmpty()) {
        QLOG_TRACE() << "Application::OnSetTheme() loading stylesheet:" << stylesheet;
        QFile f(stylesheet);
        if (f.exists()) {
            f.open(QFile::ReadOnly | QFile::Text);
            style_data = f.readAll();
        } else {
            QLOG_ERROR() << "Style sheet not found:" << stylesheet;
        };
    };

    QLOG_TRACE() << "Application::OnSetTheme() setting stylesheet";
    qApp->setStyleSheet(style_data);

    QLOG_TRACE() << "Application::OnSetTheme() setting window text color:" << text_color;
    QPalette p = QApplication::palette();
    p.setColor(QPalette::WindowText, text_color);
    QApplication::setPalette(p);
}

void Application::InitLogin(POE_API mode)
{
    QLOG_TRACE() << "Application::InitLogin() entered";

    if (test_mode_) {
        QLOG_TRACE() << "Application::InitLogin() creating memmory data store";
        data_ = std::make_unique<MemoryDataStore>();
    } else {
        const std::string league = settings_->value("league").toString().toStdString();
        const std::string account = settings_->value("account").toString().toStdString();
        const QString data_dir = Filesystem::UserDir() + "/data/";
        if (league.empty()) {
            FatalError("Login failure: the league has not been set.");
        };
        if (account.empty()) {
            FatalError("Login failure: the account has not been set.");
        };
        QLOG_TRACE() << "Application::InitLogin() league =" << league;
        QLOG_TRACE() << "Application::InitLogin() account =" << account;
        QLOG_TRACE() << "Application::InitLogin() data_dir =" << data_dir;
        const QString data_file = SqliteDataStore::MakeFilename(account, league);
        const QString data_path = data_dir + data_file;
        QLOG_TRACE() << "Application::InitLogin() data_path =" << data_path;
        data_ = std::make_unique<SqliteDataStore>(data_path);
        SaveDbOnNewVersion();
    }

    QLOG_TRACE() << "Application::InitLogin() creating rate limiter";
    rate_limiter_ = std::make_unique<RateLimiter>(this,
        network_manager(),
        oauth_manager(), mode);

    QLOG_TRACE() << "Application::InitLogin() creating buyout manager";
    buyout_manager_ = std::make_unique<BuyoutManager>(
        data());

    QLOG_TRACE() << "Application::InitLogin() creating items manager";
    items_manager_ = std::make_unique<ItemsManager>(this,
        settings(),
        network_manager(),
        repoe(),
        buyout_manager(),
        data(),
        rate_limiter());

    QLOG_TRACE() << "Application::InitLogin() creating shop";
    shop_ = std::make_unique<Shop>(this,
        settings(),
        network_manager(),
        data(),
        items_manager(),
        buyout_manager());

    QLOG_TRACE() << "Application::InitLogin() creating currency manager";
    currency_manager_ = std::make_unique<CurrencyManager>(nullptr,
        settings(),
        data(),
        items_manager());

    connect(items_manager_.get(), &ItemsManager::ItemsRefreshed, this, &Application::OnItemsRefreshed);

    if (test_mode_ == false) {
        QLOG_TRACE() << "Application::InitLogin() starting items manager";
        items_manager_->Start(mode);
    };
}

void Application::OnItemsRefreshed(bool initial_refresh) {
    QLOG_TRACE() << "Application::OnItemsRefreshed() entered";
    QLOG_TRACE() << "Application::OnItemsRefreshed() initial_refresh =" << initial_refresh;
    currency_manager_->Update();
    shop_->Update();
    if (!initial_refresh && shop_->auto_update()) {
        QLOG_TRACE() << "Application::OnItemsRefreshed() submitting shops";
        shop_->SubmitShopToForum();
    };
}

void Application::OnRunTests() {
    QLOG_TRACE() << "Application::OnRunTests() entered";
    if (!repoe_->IsInitialized()) {
        QLOG_TRACE() << "Application::OnRunTests() RePoE is not initialized";
        QMessageBox::information(nullptr,
            "Acquisition", "RePoE is not initialized yet. Try again later",
            QMessageBox::StandardButton::Ok,
            QMessageBox::StandardButton::Ok);
        return;
    }
    QLOG_TRACE() << "Application::OnRunTests() hiding login and running tests";
    login_->hide();
    const int result = test_main();
    QLOG_TRACE() << "Application::OnRunTests(): test_main returned" << result;
    QMessageBox::information(nullptr,
        "Acquisition", "Testing returned " + QString::number(result),
        QMessageBox::StandardButton::Ok,
        QMessageBox::StandardButton::Ok);
    login_->show();
}

void Application::SaveDbOnNewVersion() {

    QLOG_TRACE() << "Application::SaveDbOnNewVersion() entered";
    //If user updated from a 0.5c db to a 0.5d, db exists but no "version" in it
    std::string version = data_->Get("version", "0.5c");
    // We call this just after login, so we didn't pulled tabs for the first time ; so "tabs" shouldn't exist in the DB
    // This way we don't create an useless data_save_version folder on the first time you run acquisition

    bool first_start = data_->Get("tabs", "first_time") == "first_time" &&
        data_->GetTabs(ItemLocationType::STASH).size() == 0 &&
        data_->GetTabs(ItemLocationType::CHARACTER).size() == 0;
    QLOG_TRACE() << "Application::SaveDbOnNewVersion() first_start =" << first_start;

    if (version != APP_VERSION_STRING && !first_start) {
        QString data_path = Filesystem::UserDir() + QString("/data");
        QString save_path = data_path + "_save_" + version.c_str();
        QLOG_TRACE() << "Application::SaveDbOnNewVersion() data_path =" << data_path;
        QLOG_TRACE() << "Application::SaveDbOnNewVersion() save_path =" << save_path;
        QDir src(data_path);
        QDir dst(save_path);
        if (!dst.exists()) {
            QLOG_TRACE() << "Application::SaveDbOnNewVersion() creating save_path";
            QDir().mkpath(dst.path());
        };
        for (const auto& name : src.entryList()) {
            const QString a = data_path + QDir::separator() + name;
            const QString b = save_path + QDir::separator() + name;
            QLOG_TRACE() << "Application::SaveDbOnNewVersion() copying" << a << "to" << b;
            QFile::copy(a, b);
        }
        QLOG_INFO() << "I've created the folder " << save_path << "in your acquisition folder, containing a save of all your data";
    }

    QLOG_TRACE() << "Application::SaveDbOnNewVersion() setting 'version' to" << APP_VERSION_STRING;
    data_->Set("version", APP_VERSION_STRING);
}
