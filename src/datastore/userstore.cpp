// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2025 Tom Holz

#include "datastore/userstore.h"

#include <QSqlDriver>
#include <QSqlError>
#include <QSqlQuery>
#include <QSqlRecord>
#include <QStringLiteral>
#include <QUuid>

#include "util/json_readers.h"
#include "util/json_writers.h"
#include "util/spdlog_qt.h" // IWYU pragma: keep

//================================================================================

static constexpr int SCHEMA_VERSION = 1;

constexpr unsigned int QSQLITE_BUSY_TIMEOUT{5000};

constexpr std::array CONNECTION_PRAGMAS{
    "PRAGMA busy_timeout=5000",
    "PRAGMA temp_store=MEMORY",
    "PRAGMA journal_mode=WAL",
    "PRAGMA synchronous=NORMAL",
    "PRAGMA foreign_keys=OFF",
};

// ---------- CHARACTERS ----------

constexpr const char *CREATE_CHARACTER_TABLE{R"(
CREATE TABLE IF NOT EXISTS characters (
    id              TEXT NOT NULL,
    name            TEXT NOT NULL,
    realm           TEXT NOT NULL,
    league          TEXT,
    listed_at       INTEGER NOT NULL,
    json_fetched_at INTEGER,
    json_data       TEXT,
    PRIMARY KEY (realm, id)
)
)"};

constexpr const char *UPSERT_CHARACTER_ENTRY{R"(
INSERT INTO characters (
    id, name, realm, league, listed_at
) VALUES (
    :id, :name, :realm, :league, :listed_at
)
ON CONFLICT(realm, id) DO UPDATE SET
    name            = excluded.name,
    realm           = excluded.realm,
    league          = excluded.league,
    listed_at       = excluded.listed_at
)"};

constexpr const char *UPDATE_CHARACTER{R"(
UPDATE characters
SET
    name            = :name,
    league          = :league,
    json_fetched_at = :json_fetched_at,
    json_data       = :json_data
WHERE realm = :realm AND id = :id
)"};

// ---------- STASHES ----------

constexpr const char *CREATE_STASH_TABLE{R"(
CREATE TABLE IF NOT EXISTS stashes (
    realm           TEXT NOT NULL,
    league          TEXT NOT NULL,
    id              TEXT NOT NULL,
    parent          TEXT,
    folder          TEXT,
    name            TEXT NOT NULL,
    type            TEXT NOT NULL,
    stash_index     INTEGER,
    meta_public     INTEGER NOT NULL DEFAULT 0 CHECK (meta_public IN (0,1)),
    meta_folder     INTEGER NOT NULL DEFAULT 0 CHECK (meta_folder IN (0,1)),
    meta_colour     TEXT,
    listed_at       INTEGER NOT NULL,
    json_fetched_at INTEGER,
    json_data       TEXT,
    PRIMARY KEY (realm, league, id)
)
)"};

constexpr const char *CREATE_STASH_PARENT_INDEX{R"(
CREATE INDEX IF NOT EXISTS idx_stashes_realm_league_parent
ON stashes(realm, league, parent)
)"};

constexpr const char *CREATE_STASH_FOLDER_INDEX{R"(
CREATE INDEX IF NOT EXISTS idx_stashes_realm_league_folder
ON stashes(realm, league, folder)
)"};

constexpr const char *UPSERT_STASH_ENTRY{R"(
INSERT INTO stashes (
  realm, league, id,
  parent, folder, name, type, stash_index,
  meta_public, meta_folder, meta_colour,
  listed_at
)
VALUES (
  :realm, :league, :id,
  :parent, :folder, :name, :type, :stash_index,
  :meta_public, :meta_folder, :meta_colour,
  :listed_at
)
ON CONFLICT(realm, league, id) DO UPDATE SET
    parent          = excluded.parent,
    folder          = excluded.folder,
    name            = excluded.name,
    type            = excluded.type,
    stash_index     = excluded.stash_index,
    meta_public     = excluded.meta_public,
    meta_folder     = excluded.meta_folder,
    meta_colour     = excluded.meta_colour,
    listed_at       = excluded.listed_at
)"};

