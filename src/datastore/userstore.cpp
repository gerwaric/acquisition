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
#include "util/spdlog_qt.h"

static_assert(ACQUISITION_USE_SPDLOG);

//================================================================================
// clang-format off

// ---------- LISTS ----------

const QString UserStore::CREATE_LISTS_TABLE = QStringLiteral(R"(
    CREATE TABLE IF NOT EXISTS lists (
        name            TEXT NOT NULL,
        realm           TEXT NOT NULL,
        league          TEXT NOT NULL,
        timestamp       INTEGER NOT NULL,
        data            TEXT NOT NULL,
        PRIMARY KEY (name, realm, league)
    ) WITHOUT ROWID;
)").simplified();

const QString UserStore::INSERT_LIST = QStringLiteral(R"(
    INSERT INTO lists ( name,  realm,  league,  timestamp,  data)
    VALUES            (:name, :realm, :league, :timestamp, :data)
    ON CONFLICT(name, realm, league)
    DO UPDATE SET
        timestamp = excluded.timestamp,
        data      = excluded.data;
)").simplified();

const QString UserStore::SELECT_LIST = QStringLiteral(R"(
    SELECT data, timestamp
    FROM lists
    WHERE (name = :name) AND (realm = :realm) AND (league = :league);
)").simplified();

// ---------- CHARACTERS ----------

const QString UserStore::CREATE_CHARACTER_TABLE = QStringLiteral(R"(
    CREATE TABLE IF NOT EXISTS characters (
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
        meta_version    TEXT NOT NULL,
        timestamp       INTEGER NOT NULL,
        data            TEXT NOT NULL
    ) WITHOUT ROWID;
)").simplified();

