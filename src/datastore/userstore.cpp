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

#include <QSqlError>
#include <QSqlQuery>
#include <QStringLiteral>
#include <QUuid>

#include "userstore.h"
#include "util/json_readers.h"
#include "util/json_writers.h"
#include "util/spdlog_qt.h"

static_assert(ACQUISITION_USE_SPDLOG);

//================================================================================

static constexpr int SCHEMA_VERSION = 1;

constexpr unsigned int QSQLITE_BUSY_TIMEOUT{5000};

constexpr std::array CONNECTION_PRAGMAS{
    "PRAGMA busy_timeout=5000;",
    "PRAGMA temp_store=MEMORY;",
    "PRAGMA journal_mode=WAL;",
    "PRAGMA synchronous=NORMAL;",
    "PRAGMA foreign_keys=ON;",
};

// ---------- LISTS ----------

constexpr const char *CREATE_LISTS_TABLE{
    R"(CREATE TABLE IF NOT EXISTS lists (
        name            TEXT NOT NULL,
        realm           TEXT NOT NULL,
        league          TEXT NOT NULL,
        timestamp       INTEGER NOT NULL,
        data            TEXT NOT NULL,
        PRIMARY KEY (name, realm, league)
    ) WITHOUT ROWID;)"};

constexpr const char *INSERT_LIST{
    R"(INSERT INTO lists (
        name,
        realm,
        league,
        timestamp,
        data
    ) VALUES (
        :name,
        :realm,
        :league,
        :timestamp,
        :data
    )
    ON CONFLICT(name, realm, league)
    DO UPDATE SET
        timestamp = excluded.timestamp,
        data      = excluded.data;)"};

constexpr const char *SELECT_LIST{
    R"(SELECT data, timestamp
    FROM lists
    WHERE name = :name AND realm = :realm AND league = :league;)"};

// ---------- CHARACTERS ----------

constexpr const char *CREATE_CHARACTER_TABLE{
    R"(CREATE TABLE IF NOT EXISTS characters (
        id              TEXT PRIMARY KEY,
        name            TEXT NOT NULL,
        realm           TEXT NOT NULL,
        class           TEXT NOT NULL,
        league          TEXT,
        level           INTEGER NOT NULL,
        experience      INTEGER NOT NULL,
        ruthless        INTEGER NOT NULL DEFAULT 0 CHECK (ruthless IN (0,1)),
        expired         INTEGER NOT NULL DEFAULT 0 CHECK (expired IN (0,1)),
        deleted         INTEGER NOT NULL DEFAULT 0 CHECK (deleted IN (0,1)),
        current         INTEGER NOT NULL DEFAULT 0 CHECK (current IN (0,1)),
        meta_version    TEXT,
        timestamp       INTEGER NOT NULL,
        data            TEXT NOT NULL
    ) WITHOUT ROWID;)"};

constexpr const char *INSERT_CHARACTER{
    R"(INSERT INTO characters (
        id,
        name,
        realm,
        class,
        league,
        level,
        experience,
        ruthless,
        expired,
        deleted,
        current,
        meta_version,
        timestamp,
        data
    ) VALUES (
        :id,
        :name,
        :realm,
        :class,
        :league,
        :level,
        :experience,
        :ruthless,
        :expired,
        :deleted,
        :current,
        :meta_version,
        :timestamp,
        :data
    )
    ON CONFLICT(id) DO UPDATE SET
      name         = excluded.name,
      realm        = excluded.realm,
      class        = excluded.class,
      league       = excluded.league,
      level        = excluded.level,
      experience   = excluded.experience,
      ruthless     = excluded.ruthless,
      expired      = excluded.expired,
      deleted      = excluded.deleted,
      current      = excluded.current,
      meta_version = excluded.meta_version,
      timestamp    = excluded.timestamp,
      data         = excluded.data;)"};

constexpr const char *SELECT_CHARACTER{
    "SELECT data, timestamp FROM characters WHERE name = :name;"};

// ---------- STASHES ----------

constexpr const char *CREATE_STASH_TABLE{
    R"(CREATE TABLE IF NOT EXISTS stashes (
        id              TEXT PRIMARY KEY,
        realm           TEXT NOT NULL,
        league          TEXT NOT NULL,
        parent          TEXT,
        folder          TEXT,
        name            TEXT NOT NULL,
        type            TEXT NOT NULL,
        indx            INTEGER,
        meta_public     INTEGER NOT NULL DEFAULT 0 CHECK (meta_public IN (0,1)),
        meta_folder     INTEGER NOT NULL DEFAULT 0 CHECK (meta_folder IN (0,1)),
        meta_colour     TEXT,
        timestamp       INTEGER NOT NULL,
        data            TEXT NOT NULL,
        FOREIGN KEY(parent) REFERENCES stashes(id) ON DELETE SET NULL,
        FOREIGN KEY(folder) REFERENCES stashes(id) ON DELETE SET NULL
    ) WITHOUT ROWID;)"};