constexpr const char *UPDATE_STASH{R"(
UPDATE stashes
SET
    parent          = :parent,
    folder          = :folder,
    name            = :name,
    type            = :type,
    stash_index     = :stash_index,
    meta_public     = :meta_public,
    meta_folder     = :meta_folder,
    meta_colour     = :meta_colour,
    json_fetched_at = :json_fetched_at,
    json_data       = :json_data
WHERE realm = :realm AND league = :league AND id = :id
)"};

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
    const QString connection = "userdata:" + username + ":" + uuid;
    const QString filename = dataDir.absoluteFilePath("userstore-" + username + ".db");

    spdlog::debug("UserStore: creating database connection '{}' to '{}'", connection, filename);
    m_db = QSqlDatabase::addDatabase("QSQLITE", connection);
    m_db.setDatabaseName(filename);
    m_db.setConnectOptions(QString("QSQLITE_BUSY_TIMEOUT=%1").arg(QSQLITE_BUSY_TIMEOUT));

    if (!m_db.open()) {
        const QString msg = m_db.lastError().text();
        spdlog::error("UserStore: error opening database connection '{}' to '{}': {}",
                      connection,
                      filename,
                      msg);
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
    // Grab the connection name.
    const QString connection = m_db.connectionName();

    // Close the database and clear the member variable.
    if (m_db.isValid()) {
        m_db.close();
    }
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
    if (!q.exec("BEGIN IMMEDIATE")) {
        return;
    }

    // Another connection might have migrated while we waited.
    const int v = userVersion();
    if (v >= SCHEMA_VERSION) {
        spdlog::debug("UserStore: migration occured while waiting for the lock");
        m_db.commit();
        return;
    }

    // Assemble an ordered list of setup statements.
    constexpr std::array statements{"DROP TABLE IF EXISTS characters;",
                                    CREATE_CHARACTER_TABLE,
                                    "DROP TABLE IF EXISTS stashes;",
                                    CREATE_STASH_TABLE,
                                    CREATE_STASH_PARENT_INDEX,
                                    CREATE_STASH_FOLDER_INDEX};

    // Run setup statements.
    for (const auto &sql : statements) {
        if (!q.exec(sql)) {
            logError(q);
            attemptRollback();
            return;
        }
    }

    // Update the user_version.
    if (!setUserVersion(SCHEMA_VERSION)) {
        spdlog::error("UserStore: unable to set user_version to {}", SCHEMA_VERSION);
        m_db.rollback();
        return;
    }

    // Commit the transaction.
    if (!m_db.commit()) {
        spdlog::error("UserStore: error committing migration: {}", m_db.lastError().text());
        return;
    }

    spdlog::info("UserStore: migrated from version {} to {}", v, SCHEMA_VERSION);
}

void UserStore::logError(const QSqlQuery &q)
{
    const QString err = q.lastError().text();
    const QString sql = q.lastQuery().simplified();

    const QStringList names{q.boundValueNames()};
    const QVariantList values{q.boundValues()};

    QStringList out;
    if (names.size() != values.size()) {
        spdlog::error("UserStore: cannot log bound values: there are {} names and {} values",
                      names.size(),
                      values.size());
        out = names;
    } else {
        out.reserve(names.size());
        for (auto i = 0; i < names.size(); ++i) {
            out.push_back(QString("%1='%2'").arg(names[i], values[i].toString()));
        }
    }
    const QString bindings = out.join(", ");
    spdlog::error("UserStore: exec() failed with '{}' executing '{}' with {}", err, sql, bindings);
}

void UserStore::attemptRollback()
{
    // Make sure the database connection is open.
    if (!m_db.isOpen()) {
        spdlog::error("UserStore: cannot rollback: the connection '{}' to '{}' is closed",
                      m_db.connectionName(),
                      m_db.databaseName());
        return;
    }

    // Make sure the database driver supports rollback.
    if (!m_db.driver()->hasFeature(QSqlDriver::Transactions)) {
        spdlog::error("UserStore: cannot rollback: the driver '' doesn't suport it",
                      m_db.driverName());
        return;
    }

    // Make sure the rollback succeeded.
    if (!m_db.rollback()) {
        spdlog::error("UserStore: error during rollback: {}", m_db.lastError().text());
    }
}