const QString UserStore::INSERT_CHARACTER = QStringLiteral(R"(
    INSERT INTO characters (
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
      data         = excluded.data;
)").simplified();


const QString UserStore::SELECT_CHARACTER = QStringLiteral(R"(
    SELECT data, timestamp
    FROM characters
    WHERE id = :id;
)").simplified();

// ---------- STASHES ----------

const QString UserStore::CREATE_STASH_TABLE = QStringLiteral(R"(
    CREATE TABLE IF NOT EXISTS stashes (
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
        data            TEXT NOT NULL
    ) WITHOUT ROWID;
)").simplified();

const QStringList UserStore::CREATE_STASH_INDEXES = {
    R"(CREATE INDEX IF NOT EXISTS idx_stashes_realm  ON stashes(realm);)",
    R"(CREATE INDEX IF NOT EXISTS idx_stashes_league ON stashes(league);)",
    R"(CREATE INDEX IF NOT EXISTS idx_stashes_parent ON stashes(parent);)",
    R"(CREATE INDEX IF NOT EXISTS idx_stashes_folder ON stashes(folder);)",
    R"(CREATE INDEX IF NOT EXISTS idx_stashes_type   ON stashes(type);)"
};

const QString UserStore::INSERT_STASH = QStringLiteral(R"(
    INSERT INTO stashes (
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
        data        = excluded.data;
)").simplified();

const QString UserStore::SELECT_STASH = QStringLiteral(R"(
    SELECT data, timestamp
    FROM stashes
    WHERE id = :id;
)").simplified();

// ---------- STASH CHILDREN ----------

const QString UserStore::CREATE_STASH_CHILDREN_TABLE = QStringLiteral(R"(
    CREATE TABLE IF NOT EXISTS stash_children (
        parent_id       TEXT NOT NULL,
        child_id        TEXT NOT NULL,
        PRIMARY KEY (parent_id, child_id)
    ) WITHOUT ROWID;
)").simplified();

const QString UserStore::DELETE_STASH_CHILDREN = QStringLiteral(R"(
    DELETE FROM stash_children
    WHERE parent_id = :parent_id;
)").simplified();

const QString UserStore::INSERT_STASH_CHILDREN = QStringLiteral(R"(
    INSERT OR IGNORE INTO stash_children (
        parent_id,
        child_id
    ) VALUES (
        :parent_id,
        :child_id
    );
)").simplified();

const QString UserStore::SELECT_STASH_CHILDREN = QStringLiteral(R"(
    SELECT child_id
    FROM stash_children
    WHERE parent_id = :parent_id;
)").simplified();

// clang-format on
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

    m_connection = "userdata:" + username + ":" + uuid;
    m_filename = dataDir.absoluteFilePath(username + ".db");
    m_db = QSqlDatabase::addDatabase("QSQLITE", m_connection);
    m_db.setDatabaseName(m_filename);
    if (!m_db.open()) {
        const QString msg = m_db.lastError().text();
        spdlog::error("UserStore: error opening database: '{}': {}", m_filename, msg);
        return;
    }
    setupDatabase();
}

UserStore::~UserStore()
{
    if (m_db.isValid()) {
        m_db.close();
    }
    m_db = QSqlDatabase();
    QSqlDatabase::removeDatabase(m_connection);
}

void UserStore::setupDatabase()
{
    if (!m_db.isOpen()) {
        spdlog::error("UserStore: cannot setup database: database is not open");
        return;
    }

    static const QStringList sql_statements = QStringList({CREATE_LISTS_TABLE,
                                                           CREATE_CHARACTER_TABLE,
                                                           CREATE_STASH_TABLE,
                                                           CREATE_STASH_CHILDREN_TABLE})
                                              + CREATE_STASH_INDEXES;

    QSqlQuery query(m_db);

    if (!m_db.transaction()) {
        spdlog::error("UserStore: failed to begin transaction: {}", m_db.lastError().text());
        return;
    }

    for (const auto &sql : sql_statements) {
        if (!query.exec(sql)) {
            const auto msg = query.lastError().text();
            spdlog::error("UserStore: failed to execute statement: '{}': {}", sql, msg);
            m_db.rollback();
            return;
        }
    }

    if (!m_db.commit()) {
        spdlog::error("UserStore: failed to commit schema setup: {}", m_db.lastError().text());
        m_db.rollback();
    }
}

void UserStore::saveCharacterList(const std::vector<poe::Character> &characters,
                                  const QString &realm)
{
    if (!m_db.isOpen()) {
        spdlog::error("UserStore: cannot save character list: database is not open");
        return;
    }

    const auto json = glz::write_json(characters);
    if (!json) {
        const auto msg = glz::format_error(json.error());
        spdlog::error("UserStore: error serializing character list: {}", msg);
        return;
    }

    QSqlQuery query(m_db);

    query.prepare(INSERT_LIST);
    query.bindValue(":name", "characters");
    query.bindValue(":realm", realm);
    query.bindValue(":league", "*");
    query.bindValue(":timestamp", QDateTime::currentMSecsSinceEpoch());
    query.bindValue(":data", QByteArray::fromStdString(json.value()));
    if (!query.exec()) {
        const auto msg = query.lastError().text();
        spdlog::error("UserStore: error saving character list: {}", msg);
    }
}

void UserStore::saveStashList(const std::vector<poe::StashTab> &stashes,
                              const QString &realm,
                              const QString &league)
{
    if (!m_db.isOpen()) {
        spdlog::error("UserStore: cannot save stash list: database is not open");
        return;
    }

    const auto json = glz::write_json(stashes);
    if (!json) {
        const auto msg = glz::format_error(json.error());
        spdlog::error("UserStore: error serializing stash list: {}", msg);
        return;
    }

    QSqlQuery query(m_db);

    query.prepare(INSERT_LIST);
    query.bindValue(":name", "stashes");
    query.bindValue(":realm", realm);
    query.bindValue(":league", league);
    query.bindValue(":timestamp", QDateTime::currentMSecsSinceEpoch());
    query.bindValue(":data", QByteArray::fromStdString(json.value()));
    if (!query.exec()) {
        const auto msg = query.lastError().text();
        spdlog::error("UserStore: error saving stash list: {}", msg);
        return;
    }
}

std::vector<poe::Character> UserStore::getCharacterList(const QString &realm)
{
    if (!m_db.isOpen()) {
        spdlog::error("UserStore: cannot get character list: database is not open");
        return {};
    }

    QSqlQuery query(m_db);

    query.prepare(SELECT_LIST);
    query.bindValue(":name", "characters");
    query.bindValue(":realm", realm);
    query.bindValue(":league", "*");
    if (!query.exec()) {
        const auto msg = query.lastError().text();
        spdlog::error("UserStore: error saving character list: {}", msg);
        return {};
    }

    return {};
}

std::vector<poe::StashTab> UserStore::getStashList(const QString &realm, const QString &league)
{
    if (!m_db.isOpen()) {
        spdlog::error("UserStore: cannot get stash list: database is not open");
        return {};
    }

    QSqlQuery query(m_db);

    query.prepare(SELECT_LIST);
    query.bindValue(":name", "stashes");
    query.bindValue(":realm", realm);
    query.bindValue(":league", league);
    if (!query.exec()) {
        const auto msg = query.lastError().text();
        spdlog::error("UserStore: error saving stash list: {}", msg);
        return {};
    }

    return {};
}

void UserStore::saveCharacter(const poe::Character &character)
{
    if (!m_db.isOpen()) {
        spdlog::error("UserStore: cannot save character: database is not open");
        return;
    }

    const qint64 timestamp = QDateTime::currentMSecsSinceEpoch();
    const QVariant version = (character.metadata && character.metadata->version)
                                 ? *character.metadata->version
                                 : QVariant();

    const auto json = glz::write_json(character);
    if (!json) {
        const auto msg = glz::format_error(json.error());
        spdlog::error("UserStore: error writing character json: {}", msg);
        return;
    }

    QSqlQuery query(m_db);

    query.prepare(INSERT_CHARACTER);
    query.bindValue(":id", character.id);
    query.bindValue(":name", character.name);
    query.bindValue(":realm", character.realm);
    query.bindValue(":class", character.class_);
    query.bindValue(":league", character.league ? *character.league : QVariant());
    query.bindValue(":level", character.level);
    query.bindValue(":experience", character.experience);
    query.bindValue(":ruthless", character.ruthless.value_or(false));
    query.bindValue(":expired", character.expired.value_or(false));
    query.bindValue(":deleted", character.deleted.value_or(false));
    query.bindValue(":current", character.current.value_or(false));
    query.bindValue(":meta_version", version);
    query.bindValue(":timestamp", timestamp);
    query.bindValue(":data", QByteArray(json->data(), json->size()));
    if (!query.exec()) {
        const auto msg = query.lastError().text();
        spdlog::error("UserStore: failed to update character: {}", msg);
        return;
    }
}

void UserStore::saveStash(const poe::StashTab &stash, const QString &realm, const QString &league)
{
    if (!m_db.isOpen()) {
        spdlog::error("UserStore: cannot save stash: database is not open");
        return;
    }

    const qint64 timestamp = QDateTime::currentMSecsSinceEpoch();

    const auto json = glz::write_json(stash);
    if (!json) {
        const auto msg = glz::format_error(json.error());
        spdlog::error("UserStore: error writing stash json: {}", msg);
        return;
    }

    if (!m_db.transaction()) {
        const auto msg = m_db.lastError().text();
        spdlog::error("UserStore: error starting stash transaction: {}", msg);
        return;
    }

    QSqlQuery query(m_db);

    query.prepare(INSERT_STASH);
    query.bindValue(":id", stash.id);
    query.bindValue(":realm", realm);
    query.bindValue(":league", league);
    query.bindValue(":parent", stash.parent ? *stash.parent : QVariant());
    query.bindValue(":folder", stash.folder ? *stash.folder : QVariant());
    query.bindValue(":name", stash.name);
    query.bindValue(":type", stash.type);
    query.bindValue(":indx", stash.index ? *stash.index : QVariant());
    query.bindValue(":meta_public", stash.metadata.public_.value_or(false));
    query.bindValue(":meta_folder", stash.metadata.folder.value_or(false));
    query.bindValue(":meta_colour", stash.metadata.colour ? *stash.metadata.colour : QVariant());
    query.bindValue(":timestamp", timestamp);
    query.bindValue(":data", QByteArray(json->data(), json->size()));
    if (!query.exec()) {
        const auto msg = query.lastError().text();
        spdlog::error("UserStore: failed to update stash: {}", msg);
        m_db.rollback();
        return;
    }

    query.prepare(DELETE_STASH_CHILDREN);
    query.bindValue(":parent_id", stash.id);
    if (!query.exec()) {
        const auto msg = query.lastError().text();
        spdlog::error("UserStore: failed to delete children: {}", msg);
        m_db.rollback();
        return;
    }

    if (stash.children) {
        QVariantList parents, children;

        parents.reserve(stash.children->size());
        children.reserve(stash.children->size());
        for (const auto &child : *stash.children) {
            parents.push_back(stash.id);
            children.push_back(child.id);
        }

        query.prepare(INSERT_STASH_CHILDREN);
        query.bindValue(":parent_id", parents);
        query.bindValue(":child_id", children);
        if (!query.execBatch()) {
            const auto msg = query.lastError().text();
            spdlog::error("UserStore: failed to update stash children: {}", msg);
            m_db.rollback();
            return;
        }
    }

    if (!m_db.commit()) {
        const auto msg = m_db.lastError().text();
        spdlog::error("UserStore: error committing stash transaction: {}", msg);
        m_db.rollback();
        return;
    }
}
