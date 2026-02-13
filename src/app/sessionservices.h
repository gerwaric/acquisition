// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#pragma once

#include <QObject>

class QSqlDatabase;

class UserStore;
class RateLimiter;
class BuyoutManager;
class ItemsManager;
class ItemsManagerWorker;
class Shop;
class MainWindow;

class SessionStore;
class StashStore;
class CharacterStore;
class BuyoutStore;

namespace app {

    class UserSettings;
    class CoreServices;

    class SessionServices : public QObject
    {
        Q_OBJECT
    public:
        explicit SessionServices(app::UserSettings &settings, app::CoreServices &core);
        ~SessionServices();

        UserStore &userstore();
        RateLimiter &rate_limiter();
        BuyoutManager &buyout_manager();
        ItemsManager &items_manager();
        ItemsManagerWorker &items_worker();
        Shop &shop();

    public slots:
        void itemsRefreshed(bool initial_refresh);

    private:
        void initDatabase();
        void createChildren(app::CoreServices &core);
        void connectChildren();

        QString sessionKey() const;

        app::UserSettings &m_settings;

        QString m_connName;

        std::unique_ptr<SessionStore> m_session_store;
        std::unique_ptr<StashStore> m_stash_store;
        std::unique_ptr<CharacterStore> m_character_store;
        std::unique_ptr<BuyoutStore> m_buyout_store;

        std::unique_ptr<UserStore> m_userstore;
        std::unique_ptr<RateLimiter> m_rate_limiter;
        std::unique_ptr<BuyoutManager> m_buyout_manager;
        std::unique_ptr<ItemsManager> m_items_manager;
        std::unique_ptr<ItemsManagerWorker> m_items_worker;
        std::unique_ptr<Shop> m_shop;
    };

} // namespace app
