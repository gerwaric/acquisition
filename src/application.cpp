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
#include <QtHttpServer/QHttpServer>

#include "QsLog.h"

#include "buyoutmanager.h"
#include "crashpad.h"
#include "currencymanager.h"
#include "filesystem.h"
#include "itemsmanager.h"
#include "logindialog.h"
#include "mainwindow.h"
#include "memorydatastore.h"
#include "network_info.h"
#include "oauthmanager.h"
#include "ratelimiter.h"
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
    connect(this, &Application::Quit, qApp, &QApplication::quit, Qt::QueuedConnection);

    if (test_mode_) {

        settings_ = TestSettings::NewInstance();
        global_data_ = std::make_unique<MemoryDataStore>();

    } else {

        const QString user_dir = Filesystem::UserDir();
        const QString settings_path = user_dir + "/settings.ini";
        const QString global_data_file = user_dir + "/data/" + SqliteDataStore::MakeFilename("", "");
       
        settings_ = std::make_unique<QSettings>(settings_path, QSettings::IniFormat);
        global_data_ = std::make_unique<SqliteDataStore>(global_data_file);

        InitCrashReporting();
        LoadTheme();
    };

    network_manager_ = std::make_unique<QNetworkAccessManager>(this);
    repoe_ = std::make_unique<RePoE>(this, network_manager());
    update_checker_ = std::make_unique<UpdateChecker>(this, settings(), network_manager());
    oauth_manager_ = std::make_unique<OAuthManager>(this, network_manager(), global_data());

    // Start the process of fetching RePoE data.
    repoe_->Init();

    if (test_mode) {
        InitLogin(POE_API::LEGACY);
    };
}

Application::~Application() {
    if (buyout_manager_) {
        buyout_manager_->Save();
    };
}

void Application::Start() {

    login_ = std::make_unique<LoginDialog>(
        settings(),
        network_manager(),
        oauth_manager());

    if (!login_) {
        QLOG_FATAL() << "Failed to create the login dialog.";
        return;
    };

    // Connect to the update signal in case an update is detected before the main window is open.
    QObject::connect(update_checker_.get(), &UpdateChecker::UpdateAvailable, update_checker_.get(), &UpdateChecker::AskUserToUpdate);

    // Use the login complete signal to setup the main window.
    QObject::connect(login_.get(), &LoginDialog::LoginComplete, this, &Application::OnLogin);

    // Setup the ability to trigger testing from the UI
    connect(&test_action_, &QAction::triggered, this, &Application::OnRunTests);
    test_action_.setShortcut(Qt::Key_T | Qt::CTRL);
    login_->addAction(&test_action_);

    // Start the initial check for updates.
    update_checker_->CheckForUpdates();

    // Show the login dialog now.
    login_->show();
}

void Application::OnLogin(POE_API api) {

    // Stop listening for CTRL+T
    login_->removeAction(&test_action_);

    // Disconnect from the update signal so that only the main window gets it from now on.
    QObject::disconnect(&update_checker(), &UpdateChecker::UpdateAvailable, nullptr, nullptr);

    // Call init login to setup the shop, items manager, and other objects.
    InitLogin(api);

    // Prepare to show the main window now that everything is initialized.
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

    login_->close();
    main_window_->show();
}

QSettings& Application::settings() const {
    if (!settings_) {
        FatalAccessError("settings");
    };
    return *settings_;
};

ItemsManager& Application::items_manager() const {
    if (!items_manager_) {
        FatalAccessError("items nanager");
    };
    return *items_manager_;
};

DataStore& Application::global_data() const {
    if (!global_data_) {
        FatalAccessError("global datastore");
    };
    return *global_data_;
};

DataStore& Application::data() const {
    if (!data_) {
        FatalAccessError("datastore");
    };
    return *data_;
};

BuyoutManager& Application::buyout_manager() const {
    if (!buyout_manager_) {
        FatalAccessError("buyout manager");
    };
    return *buyout_manager_;
};

QNetworkAccessManager& Application::network_manager() const {
    if (!network_manager_) {
        FatalAccessError("network access manager");
    };
    return *network_manager_;
}

RePoE& Application::repoe() const {
    if (!repoe_) {
        FatalAccessError("RePoE");
    };
    return *repoe_;
}

Shop& Application::shop() const {
    if (!shop_) {
        FatalAccessError("shop");
    };
    return *shop_;
}

CurrencyManager& Application::currency_manager() const {
    if (!currency_manager_) {
        FatalAccessError("currency manager");
    };
    return *currency_manager_;
}

UpdateChecker& Application::update_checker() const {
    if (!update_checker_) {
        FatalAccessError("update checker");
    };
    return *update_checker_;
}

OAuthManager& Application::oauth_manager() const {
    if (!oauth_manager_) {
        FatalAccessError("OAuth manager");
    };
    return *oauth_manager_;
}

RateLimiter& Application::rate_limiter() const {
    if (!rate_limiter_) {
        FatalAccessError("rate limiter");
    };
    return *rate_limiter_;
}

void Application::FatalAccessError(const char* object_name) const {
    const QString message = QString("The '%1' object was invalid.").arg(object_name);
    QLOG_FATAL() << message;
    QMessageBox::critical(nullptr,
        "Acquisition Fatal Error", message,
        QMessageBox::StandardButton::Abort,
        QMessageBox::StandardButton::Abort);
}