void UserStore::saveCharacterList(const std::vector<poe::Character> &characters,
                                  const QString &realm)
{
    spdlog::debug("UserStore: saving character list: realm='{}', size={}", realm, characters.size());

    if (characters.empty()) {
        spdlog::debug("UserStore: nothing to do");
        return;
    }

    if (!m_db.transaction()) {
        spdlog::error("UserStore: transaction() failed: {}", m_db.lastError().text());
        return;
    }

    QSqlQuery q(m_db);
    if (!q.prepare(UPSERT_CHARACTER_ENTRY)) {
        spdlog::error("UserStore: prepare() failed: {}", q.lastError().text());
        return;
    }

    const qint64 listedAt = QDateTime::currentMSecsSinceEpoch();

    for (const auto &character : characters) {
        if (character.realm != realm) {
            spdlog::error("UserStore: skipping character '{}' ({}) because realm is '{}'",
                          character.name,
                          character.id,
                          character.realm);
            continue;
        }

        q.bindValue(":id", character.id);
        q.bindValue(":name", character.name);
        q.bindValue(":realm", character.realm);
        q.bindValue(":league", optionalValue(character.league));
        q.bindValue(":listed_at", listedAt);

        if (!q.exec()) {
            logError(q);
            attemptRollback();
            return;
        }
    }

    // Delete characters that don't exist any more.
    q.prepare("DELETE FROM characters WHERE realm = :realm AND listed_at < :listed_at");
    q.bindValue(":realm", realm);
    q.bindValue(":listed_at", listedAt);

    if (!q.exec()) {
        logError(q);
        attemptRollback();
        return;
    }

    if (!m_db.commit()) {
        spdlog::error("UserStore: commit() failed: {}", m_db.lastError().text());
        attemptRollback();
    }
}

