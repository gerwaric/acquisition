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

#pragma once

#include <QApplication>
#include <QDir>
#include <QMessageBox>
#include <QObject>
#include <QString>
#include <QDateTime>

class QNetworkAccessManager;
class QNetworkReply;
class QSettings;

class BuyoutManager;
class CurrencyManager;
class DataStore;
class ImageCache;
class ItemsManager;
class LoginDialog;
class MainWindow;
class OAuthManager;
class RateLimiter;
class RePoE;
class Shop;
class UpdateChecker;

enum class POE_API;

class Application : public QObject {
    Q_OBJECT
public:
    explicit Application(const QDir& appDataDir);
    ~Application();
    Application(const Application&) = delete;
    Application& operator=(const Application&) = delete;
    void Start();
    void InitLogin(POE_API api);
    QSettings& settings() const;
    ItemsManager& items_manager() const;
    DataStore& global_data() const;
    DataStore& data() const;
    BuyoutManager& buyout_manager() const;
    QNetworkAccessManager& network_manager() const;
    RePoE& repoe() const;
    Shop& shop() const;
    ImageCache& image_cache() const;
    CurrencyManager& currency_manager() const;
    UpdateChecker& update_checker() const;
    OAuthManager& oauth_manager() const;
    RateLimiter& rate_limiter() const;
public slots:
    void SetSessionId(const QString& poesessid);
    void SetTheme(const QString& theme);
    void SetUserDir(const QString& dir);
    void OnLogin(POE_API api);
    void OnItemsRefreshed(bool initial_refresh);
    void OnRunTests();
private:
    void Stop();
    void InitUserDir(const QString& dir);
    void InitCrashReporting();
    void SaveDbOnNewVersion();

    std::unique_ptr<QNetworkAccessManager> network_manager_;
    std::unique_ptr<QSettings> settings_;

    std::unique_ptr<RePoE> repoe_;
    std::unique_ptr<DataStore> global_data_;
    std::unique_ptr<DataStore> data_;
    std::unique_ptr<UpdateChecker> update_checker_;
    std::unique_ptr<OAuthManager> oauth_manager_;

    std::unique_ptr<BuyoutManager> buyout_manager_;
    std::unique_ptr<Shop> shop_;
    std::unique_ptr<ItemsManager> items_manager_;
    std::unique_ptr<CurrencyManager> currency_manager_;
    std::unique_ptr<RateLimiter> rate_limiter_;
    std::unique_ptr<ImageCache> image_cache_;

    std::unique_ptr<LoginDialog> login_;
    std::unique_ptr<MainWindow> main_window_;

    QDir data_dir_;
    QString active_theme_;
};