void Application::InitCrashReporting() {

    // Make sure the settings object exists.
    if (!settings_) {
        QLOG_ERROR() << "Cannot init crash reporting because settings object is invalid";
        return;
    };

    // Enable crash reporting by default.
    bool report_crashes = true;

    if (!settings_->contains("report_crashes")) {
        // Update the setting if it didn't exist before.
        settings_->setValue("report_crashes", true);
    } else {
        // Use the exiting setting.
        report_crashes = settings_->value("report_crashes").toBool();
    };

    // Initialize crash reporting with crashpad.
    if (report_crashes) {
        initializeCrashpad(Filesystem::UserDir(), APP_PUBLISHER, APP_NAME, APP_VERSION_STRING);
    };
}

void Application::LoadTheme() {
    // Load the appropriate theme.
    const QString theme = settings_->value("theme").toString();

    // Do nothing for the default theme.
    if (theme == "default") {
        return;
    };

    // Determine which qss file to use.
    QString stylesheet;
    if (theme == "dark") {
        stylesheet = ":qdarkstyle/dark/darkstyle.qss";
    } else if (theme == "light") {
        stylesheet = ":qdarkstyle/light/lightstyle.qss";
    } else {
        QLOG_ERROR() << "Invalid theme:" << theme;
        return;
    };

    // Load the theme.
    QFile f(stylesheet);
    if (!f.exists()) {
        QLOG_ERROR() << "Theme stylesheet not found:" << stylesheet;
    } else {
        f.open(QFile::ReadOnly | QFile::Text);
        QTextStream ts(&f);
        const QString theme_data = ts.readAll();
        qApp->setStyleSheet(theme_data);
        QPalette pal = QApplication::palette();
        pal.setColor(QPalette::WindowText, Qt::white);
        QApplication::setPalette(pal);
    };
}

void Application::InitLogin(POE_API mode)
{
    if (test_mode_) {
        data_ = std::make_unique<MemoryDataStore>();
    } else {
        const std::string league = settings_->value("league").toString().toStdString();
        const std::string account = settings_->value("account").toString().toStdString();
        const QString data_dir = Filesystem::UserDir() + "/data/";
        const QString data_file = SqliteDataStore::MakeFilename(account, league);
        const QString data_path = data_dir + data_file;
        data_ = std::make_unique<SqliteDataStore>(data_path);
        SaveDbOnNewVersion();
    }

    rate_limiter_ = std::make_unique<RateLimiter>(this,
        network_manager(),
        oauth_manager(), mode);

    buyout_manager_ = std::make_unique<BuyoutManager>(
        data());

    items_manager_ = std::make_unique<ItemsManager>(this,
        settings(),
        network_manager(),
        repoe(),
        buyout_manager(),
        data(),
        rate_limiter());

    shop_ = std::make_unique<Shop>(this,
        settings(),
        network_manager(),
        data(),
        items_manager(),
        buyout_manager());

    currency_manager_ = std::make_unique<CurrencyManager>(nullptr,
        settings(),
        data(),
        items_manager());

    connect(items_manager_.get(), &ItemsManager::ItemsRefreshed, this, &Application::OnItemsRefreshed);

    if (test_mode_ == false) {
        items_manager_->Start(mode);
    };
}

void Application::OnItemsRefreshed(bool initial_refresh) {
    currency_manager_->Update();
    shop_->Update();
    if (!initial_refresh && shop_->auto_update())
        shop_->SubmitShopToForum();
}

void Application::OnRunTests() {
    if (!repoe_->IsInitialized()) {
        QMessageBox::information(nullptr,
            "Acquisition", "RePoE is not initialized yet. Try again later",
            QMessageBox::StandardButton::Ok,
            QMessageBox::StandardButton::Ok);
        return;
    };
    login_->hide();
    const int result = test_main();
    QMessageBox::information(nullptr,
        "Acquisition", "Testing returned " + QString::number(result),
        QMessageBox::StandardButton::Ok,
        QMessageBox::StandardButton::Ok);
    login_->show();
}

void Application::SaveDbOnNewVersion() {
    //If user updated from a 0.5c db to a 0.5d, db exists but no "version" in it
    std::string version = data_->Get("version", "0.5c");
    // We call this just after login, so we didn't pulled tabs for the first time ; so "tabs" shouldn't exist in the DB
    // This way we don't create an useless data_save_version folder on the first time you run acquisition

    bool first_start = data_->Get("tabs", "first_time") == "first_time" &&
        data_->GetTabs(ItemLocationType::STASH).size() == 0 &&
        data_->GetTabs(ItemLocationType::CHARACTER).size() == 0;
    if (version != APP_VERSION_STRING && !first_start) {
        QString data_path = Filesystem::UserDir() + QString("/data");
        QString save_path = data_path + "_save_" + version.c_str();
        QDir src(data_path);
        QDir dst(save_path);
        if (!dst.exists())
            QDir().mkpath(dst.path());
        for (const auto& name : src.entryList()) {
            QFile::copy(data_path + QDir::separator() + name, save_path + QDir::separator() + name);
        }
        QLOG_INFO() << "I've created the folder " << save_path << "in your acquisition folder, containing a save of all your data";
    }
    data_->Set("version", APP_VERSION_STRING);
}
