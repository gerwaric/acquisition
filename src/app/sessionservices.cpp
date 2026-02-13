// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#include "app/sessionservices.h"

#include <QSettings>
#include <QSqlDatabase>
#include <QSqlError>
#include <QSqlQuery>
#include <QUuid>

#include "app/coreservices.h"
#include "app/usersettings.h"
#include "buyoutmanager.h"
#include "datastore/buyoutstore.h"
#include "datastore/characterstore.h"
#include "datastore/sessionstore.h"
#include "datastore/stashstore.h"
#include "datastore/userstore.h"
#include "itemsmanager.h"
#include "itemsmanagerworker.h"
#include "ratelimit/ratelimiter.h"
#include "shop.h"
#include "util/deref.h"
#include "util/json_utils.h"
#include "util/spdlog_qt.h" // IWYU pragma: keep

app::SessionServices::SessionServices(app::UserSettings &settings, app::CoreServices &core)
    : m_settings(settings)
{
    const QString username = m_settings.username();
    const QString uuid = QUuid::createUuid().toString(QUuid::WithoutBraces);
    m_connName = QString("acquisition:%1:%2").arg(username, uuid);

    spdlog::trace("Application::InitLogin() entered");
    initDatabase();
    createChildren(core);
    connectChildren();
}

QString app::SessionServices::sessionKey() const
{
    return QString("%1/%2/%3").arg(m_settings.username(), m_settings.realm(), m_settings.league());
}

app::SessionServices::~SessionServices() {}

void app::SessionServices::initDatabase()
{
    QSqlDatabase db = QSqlDatabase::addDatabase("QSQLITE", m_connName);
    if (!db.isValid()) {
        spdlog::error("UserStore: database is not valid: {}", db.lastError().text());
        return;
    }

    QDir dir = m_settings.userDir();

    if (!dir.mkpath(dir.absolutePath())) {
        spdlog::error("UserStore: unable to create directory: {}", dir.absolutePath());
        return;
    }

    const QString username = m_settings.username();
    const QString filename = dir.absoluteFilePath("userstore-" + username + ".db");
    db.setDatabaseName(filename);
    db.setConnectOptions(QString("QSQLITE_BUSY_TIMEOUT=%1;").arg(QSQLITE_BUSY_TIMEOUT));
    spdlog::debug("UserStore: created database connection '{}' to '{}'",
                  db.connectionName(),
                  db.databaseName());

    if (!m_db->open()) {
        spdlog::error("UserStore: error opening database connection '{}' to '{}': {}",
                      db.connectionName(),
                      db.databaseName(),
                      db.lastError().text());
        return;
    }

    QSqlQuery q(db);
    for (const auto &pragma : CONNECTION_PRAGMAS) {
        if (!q.exec(pragma)) {
            spdlog::warn("UserStore: pragma failed: {} ({})", pragma, q.lastError().text());
        }
    }

    const int version = userVersion();
    spdlog::debug("UserStore: user_version is {}, schema version is {}", version, SCHEMA_VERSION);
    if (version < SCHEMA_VERSION) {
        spdlog::info("UserStore: migrating from user_version {} to {}", version, SCHEMA_VERSION);
        migrate();
    }
}

