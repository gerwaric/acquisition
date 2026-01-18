// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2025 Tom Holz

#include "datastore/userstore.h"

#include <QSqlDatabase>
#include <QSqlDriver>
#include <QSqlError>
#include <QSqlQuery>
#include <QUuid>

#include "datastore/buyoutrepo.h"
#include "datastore/characterrepo.h"
#include "datastore/datastore_utils.h"
#include "datastore/stashrepo.h"
#include "util/spdlog_qt.h" // IWYU pragma: keep

static constexpr int SCHEMA_VERSION = 1;

constexpr unsigned int QSQLITE_BUSY_TIMEOUT{5000};

constexpr std::array CONNECTION_PRAGMAS{
    "PRAGMA busy_timeout=5000",
    "PRAGMA temp_store=MEMORY",
    "PRAGMA journal_mode=WAL",
    "PRAGMA synchronous=NORMAL",
    "PRAGMA foreign_keys=OFF",
};

struct UserStore::Impl
{
    Impl(const QString &username);
    ~Impl();

    int userVersion();
    bool setUserVersion(int v);
    void migrate();

    QSqlDatabase db;
    std::unique_ptr<BuyoutRepo> buyouts;
    std::unique_ptr<CharacterRepo> characters;
    std::unique_ptr<StashRepo> stashes;
};

UserStore::UserStore(const QDir &dir, const QString &username)
{
    m_impl = std::make_unique<UserStore::Impl>(username);

    QDir dataDir(dir);
    if (!dataDir.mkpath(dir.absolutePath())) {
        spdlog::error("UserStore: unable to create directory: {}", dir.absolutePath());
        return;
    }
    const QString filename = dataDir.absoluteFilePath("userstore-" + username + ".db");

    QSqlDatabase &db = m_impl->db;
    db.setDatabaseName(filename);
    db.setConnectOptions(QString("QSQLITE_BUSY_TIMEOUT=%1").arg(QSQLITE_BUSY_TIMEOUT));
    spdlog::debug("UserStore: created database connection '{}' to '{}'",
                  db.connectionName(),
                  db.databaseName());

    if (!db.open()) {
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

    const int v = m_impl->userVersion();
    spdlog::debug("UserStore: user_version is {}, schema version is {}", v, SCHEMA_VERSION);
    if (v < SCHEMA_VERSION) {
        spdlog::info("UserStore: migrating from user_version {} to {}", v, SCHEMA_VERSION);
        m_impl->migrate();
    }
}

UserStore::~UserStore() = default;

BuyoutRepo &UserStore::buyouts()
{
    return *m_impl->buyouts;
}

CharacterRepo &UserStore::characters()
{
    return *m_impl->characters;
}

StashRepo &UserStore::stashes()
{
    return *m_impl->stashes;
}

UserStore::Impl::Impl(const QString &username)
{
    const QString uuid = QUuid::createUuid().toString(QUuid::WithoutBraces);
    const QString connection = "UserStore:" + username + ":" + uuid;

    // Create the database connection and repos. They will not be usable
    // until the connection has been opened, but this prevents null pointer
    // dereference errors.
    db = QSqlDatabase::addDatabase("QSQLITE", connection);
    buyouts = std::make_unique<BuyoutRepo>(db);
    characters = std::make_unique<CharacterRepo>(db);
    stashes = std::make_unique<StashRepo>(db);
}

UserStore::Impl::~Impl()
{
    // Grab the connection name.
    const QString connection = db.connectionName();

    // Close the database.
    if (db.isValid()) {
        db.close();
    }

    // Clear member variables.
    db = QSqlDatabase();
    characters = nullptr;
    stashes = nullptr;

    // Remove the database connection.
    if (QSqlDatabase::contains(connection)) {
        QSqlDatabase::removeDatabase(connection);
    }
}

int UserStore::Impl::userVersion()
{
    QSqlQuery q(db);
    if (!q.exec("PRAGMA user_version")) {
        spdlog::error("UserStore: error getting user_version: {}", q.lastError().text());
    }
    return q.next() ? q.value(0).toInt() : 0;
}

bool UserStore::Impl::setUserVersion(int v)
{
    QSqlQuery q(db);
    const bool ok = q.exec(QString("PRAGMA user_version=%1").arg(v));
    if (!ok) {
        spdlog::error("UserStore: error setting user_version: {}", q.lastError().text());
    }
    return ok;
}

void UserStore::Impl::migrate()
{
    QSqlQuery q(db);

    // Acquire a write lock so only one migrator proceeds.
    if (!q.exec("BEGIN IMMEDIATE")) {
        return;
    }

    // Another connection might have migrated while we waited.
    const int v = userVersion();
    if (v >= SCHEMA_VERSION) {
        spdlog::debug("UserStore: migration occured while waiting for the lock");
        db.commit();
        return;
    }

    if (!characters->reset()) {
        db.rollback();
        return;
    }

    if (!stashes->reset()) {
        db.rollback();
        return;
    }

    // Update the user_version.
    if (!setUserVersion(SCHEMA_VERSION)) {
        spdlog::error("UserStore: unable to set user_version to {}", SCHEMA_VERSION);
        db.rollback();
        return;
    }

    // Commit the transaction.
    if (!db.commit()) {
        spdlog::error("UserStore: error committing migration: {}", db.lastError().text());
        return;
    }

    spdlog::info("UserStore: migrated from version {} to {}", v, SCHEMA_VERSION);
}