constexpr std::array
    CREATE_STASH_INDEXES{"CREATE INDEX IF NOT EXISTS idx_stashes_realm  ON stashes(realm);",
                         "CREATE INDEX IF NOT EXISTS idx_stashes_league ON stashes(league);",
                         "CREATE INDEX IF NOT EXISTS idx_stashes_parent ON stashes(parent);",
                         "CREATE INDEX IF NOT EXISTS idx_stashes_folder ON stashes(folder);",
                         "CREATE INDEX IF NOT EXISTS idx_stashes_type   ON stashes(type);"};

constexpr const char *INSERT_STASH =
    R"(INSERT INTO stashes (
        id,
        realm,
        league,
        parent,
        folder,
        name,
        type,
        indx,
        meta_public,
        meta_folder,
        meta_colour,
        timestamp,
        data
    ) VALUES (
        :id,
        :realm,
        :league,
        :parent,
        :folder,
        :name,
        :type,
        :indx,
        :meta_public,
        :meta_folder,
        :meta_colour,
        :timestamp,
        :data
    )
    ON CONFLICT(id) DO UPDATE SET
        realm       = excluded.realm,
        league      = excluded.league,
        parent      = excluded.parent,
        folder      = excluded.folder,
        name        = excluded.name,
        type        = excluded.type,
        indx        = excluded.indx,
        meta_public = excluded.meta_public,
        meta_folder = excluded.meta_folder,
        meta_colour = excluded.meta_colour,
        timestamp   = excluded.timestamp,
        data        = excluded.data;)";

constexpr const char *SELECT_STASH{"SELECT data, timestamp FROM stashes WHERE id = :id;"};

//================================================================================

UserStore::UserStore(const QDir &dir, const QString &username, QObject *parent)
    : QObject(parent)
{
    QDir dataDir(dir);
    if (!dataDir.mkpath(dir.absolutePath())) {
        spdlog::error("UserStore: unable to create directory: {}", dir.absolutePath());
        return;
    }

    const auto uuid = QUuid::createUuid().toString(QUuid::WithoutBraces);

    m_connection = QString("userdata:%1:%2").arg(username, uuid);
    m_filename = dataDir.absoluteFilePath("userstore-" + username + ".db");

    spdlog::debug("Creating DataStore {}: '{}'", m_connection, m_filename);

    m_db = QSqlDatabase::addDatabase("QSQLITE", m_connection);
    m_db.setDatabaseName(m_filename);
    m_db.setConnectOptions(QString("QSQLITE_BUSY_TIMEOUT=%1").arg(QSQLITE_BUSY_TIMEOUT));

    if (!m_db.open()) {
        const QString msg = m_db.lastError().text();
        spdlog::error("UserStore: error opening database: '{}': {}", m_filename, msg);
        return;
    }

    QSqlQuery q(m_db);
    for (const auto &pragma : CONNECTION_PRAGMAS) {
        if (!q.exec(pragma)) {
            spdlog::warn("UserStore: pragma failed: {} ({})", pragma, q.lastError().text());
        }
    }

    const int v = userVersion();
    spdlog::debug("UserStore: user_version is {}, schema version is {}", v, SCHEMA_VERSION);
    if (v < SCHEMA_VERSION) {
        spdlog::info("UserStore: migrating from user_version {} to {}", v, SCHEMA_VERSION);
        migrate();
    }
}

UserStore::~UserStore()
{
    if (m_db.isValid()) {
        m_db.close();
    }
    m_db = QSqlDatabase();
    QSqlDatabase::removeDatabase(m_connection);
}

int UserStore::userVersion()
{
    QSqlQuery q(m_db);
    if (!q.exec("PRAGMA user_version;")) {
        spdlog::error("UserStore: error getting user_version: {}", q.lastError().text());
    }
    return q.next() ? q.value(0).toInt() : 0;
}

bool UserStore::setUserVersion(int v)
{
    QSqlQuery q(m_db);
    const bool ok = q.exec(QString("PRAGMA user_version=%1;").arg(v));
    if (!ok) {
        spdlog::error("UserStore: error setting user_version: {}", q.lastError().text());
    }
    return ok;
}