void app::SessionServices::createChildren(app::CoreServices &core)
{
    const QString account = m_settings.username();
    const QString realm = m_settings.realm();
    const QString league = m_settings.league();

    spdlog::debug("UserSession: realm='{}', league='{}', account='{}'", realm, league, account);

    QDir data_dir = m_settings.userDir().filePath("data");
    //const QString data_file = SqliteDataStore::MakeFilename(account, league);
    //const QString data_path = core.data_dir().filePath("data/" + data_file);
    //spdlog::trace("Application::InitLogin() data_path = {}", data_path);
    //m_data = new SqliteDataStore(realm, league, data_path, this);

    m_session_store = std::make_unique<SessionStore>(m_connName, m_settings);
    m_stash_store = std::make_unique<StashStore>(m_connName);
    m_character_store = std::make_unique<CharacterStore>(m_connName);
    m_buyout_store = std::make_unique<BuyoutStore>(m_connName);

    //spdlog::trace("Application::InitLogin() creating user datastore");
    //m_userstore = std::make_unique<UserStore>(data_dir, account);

    spdlog::trace("Application::InitLogin() creating rate limiter");
    m_rate_limiter = std::make_unique<RateLimiter>(core.network_manager());

    spdlog::trace("Application::InitLogin() creating buyout manager");
    m_buyout_manager = std::make_unique<BuyoutManager>(m_settings,
                                                       *m_session_store,
                                                       *m_buyout_store);

    spdlog::trace("Application::InitLogin() creating items manager");
    m_items_manager = std::make_unique<ItemsManager>(m_settings, buyout_manager(), *m_session_store);

    spdlog::trace("Application::InitLogin() creating items worker");
    m_items_worker = std::make_unique<ItemsManagerWorker>(m_settings,
                                                          buyout_manager(),
                                                          rate_limiter());

    spdlog::trace("Application::InitLogin() creating shop");
    m_shop = std::make_unique<Shop>(m_settings,
                                    core.network_manager(),
                                    *m_rate_limiter,
                                    *m_session_store,
                                    *m_items_manager,
                                    *m_buyout_manager);
}

void app::SessionServices::connectChildren()
{
    auto items_manager = m_items_manager.get();
    auto items_worker = m_items_worker.get();
    auto buyout_manager = m_buyout_manager.get();

    connect(items_manager, &ItemsManager::UpdateSignal, items_worker, &ItemsManagerWorker::Update);
    connect(items_worker,
            &ItemsManagerWorker::StatusUpdate,
            items_manager,
            &ItemsManager::OnStatusUpdate);
    connect(items_worker,
            &ItemsManagerWorker::ItemsRefreshed,
            items_manager,
            &ItemsManager::OnItemsRefreshed);

    auto &character_repo = userstore().characters();
    connect(items_worker,
            &ItemsManagerWorker::characterListReceived,
            &character_repo,
            &CharacterStore::saveCharacterList);
    connect(items_worker,
            &ItemsManagerWorker::characterReceived,
            &character_repo,
            &CharacterStore::saveCharacter);

    auto &stash_repo = userstore().stashes();
    connect(items_worker,
            &ItemsManagerWorker::stashListReceived,
            &stash_repo,
            &StashStore::saveStashList);
    connect(items_worker, &ItemsManagerWorker::stashReceived, &stash_repo, &StashStore::saveStash);

    //auto state = &userstore().state();

    auto &buyout_repo = userstore().buyouts();
    connect(buyout_manager,
            &BuyoutManager::SetItemBuyout,
            &buyout_repo,
            &BuyoutStore::saveItemBuyout);
    connect(buyout_manager,
            &BuyoutManager::SetLocationBuyout,
            &buyout_repo,
            &BuyoutStore::saveLocationBuyout);

    //connect(buyout_mgr, &BuyoutManager::SetCheckRefresh, state, &StateRepo::setRefreshFlag);

    connect(items_manager,
            &ItemsManager::ItemsRefreshed,
            this,
            &app::SessionServices::itemsRefreshed);
}

/*
DataStore &app::SessionServices::data()
{
    return deref(m_data, "data");
}
*/

UserStore &app::SessionServices::userstore()
{
    return deref(m_userstore, "SessionServices::userstore()");
}

RateLimiter &app::SessionServices::rate_limiter()
{
    return deref(m_rate_limiter, "SessionServices::rate_limiter()");
}

BuyoutManager &app::SessionServices::buyout_manager()
{
    return deref(m_buyout_manager, "SessionServices::buyout_manager()");
}

ItemsManager &app::SessionServices::items_manager()
{
    return deref(m_items_manager, "SessionServices::items_manager()");
}

ItemsManagerWorker &app::SessionServices::items_worker()
{
    return deref(m_items_worker, "SessionServices::items_worker()");
}

Shop &app::SessionServices::shop()
{
    return deref(m_shop, "SessionServices::shop()");
}

void app::SessionServices::itemsRefreshed(bool initial_refresh)
{
    spdlog::trace("Application::OnItemsRefreshed() initial_refresh = {}", initial_refresh);

    shop().ExpireShopData();
    if (!initial_refresh && shop().auto_update()) {
        spdlog::trace("Application::OnItemsRefreshed() submitting shops");
        shop().SubmitShopToForum();
    }
}
