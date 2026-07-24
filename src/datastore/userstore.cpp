// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2025 Tom Holz

#include "datastore/userstore.h"

#include <QSqlDriver>
#include <QSqlError>
#include <QSqlQuery>
#include <QUuid>

#include "datastore/buyoutrepo.h"
#include "datastore/characterrepo.h"
#include "datastore/datastore_utils.h"
#include "datastore/stashrepo.h"
#include "util/spdlog_qt.h" // IWYU pragma: keep

// Shape of the tables. Bump when the DDL changes and add a ladder step in
// migrate(). This is separate from json::PAYLOAD_VERSION, which versions the
// json stored inside them: a payload change needs no schema bump, and this
// one is compared with '<' (migrations replay forward) where the payload
// version is compared with '!=' (a downgrade must not misparse newer blobs).
static constexpr int SCHEMA_VERSION = 2;

constexpr unsigned int QSQLITE_BUSY_TIMEOUT{5000};

constexpr std::array CONNECTION_PRAGMAS{
    "PRAGMA busy_timeout=5000",
    "PRAGMA temp_store=MEMORY",
    "PRAGMA journal_mode=WAL",
    "PRAGMA synchronous=NORMAL",
    "PRAGMA foreign_keys=OFF",
};

UserStore::UserStore(const QDir &dir, const QString &username)
{
    const QString uuid = QUuid::createUuid().toString(QUuid::WithoutBraces);
    const QString connection = "UserStore:" + username + ":" + uuid;
    m_db = QSqlDatabase::addDatabase("QSQLITE", connection);

    m_characters = std::make_unique<CharacterRepo>(m_db);
    m_stashes = std::make_unique<StashRepo>(m_db);
    m_buyouts = std::make_unique<BuyoutRepo>(m_db);

    if (!m_db.isValid()) {
        spdlog::error("UserStore: database is not valid: {}", m_db.lastError().text());
        return;
    }

    QDir dataDir(dir);
    if (!dataDir.mkpath(dir.absolutePath())) {
        spdlog::error("UserStore: unable to create directory: {}", dir.absolutePath());
        return;
    }

    const QString filename = dataDir.absoluteFilePath("userstore-" + username + ".db");
    m_db.setDatabaseName(filename);
    m_db.setConnectOptions(QString("QSQLITE_BUSY_TIMEOUT=%1").arg(QSQLITE_BUSY_TIMEOUT));
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
    if (!q.exec("PRAGMA user_version")) {
        spdlog::error("UserStore: error getting user_version: {}", q.lastError().text());
    }
    return q.next() ? q.value(0).toInt() : 0;
}

void UserStore::migrate()
{
    QSqlQuery q(m_db);

    // Acquire a write lock so only one migrator proceeds.
    if (!q.exec("BEGIN IMMEDIATE")) {
        return;
    }

    // Another connection might have migrated while we waited.
    const int version = userVersion();
    if (version >= SCHEMA_VERSION) {
        spdlog::debug("UserStore: migration occured while waiting for the lock");
        m_db.commit();
        return;
    }

    // Two distinct cases, and they must not be mixed.
    //
    // Version 0 is a database with no schema of ours at all (a fresh file, or
    // one predating the repos). It is built directly at the current schema,
    // because the CREATE statements always carry every column — so it must
    // NOT then replay the ladder below, which would try to add columns the
    // CREATE just made (ALTER ADD COLUMN fails on a duplicate).
    //
    // Anything else is an existing database that must replay every step it
    // missed. Those steps test `version < N` rather than `version == N - 1`
    // so they compose: a database several versions behind runs all of them,
    // in order.
    if (version < 1) {
        if (!m_characters->resetRepo()) {
            m_db.rollback();
            return;
        }

        if (!m_stashes->resetRepo()) {
            m_db.rollback();
            return;
        }

        if (!m_buyouts->resetRepo()) {
            m_db.rollback();
            return;
        }

    } else {
        // 1 -> 2: add the payload version column. Existing rows get NULL,
        // which never equals json::PAYLOAD_VERSION, so their cached json is
        // treated as unfetched and refetched on the next refresh. That is the
        // intended outcome here: 3.29 turned implicitMods/explicitMods into
        // objects, so every blob written before this version is unreadable
        // anyway. No bulk UPDATE is needed, and no future payload bump will
        // need a migration at all — bumping the constant is enough to strand
        // stale rows.
        if (version < 2) {
            constexpr std::array statements{
                "ALTER TABLE stashes ADD COLUMN json_version INTEGER",
                "ALTER TABLE characters ADD COLUMN json_version INTEGER",
            };
            for (const auto &sql : statements) {
                if (!q.exec(sql)) {
                    ds::logQueryError("UserStore::migrate", q);
                    m_db.rollback();
                    return;
                }
            }
        }
    }

    // Update the user_version.
    if (!q.exec(QString("PRAGMA user_version=%1").arg(SCHEMA_VERSION))) {
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