void UserStore::migrate()
{
    QSqlQuery q(m_db);

    // Acquire a write lock so only one migrator proceeds.
    if (!q.exec("BEGIN IMMEDIATE;")) {
        spdlog::error("UserStore: unable to acquire write lock: {}", q.lastError().text());
        return;
    }

    // Another connection might have migrated while we waited.
    const int v = userVersion();
    if (v >= SCHEMA_VERSION) {
        spdlog::debug("UserStore: migration occured while waiting for the lock");
        q.exec("COMMIT;");
        return;
    }

    constexpr const std::array tables{
        std::make_pair("lists", CREATE_LISTS_TABLE),
        std::make_pair("characters", CREATE_CHARACTER_TABLE),
        std::make_pair("stashes", CREATE_STASH_TABLE),
    };

    // Create tables.
    for (const auto &[name, sql] : tables) {
        spdlog::debug("UserStore: setting up table: {}", name);

        if (!q.exec(QStringLiteral("DROP TABLE IF EXISTS '%1'").arg(name))) {
            const auto msg = q.lastError().text();
            spdlog::error("UserStore: error deleting table: '{}': {}", name, msg);
            q.exec("ROLLBACK;");
            return;
        }

        if (!q.exec(sql)) {
            const auto msg = q.lastError().text();
            spdlog::error("UserStore: error creating table: '{}': {}", sql, msg);
            q.exec("ROLLBACK;");
            return;
        }
    }

    // Create indexes.
    for (const auto &sql : CREATE_STASH_INDEXES) {
        if (!q.exec(sql)) {
            const auto msg = q.lastError().text();
            spdlog::error("UserStore: error creating index: '{}': {}", sql, msg);
            q.exec("ROLLBACK;");
            return;
        }
    }

    if (!setUserVersion(SCHEMA_VERSION)) {
        spdlog::error("UserStore: unable to set user_version to {}", SCHEMA_VERSION);
        q.exec("ROLLBACK;");
        return;
    }

    if (!q.exec("COMMIT;")) {
        spdlog::error("UserStore: error committing migration: {}", q.lastError().text());
        return;
    }

    spdlog::info("UserStore: migrated from version {} to {}", v, SCHEMA_VERSION);
}

void UserStore::saveCharacterList(const std::vector<poe::Character> &characters,
                                  const QString &realm)
{
    spdlog::debug("UserStore: saving character list: realm='{}', size={}", realm, characters.size());

    const QByteArray json = writeCharacterList(characters);

    QSqlQuery q(m_db);
    q.prepare(INSERT_LIST);
    q.bindValue(":name", "characters");
    q.bindValue(":realm", realm);
    q.bindValue(":league", "*");
    q.bindValue(":timestamp", QDateTime::currentMSecsSinceEpoch());
    q.bindValue(":data", json);

    if (!q.exec()) {
        spdlog::error("UserStore: error saving character list: {}", q.lastError().text());
    }
}

void UserStore::saveStashList(const std::vector<poe::StashTab> &stashes,
                              const QString &realm,
                              const QString &league)
{
    spdlog::debug("UserStore: saving stash list: realm='{}', league='{}', size={}",
                  realm,
                  league,
                  stashes.size());

    const QByteArray json = writeStashList(stashes);

    QSqlQuery q(m_db);
    q.prepare(INSERT_LIST);
    q.bindValue(":name", "stashes");
    q.bindValue(":realm", realm);
    q.bindValue(":league", league);
    q.bindValue(":timestamp", QDateTime::currentMSecsSinceEpoch());
    q.bindValue(":data", json);

    if (!q.exec()) {
        spdlog::error("UserStore: error saving stash list: {}", q.lastError().text());
        return;
    }
}

std::vector<poe::Character> UserStore::getCharacterList(const QString &realm)
{
    spdlog::debug("UserStore: getting character list: realm='{}'", realm);

    const QString name{"characters"};
    const QString league{"*"};

    QSqlQuery q(m_db);
    q.prepare(SELECT_LIST);
    q.bindValue(":name", name);
    q.bindValue(":realm", realm);
    q.bindValue(":league", league);

    if (!q.exec()) {
        spdlog::error("UserStore: error selecting character list: {}", q.lastError().text());
        return {};
    }

    if (!q.next()) {
        spdlog::debug("No rows. active={} at={} isSelect={}", q.isActive(), q.at(), q.isSelect());
        return {};
    }

    const QVariant data = q.value("data");
    return readCharacterList(data.toByteArray());
}

