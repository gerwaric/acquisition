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

#pragma once

#include <QApplication>
#include <QDateTime>
#include <QDir>
#include <QMessageBox>
#include <QObject>
#include <QString>

class QNetworkReply;
class QSettings;

class BuyoutManager;
class CurrencyManager;
class DataStore;
class ImageCache;
class ItemsManager;
class LoginDialog;
class MainWindow;
class NetworkManager;
class OAuthManager;
struct OAuthToken;
class RateLimiter;
class RePoE;
class Shop;
class UserStore;
class UpdateChecker;

class Application : public QObject
{
    Q_OBJECT
public:
    explicit Application();
    ~Application();
    Application(const Application &) = delete;
    Application &operator=(const Application &) = delete;
    void InitLogin();

private:
    QSettings &settings() const;
    ItemsManager &items_manager() const;
    DataStore &global_data() const;
    DataStore &data() const;
    BuyoutManager &buyout_manager() const;
    NetworkManager &network_manager() const;
    RePoE &repoe() const;
    Shop &shop() const;
    ImageCache &image_cache() const;
    CurrencyManager &currency_manager() const;
    UpdateChecker &update_checker() const;
    OAuthManager &oauth_manager() const;
    RateLimiter &rate_limiter() const;

public slots:
    void Start(const QDir &appDataDir);
    void SetSessionId(const QString &poesessid);
    void SetTheme(const QString &theme);
    void SetUserDir(const QString &dir);
    void OnLogin();
    void OnItemsRefreshed(bool initial_refresh);

private:
    void Stop();
    void InitUserDir(const QString &dir);
    void InitCrashReporting();
    void SaveDbOnNewVersion();

    std::unique_ptr<QSettings> m_settings;

    std::unique_ptr<NetworkManager> m_network_manager;

    std::unique_ptr<UserStore> m_userstore;

    std::unique_ptr<RePoE> m_repoe;
    std::unique_ptr<DataStore> m_global_data;
    std::unique_ptr<DataStore> m_data;
    std::unique_ptr<UpdateChecker> m_update_checker;
    std::unique_ptr<OAuthManager> m_oauth_manager;

    std::unique_ptr<BuyoutManager> m_buyout_manager;
    std::unique_ptr<Shop> m_shop;
    std::unique_ptr<ItemsManager> m_items_manager;
    std::unique_ptr<CurrencyManager> m_currency_manager;
    std::unique_ptr<RateLimiter> m_rate_limiter;
    std::unique_ptr<ImageCache> m_image_cache;

    std::unique_ptr<LoginDialog> m_login;
    std::unique_ptr<MainWindow> m_main_window;

    QDir m_data_dir;
    QString m_active_theme;
};
