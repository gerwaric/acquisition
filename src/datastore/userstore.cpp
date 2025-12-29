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

#include "poe/types/character.h"
#include "poe/types/stashtab.h"
#include "userstore.h"
#include "util/spdlog_qt.h"

static_assert(ACQUISITION_USE_SPDLOG);

//================================================================================
// clang-format off

// ---------- LISTS ----------

const QString UserStore::CREATE_LISTS_TABLE = QStringLiteral(R"(
    CREATE TABLE IF NOT EXISTS lists (
        name            TEXT,
        realm           TEXT,
        league          TEXT,
        timestamp       INTEGER,
        data            TEXT
    )
)").simplified();

const QString UserStore::INSERT_CHARACTER_LIST = QStringLiteral(R"(
    INSERT OR REPLACE INTO lists (
        name,
        realm,
        timestamp,
        data
    ) VALUES (
        "characters",
        :realm,
        :timestamp,
        :data
    )
)").simplified();

const QString UserStore::SELECT_CHARACTER_LIST =
    R"(SELECT FROM lists WHERE (name = "characters") AND (realm = :realm))";

const QString UserStore::INSERT_STASH_LIST = QStringLiteral(R"(
    INSERT OR REPLACE INTO lists (
        name,
        realm,
        league,
        timestamp,
        data
    ) VALUES (
        "stashes",
        :realm,
        :league,
        :timestamp,
        :data
    )
)").simplified();

const QString UserStore::SELECT_STASH_LIST =
    R"(SELECT FROM lists WHERE (name = "stashes") AND (realm = :realm) AND (league = :league))";

// ---------- CHARACTERS ----------

const QString UserStore::CREATE_CHARACTER_TABLE = QStringLiteral(R"(
    CREATE TABLE IF NOT EXISTS characters (
        id              TEXT PRIMARY KEY,
        name            TEXT,
        realm           TEXT,
        class           TEXT,
        league          TEXT,
        level           INTEGER,
        experience      INTEGER,
        ruthless        INTEGER,
        expired         INTEGER,
        deleted         INTEGER,
        current         INTEGER,
        meta_version    TEXT,
        timestamp       INTEGER,
        data            BLOB
    ) WITHOUT ROWID
)").simplified();