std::vector<poe::StashTab> UserStore::getStashList(const QString &realm, const QString &league)
{
    spdlog::debug("UserStore: getting stash list: realm='{}', league='{}'", realm, league);

    const QString name{"stashes"};

    QSqlQuery q(m_db);
    q.prepare(SELECT_LIST);
    q.bindValue(":name", name);
    q.bindValue(":realm", realm);
    q.bindValue(":league", league);

    if (!q.exec()) {
        spdlog::error("UserStore: error selecting stash list: {}", q.lastError().text());
        return {};
    }

    if (!q.next()) {
        spdlog::debug("No rows. active={} at={} isSelect={}", q.isActive(), q.at(), q.isSelect());
        return {};
    }

    const QVariant data = q.value("data");
    return readStashList(data.toByteArray());
}

void UserStore::saveCharacter(const poe::Character &character)
{
    spdlog::debug("UserStore: saving character: realm='{}', league='{}', id='{}', name='{}'",
                  character.realm,
                  character.league.value_or(""),
                  character.id,
                  character.name);

    const QByteArray json = writeCharacter(character);
    const QVariant version = (character.metadata && character.metadata->version)
                                 ? *character.metadata->version
                                 : QVariant();

    QSqlQuery q(m_db);

    q.prepare(INSERT_CHARACTER);
    q.bindValue(":id", character.id);
    q.bindValue(":name", character.name);
    q.bindValue(":realm", character.realm);
    q.bindValue(":class", character.class_);
    q.bindValue(":league", character.league ? *character.league : QVariant());
    q.bindValue(":level", character.level);
    q.bindValue(":experience", character.experience);
    q.bindValue(":ruthless", character.ruthless.value_or(false));
    q.bindValue(":expired", character.expired.value_or(false));
    q.bindValue(":deleted", character.deleted.value_or(false));
    q.bindValue(":current", character.current.value_or(false));
    q.bindValue(":meta_version", version);
    q.bindValue(":timestamp", QDateTime::currentMSecsSinceEpoch());
    q.bindValue(":data", json);

    if (!q.exec()) {
        spdlog::error("UserStore: error saving character: {}", q.lastError().text());
        return;
    }
}

void UserStore::saveStash(const poe::StashTab &stash, const QString &realm, const QString &league)
{
    spdlog::debug("UserStore: saving stash: realm='{}', league='{}', id='{}', name='{}'",
                  realm,
                  league,
                  stash.id,
                  stash.name);

    const QByteArray json = writeStash(stash);

    QSqlQuery q(m_db);

    q.prepare(INSERT_STASH);
    q.bindValue(":id", stash.id);
    q.bindValue(":realm", realm);
    q.bindValue(":league", league);
    q.bindValue(":parent", stash.parent ? *stash.parent : QVariant());
    q.bindValue(":folder", stash.folder ? *stash.folder : QVariant());
    q.bindValue(":name", stash.name);
    q.bindValue(":type", stash.type);
    q.bindValue(":indx", stash.index ? *stash.index : QVariant());
    q.bindValue(":meta_public", stash.metadata.public_.value_or(false));
    q.bindValue(":meta_folder", stash.metadata.folder.value_or(false));
    q.bindValue(":meta_colour", stash.metadata.colour ? *stash.metadata.colour : QVariant());
    q.bindValue(":timestamp", QDateTime::currentMSecsSinceEpoch());
    q.bindValue(":data", json);

    if (!q.exec()) {
        spdlog::error("UserStore: error saving stash: {}", q.lastError().text());
        return;
    }
}

poe::Character UserStore::getCharacter(const QString &name)
{
    spdlog::debug("UserStore: getting character: name='{}'", name);

    QSqlQuery q(m_db);
    q.prepare(SELECT_CHARACTER);
    q.bindValue(":name", name);

    if (!q.exec()) {
        spdlog::error("UserStore: error selecting character '{}': {}", name, q.lastError().text());
        return {};
    }

    if (!q.next()) {
        spdlog::error("UserStore: error getting character '{}': {}", name, q.lastError().text());
        return {};
    }

    const QVariant data = q.value("data");
    return readCharacter(data.toByteArray());
}

poe::StashTab UserStore::getStash(const QString &id)
{
    spdlog::debug("UserStore: getting stash: id='{}'", id);

    QSqlQuery q(m_db);
    q.prepare(SELECT_STASH);
    q.bindValue(":id", id);

    if (!q.exec()) {
        spdlog::error("UserStore: error selecting stash '{}': {}", id, q.lastError().text());
        return {};
    }

    if (!q.next()) {
        spdlog::error("UserStore: error getting stash '{}': {}", id, q.lastError().text());
        return {};
    }

    const QVariant data = q.value("data");
    return readStash(data.toByteArray());
}