void UserStore::saveCharacter(const poe::Character &character)
{
    spdlog::debug("UserStore: saving character: realm='{}', id='{}', name='{}'",
                  character.realm,
                  character.id,
                  character.name);

    QSqlQuery q(m_db);
    if (!q.prepare(UPDATE_CHARACTER)) {
        spdlog::error("UserStore: prepare() failed: {}", q.lastError().text());
        return;
    }

    const qint64 jsonFetchedAt = QDateTime::currentMSecsSinceEpoch();
    const QByteArray json = writeCharacter(character);

    q.bindValue(":id", character.id);
    q.bindValue(":name", character.name);
    q.bindValue(":realm", character.realm);
    q.bindValue(":league", optionalValue(character.league));
    q.bindValue(":json_fetched_at", jsonFetchedAt);
    q.bindValue(":json_data", json);

    if (!q.exec()) {
        logError(q);
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

    if (stashes.empty()) {
        spdlog::debug("UserStore: nothing to do");
        return;
    }

    if (!m_db.transaction()) {
        spdlog::error("UserStore: transaction() failed: {}", m_db.lastError().text());
        return;
    }

    QSqlQuery q(m_db);
    if (!q.prepare(UPSERT_STASH_ENTRY)) {
        spdlog::error("UserStore: prepare() failed: {}", q.lastError().text());
        return;
    }

    const qint64 listedAt = QDateTime::currentMSecsSinceEpoch();

    for (const auto &stash : stashes) {
        q.bindValue(":realm", realm);
        q.bindValue(":league", league);
        q.bindValue(":id", stash.id);
        q.bindValue(":parent", optionalValue(stash.parent));
        q.bindValue(":folder", optionalValue(stash.folder));
        q.bindValue(":name", stash.name);
        q.bindValue(":type", stash.type);
        q.bindValue(":stash_index", optionalValue(stash.index));
        q.bindValue(":meta_public", stash.metadata.public_.value_or(false));
        q.bindValue(":meta_folder", stash.metadata.folder.value_or(false));
        q.bindValue(":meta_colour", optionalValue(stash.metadata.colour));
        q.bindValue(":listed_at", listedAt);

        if (!q.exec()) {
            logError(q);
            attemptRollback();
            return;
        }
    }

    // TBD: when can we delete missing stashes, and how do we avoid accidentally
    // deleting the child stashes of Map, Unique, and Flask tabs?

    if (!m_db.commit()) {
        spdlog::error("UserStore: commit() failed: {}", m_db.lastError().text());
        attemptRollback();
    }
}

void UserStore::saveStash(const poe::StashTab &stash, const QString &realm, const QString &league)
{
    spdlog::debug("UserStore: saving stash: realm='{}', league='{}', id='{}', name='{}'",
                  realm,
                  league,
                  stash.id,
                  stash.name);

    QSqlQuery q(m_db);
    if (!q.prepare(UPDATE_STASH)) {
        spdlog::error("UserStore: prepare() failed: {}", q.lastError().text());
        return;
    }

    const qint64 jsonFetchedAt = QDateTime::currentMSecsSinceEpoch();
    const QByteArray json = writeStash(stash);

    q.bindValue(":id", stash.id);
    q.bindValue(":realm", realm);
    q.bindValue(":league", league);
    q.bindValue(":parent", optionalValue(stash.parent));
    q.bindValue(":folder", optionalValue(stash.folder));
    q.bindValue(":name", stash.name);
    q.bindValue(":type", stash.type);
    q.bindValue(":stash_index", optionalValue(stash.index));
    q.bindValue(":meta_public", stash.metadata.public_.value_or(false));
    q.bindValue(":meta_folder", stash.metadata.folder.value_or(false));
    q.bindValue(":meta_colour", optionalValue(stash.metadata.colour));
    q.bindValue(":json_fetched_at", jsonFetchedAt);
    q.bindValue(":json_data", json);

    if (!q.exec()) {
        logError(q);
    }
}

std::vector<poe::Character> UserStore::getCharacterList(const QString &realm)
{
    spdlog::debug("UserStore: getting character list for realm='{}'", realm);

    QSqlQuery q(m_db);

    if (!q.prepare("SELECT id, name, realm, league"
                   " FROM characters"
                   " WHERE realm = :realm")) {
        logError(q);
        return {};
    }

    q.bindValue(":realm", realm);

    if (!q.exec()) {
        logError(q);
        return {};
    }

    std::vector<poe::Character> characters;

    while (q.next()) {
        poe::Character character;
        character.id = q.value("id").toString();
        character.name = q.value("name").toString();
        character.realm = q.value("realm").toString();
        if (!q.isNull("league")) {
            character.league = q.value("league").toString();
        }
        characters.push_back(character);
    }

    spdlog::debug("UserStore: returning {} characters", characters.size());
    return characters;
}

std::optional<poe::Character> UserStore::getCharacter(const QString &name, const QString &realm)
{
    spdlog::debug("UserStore: getting character: name='{}', realm='{}'", name, realm);

    QSqlQuery q(m_db);

    if (!q.prepare("SELECT json_data"
                   " FROM characters"
                   " WHERE name = :name AND realm = :realm")) {
        logError(q);
        return std::nullopt;
    }

    q.bindValue(":name", name);
    q.bindValue(":realm", realm);

    if (!q.exec()) {
        logError(q);
        return std::nullopt;
    }

    if (q.size() > 1) {
        spdlog::error("UserStore: multiple characters found: name='{}', realm='{}'", name, realm);
        return std::nullopt;
    }

    if (!q.next()) {
        spdlog::error("UserStore: character not found: name='{}', realm='{}'", name, realm);
        return std::nullopt;
    }

    if (q.isNull(0)) {
        spdlog::debug("UserStore: character has not been fetched: name='{}', realm='{}'",
                      name,
                      realm);
        return std::nullopt;
    }

    const auto json = q.value(0).toByteArray();
    return readCharacter(json);
}

std::vector<poe::StashTab> UserStore::getStashList(const QString &realm, const QString &league)
{
    spdlog::debug("UserStore: getting stash list: realm='{}', league='{}'", realm, league);

    QSqlQuery q(m_db);

    if (!q.prepare("SELECT id, name, type, stash_index, meta_colour"
                   " FROM stashes"
                   " WHERE realm = :realm AND league = :league")) {
        logError(q);
        return {};
    }

    q.bindValue(":realm", realm);
    q.bindValue(":league", league);

    if (!q.exec()) {
        logError(q);
        return {};
    }

    std::vector<poe::StashTab> stashes;

    while (q.next()) {
        poe::StashTab stash;
        stash.id = q.value("id").toString();
        stash.name = q.value("name").toString();
        stash.type = q.value("type").toString();
        if (!q.isNull("stash_index")) {
            stash.index = q.value("stash_index").toUInt();
        }
        if (!q.isNull("meta_colour")) {
            stash.metadata.colour = q.value("meta_colour").toString();
        }
        stashes.push_back(stash);
    }

    spdlog::debug("UserStore: returning {} stashes", stashes.size());
    return stashes;
}

std::optional<poe::StashTab> UserStore::getStash(const QString &id,
                                                 const QString &realm,
                                                 const QString &league)
{
    spdlog::debug("UserStore: getting stash: id='{}', realm='{}', league='{}'", id, realm, league);

    QSqlQuery q(m_db);

    if (!q.prepare("SELECT json_data"
                   " FROM stashes"
                   " WHERE realm = :realm AND league = :league AND id = :id")) {
        logError(q);
        return std::nullopt;
    }

    q.bindValue(":id", id);
    q.bindValue(":realm", realm);
    q.bindValue(":league", league);

    if (!q.exec()) {
        logError(q);
        return std::nullopt;
    }

    if (!q.next()) {
        spdlog::error("UserStore: stash not found: id='{}', realm='{}', league='{}'",
                      id,
                      realm,
                      league);
        return std::nullopt;
    }

    if (q.isNull(0)) {
        spdlog::debug("UserStore: stash has not been fetched: id='{}', realm='{}', league='{}'",
                      id,
                      realm,
                      league);
        return std::nullopt;
    }

    const auto json = q.value(0).toByteArray();
    return readStash(json);
}