const QString UserStore::INSERT_CHARACTER = QStringLiteral(R"(
    INSERT OR REPLACE INTO "characters" (
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
)").simplified();


const QString UserStore::SELECT_CHARACTER =
    R"(SELECT FROM characters WHERE (name = :name))";

// ---------- STASHES ----------

const QString UserStore::CREATE_STASH_TABLE = QStringLiteral(R"(
    CREATE TABLE IF NOT EXISTS stashes (
        realm           TEXT,
        league          TEXT,
        id              TEXT PRIMARY KEY,
        parent          TEXT,
        folder          TEXT,
        name            TEXT,
        type            TEXT,
        indx            INTEGER,
        meta_public     INTEGER,
        meta_folder     INTEGER,
        meta_colour     TEXT,
        children        TEXT,
        timestamp       INTEGER,
        data            BLOB
    ) WITHOUT ROWID
)").simplified();

const QStringList UserStore::CREATE_STASH_INDEXES = {
    R"(CREATE INDEX IF NOT EXISTS "idx_stashes_realm"  ON "stashes"("realm"))",
    R"(CREATE INDEX IF NOT EXISTS "idx_stashes_league" ON "stashes"("league"))",
    R"(CREATE INDEX IF NOT EXISTS "idx_stashes_parent" ON "stashes"("parent"))",
    R"(CREATE INDEX IF NOT EXISTS "idx_stashes_folder" ON "stashes"("folder"))",
    R"(CREATE INDEX IF NOT EXISTS "idx_stashes_type"   ON "stashes"("type"))"
};

const QString UserStore::INSERT_STASH = QStringLiteral(R"(
    INSERT OR REPLACE INTO stashes (
        realm,
        league,
        id,
        parent,
        folder,
        name,
        type,
        indx,
        meta_public,
        meta_folder,
        meta_colour,
        children,
        timestamp,
        data
    ) VALUES (
        :realm,
        :league,
        :id,
        :parent,
        :folder,
        :name,
        :type,
        :index,
        :meta_public,
        :meta_folder,
        :meta_colour,
        :children,
        :timestamp,
        :data
    )
)").simplified();

const QString UserStore::SELECT_STASH =
    R"(SELECT FROM stashes WHERE (id = :id))";

// ---------- STASH CHILDREN ----------

const QString UserStore::CREATE_STASH_CHILDREN_TABLE = QStringLiteral(R"(
    CREATE TABLE IF NOT EXISTS stash_children (
        parent_id       TEXT,
        child_id        TEXT
    )
)").simplified();

const QString UserStore::INSERT_STASH_CHILDREN = QStringLiteral(R"(
    INSERT OR REPLACE INTO stash_children (
        parent_id       TEXT,
        child_id        TEXT
    ) VALUES (
        :parent_id,
        :child_id
    )
)").simplified();

const QString UserStore::SELECT_STASH_CHILDREN =
    R"(SELECT FROM stash_children WHERE (parent_id = :parent_id))";

// clang-format on
//================================================================================

UserStore::UserStore(QDir &dir, const QString &username, QObject *parent)
    : QObject(parent)
{
    dir.mkpath(username);

    const auto id = reinterpret_cast<quintptr>(this);
    m_connection = "userdata:" + QString::number(id);
    m_filename = dir.absoluteFilePath(username + "/userdata.db");

    m_db = QSqlDatabase::addDatabase("QSQLITE", m_connection);
    m_db.setDatabaseName(m_filename);
    if (!m_db.open()) {
        const QString msg = m_db.lastError().text();
        spdlog::error("UserStore: error opening database: '{}': {}", m_filename, msg);
        return;
    };

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
            const QString msg = query.lastError().text();
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
    static const QString kInsertCharacterList = QString(INSERT_CHARACTER_LIST).simplified();

    auto json = glz::write_json(characters);
    if (!json) {
        const auto msg = glz::format_error(json.error(), json.value());
        spdlog::error("UserStore: error serializing character list: {}", msg);
        return;
    }

    QSqlQuery query(m_db);

    query.prepare(kInsertCharacterList);
    query.bindValue(":realm", realm);
    query.bindValue(":timestamp", QDateTime::currentMSecsSinceEpoch());
    query.bindValue(":data", QByteArray::fromStdString(json.value()));

    if (!query.exec()) {
        const QString msg = query.lastError().text();
        spdlog::error("UserStore: error saving character list: {}", msg);
    }
}

void UserStore::saveStashList(const std::vector<poe::StashTab> &stashes,
                              const QString &realm,
                              const QString &league)
{
    static const QString kInsertStashList = QString(INSERT_STASH_LIST).simplified();

    auto json = glz::write_json(stashes);
    if (!json) {
        const auto msg = glz::format_error(json.error(), json.value());
        spdlog::error("UserStore: error serializing stash list: {}", msg);
        return;
    }

    QSqlQuery query(m_db);

    query.prepare(kInsertStashList);
    query.bindValue(":realm", realm);
    query.bindValue(":timestamp", QDateTime::currentMSecsSinceEpoch());
    query.bindValue(":data", QByteArray::fromStdString(json.value()));

    if (!query.exec()) {
        const QString msg = query.lastError().text();
        spdlog::error("UserStore: error saving stash list: {}", msg);
    }
}

std::vector<poe::Character> UserStore::getCharacterList(const QString &realm)
{
    QSqlQuery query(m_db);

    query.prepare(SELECT_CHARACTER_LIST);
    query.bindValue(":realm", realm);

    if (!query.exec()) {
        const QString msg = query.lastError().text();
        spdlog::error("UserStore: error saving character list: {}", msg);
        return {};
    }

    return {};
}

std::vector<poe::StashTab> UserStore::getStashList(const QString &realm, const QString &league)
{
    QSqlQuery query(m_db);

    query.prepare(SELECT_STASH_LIST);
    query.bindValue(":realm", realm);
    query.bindValue(":league", league);

    if (!query.exec()) {
        const QString msg = query.lastError().text();
        spdlog::error("UserStore: error saving stash list: {}", msg);
        return {};
    }

    return {};
}

void UserStore::saveCharacter(const poe::Character &character)
{
    const QString version = character.metadata ? character.metadata->version.value_or("") : "";
    const QString league = character.league.value_or("");
    const qint64 timestamp = QDateTime::currentMSecsSinceEpoch();

    std::string json;
    const auto ec = glz::write_json(character, json);
    if (ec) {
        spdlog::error("UserStore: error writing character json: {}", glz::format_error(ec, json));
    }

    QSqlQuery query(m_db);

    query.prepare(INSERT_CHARACTER);
    query.bindValue(":id", character.id);
    query.bindValue(":name", character.name);
    query.bindValue(":realm", character.realm);
    query.bindValue(":class", character.class_);
    query.bindValue(":league", league);
    query.bindValue(":level", character.level);
    query.bindValue(":experience", character.experience);
    query.bindValue(":ruthless", character.ruthless.value_or(false));
    query.bindValue(":expired", character.expired.value_or(false));
    query.bindValue(":deleted", character.deleted.value_or(false));
    query.bindValue(":current", character.current.value_or(false));
    query.bindValue(":meta_version", version);
    query.bindValue(":timestamp", timestamp);
    query.bindValue(":data", QByteArray(json.data(), json.size()));

    if (!query.exec()) {
        const QString msg = query.lastError().text();
        spdlog::error("UserStore: failed to update character: {}", msg);
    }
}

void UserStore::saveStash(const poe::StashTab &stash, const QString &realm, const QString &league)
{
    QStringList child_ids;
    if (stash.children) {
        const auto children = *stash.children;
        child_ids.reserve(children.size());
        for (const auto &child : children) {
            child_ids.push_back(child.id);
        }
    }
    const QString children = child_ids.join(",");
    const qint64 timestamp = QDateTime::currentMSecsSinceEpoch();

    std::string json;
    const auto ec = glz::write_json(stash, json);
    if (ec) {
        spdlog::error("UserStore: error writing stash json: {}", glz::format_error(ec, json));
    }

    QSqlQuery query(m_db);

    query.prepare(INSERT_STASH);
    query.bindValue(":realm", realm);
    query.bindValue(":league", league);
    query.bindValue(":id", stash.id);
    query.bindValue(":parent", stash.parent.value_or(""));
    query.bindValue(":folder", stash.folder.value_or(""));
    query.bindValue(":name", stash.name);
    query.bindValue(":type", stash.type);
    query.bindValue(":index", stash.index.value_or(-1));
    query.bindValue(":meta_public", stash.metadata.public_.value_or(false));
    query.bindValue(":meta_folder", stash.metadata.folder.value_or(false));
    query.bindValue(":meta_colour", stash.metadata.colour.value_or(""));
    query.bindValue(":children", children);
    query.bindValue(":timestamp", timestamp);
    query.bindValue(":data", QByteArray(json.data(), json.size()));

    if (!query.exec()) {
        const QString msg = query.lastError().text();
        spdlog::error("UserStore: failed to update stash: {}", msg);
    }

    if (stash.children) {
        for (const auto &child : *stash.children) {
            query.prepare(INSERT_STASH_CHILDREN);
            query.bindValue(":parent_id", stash.id);
            query.bindValue(":child_id", child.id);

            if (!query.exec()) {
                const QString msg = query.lastError().text();
                spdlog::error("UserStore: failed to update stash children: {}", msg);
            }
        }
    }
}
