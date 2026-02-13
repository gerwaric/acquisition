// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2025 Tom Holz

#include "datastore/userstore.h"

#include <QSqlDriver>
#include <QSqlError>
#include <QSqlQuery>
#include <QUuid>

#include "datastore/buyoutstore.h"
#include "datastore/characterstore.h"
#include "datastore/datarepo.h"
#include "datastore/datastore_utils.h"
#include "datastore/stashstore.h"
#include "util/spdlog_qt.h" // IWYU pragma: keep

static constexpr int SCHEMA_VERSION = 3;

constexpr unsigned int QSQLITE_BUSY_TIMEOUT{5000};

constexpr std::array CONNECTION_PRAGMAS{
    "PRAGMA busy_timeout=5000;",
    "PRAGMA temp_store=MEMORY;",
    "PRAGMA journal_mode=WAL;",
    "PRAGMA synchronous=NORMAL;",
    "PRAGMA foreign_keys=OFF;",
};

UserStore::UserStore(const QDir &dir, const QString &username)
    : m_username(username)
    , m_dir(dir)
{
    const QString uuid = QUuid::createUuid().toString(QUuid::WithoutBraces);
    const QString connection = "UserStore:" + m_username + ":" + uuid;
    m_db = QSqlDatabase::addDatabase("QSQLITE", connection);

    m_data = new DataRepo(m_db, this);
    m_stashes = new StashRepo(m_db, this);
    m_characters = new CharacterStore(m_db, this);
    m_buyouts = new BuyoutStore(m_db, this);

    if (!m_db.isValid()) {
        spdlog::error("UserStore: database is not valid: {}", m_db.lastError().text());
        return;
    }

    if (!m_dir.mkpath(m_dir.absolutePath())) {
        spdlog::error("UserStore: unable to create directory: {}", m_dir.absolutePath());
        return;
    }

    const QString filename = m_dir.absoluteFilePath("userstore-" + m_username + ".db");
    m_db.setDatabaseName(filename);
    m_db.setConnectOptions(QString("QSQLITE_BUSY_TIMEOUT=%1;").arg(QSQLITE_BUSY_TIMEOUT));
    spdlog::debug("UserStore: created database connection '{}' to '{}'",
                  m_db.connectionName(),
                  m_db.databaseName());

    if (!m_db.open()) {
        spdlog::error("UserStore: error opening database connection '{}' to '{}': {}",
                      m_db.connectionName(),
                      m_db.databaseName(),
                      m_db.lastError().text());
        return;
    }

    QSqlQuery q(m_db);
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

UserStore::~UserStore()
{
    m_buyouts = nullptr;
    m_characters = nullptr;
    m_stashes = nullptr;

    // Close the database.
    if (m_db.isValid()) {
        m_db.close();
    }

    // Grab the connection name.
    const QString connection = m_db.connectionName();

    // Clear member variables.
    m_db = QSqlDatabase();

    // Remove the database connection.
    if (QSqlDatabase::contains(connection)) {
        QSqlDatabase::removeDatabase(connection);
    }
}

int UserStore::userVersion()
{
    QSqlQuery q(m_db);
    if (!q.exec("PRAGMA user_version;")) {
        spdlog::error("UserStore: error getting user_version: {}", q.lastError().text());
    }
    return q.next() ? q.value(0).toInt() : 0;
}

void UserStore::migrate()
{
    QSqlQuery q(m_db);

    // Acquire a write lock so only one migrator proceeds.
    if (!q.exec("BEGIN IMMEDIATE;")) {
        return;
    }

    // Another connection might have migrated while we waited.
    const int version = userVersion();
    if (version >= SCHEMA_VERSION) {
        spdlog::debug("UserStore: migration occured while waiting for the lock");
        m_db.commit();
        return;
    }

    if (!m_data->resetRepo()) {
        ds::logQueryError("UserStore::migrate", q);
        m_db.rollback();
        return;
    }

    if (!m_characters->resetRepo()) {
        ds::logQueryError("UserStore::migrate", q);
        m_db.rollback();
        return;
    }

    if (!m_stashes->resetRepo()) {
        ds::logQueryError("UserStore::migrate", q);
        m_db.rollback();
        return;
    }

    if (!m_buyouts->resetRepo()) {
        ds::logQueryError("UserStore::migrate", q);
        m_db.rollback();
        return;
    }

    // Update the user_version.
    if (!q.exec(QString("PRAGMA user_version=%1;").arg(SCHEMA_VERSION))) {
        spdlog::error("UserStore: error setting user_version: {}", q.lastError().text());
        m_db.rollback();
        return;
    }

    // Commit the transaction.
    if (!m_db.commit()) {
        spdlog::error("UserStore: error committing migration: {}", m_db.lastError().text());
        return;
    }

    spdlog::info("UserStore: migrated from version {} to {}", version, SCHEMA_VERSION);
}
