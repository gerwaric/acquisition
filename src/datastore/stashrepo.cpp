// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#include "datastore/stashrepo.h"

#include <QSqlDatabase>
#include <QSqlError>
#include <QSqlQuery>
#include <QString>

#include "datastore/datastore_utils.h"
#include "poe/types/stashtab.h"
#include "util/json_readers.h"
#include "util/json_writers.h"
#include "util/spdlog_qt.h" // IWYU pragma: keep

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
    listed_at       TEXT,
    json_fetched_at TEXT,
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

constexpr const char *UPSERT_STASH{R"(
INSERT INTO stashes (
    realm, league, id,
    parent, folder, name, type, stash_index,
    meta_public, meta_folder, meta_colour,
    json_fetched_at, json_data
)
VALUES (
    :realm, :league, :id,
    :parent, :folder, :name, :type, :stash_index,
    :meta_public, :meta_folder, :meta_colour,
    :json_fetched_at, :json_data
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
    json_fetched_at = excluded.json_fetched_at,
    json_data       = excluded.json_data
)"};

bool StashRepo::reset()
{
    QSqlQuery q(m_db);

    if (!q.exec("DROP TABLE IF EXISTS stashes")) {
        ds::logQueryError("StashRepo::reset", q);
        return false;
    }
    return ensureSchema();
}

bool StashRepo::ensureSchema()
{
    constexpr std::array statements{
        CREATE_STASH_TABLE,
        CREATE_STASH_PARENT_INDEX,
        CREATE_STASH_FOLDER_INDEX,
    };

    QSqlQuery q(m_db);

    // Run setup statements.
    for (const auto &sql : statements) {
        if (!q.exec(sql)) {
            ds::logQueryError("StashRepo::ensureSchema", q);
            return false;
        }
    }

    return true;
}

bool StashRepo::saveStash(const poe::StashTab &stash, const QString &realm, const QString &league)
{
    spdlog::debug("StashRepo: saving stash: realm='{}', league='{}', id='{}', name='{}'",
                  realm,
                  league,
                  stash.id,
                  stash.name);

    QSqlQuery q(m_db);
    if (!q.prepare(UPSERT_STASH)) {
        spdlog::error("StashRepo: prepare() failed: {}", q.lastError().text());
        return false;
    }

    const QDateTime json_fetched_at = ds::timestamp();
    const QByteArray json = json::writeStash(stash);

    q.bindValue(":id", stash.id);
    q.bindValue(":realm", realm);
    q.bindValue(":league", league);
    q.bindValue(":parent", ds::optionalAsNull(stash.parent));
    q.bindValue(":folder", ds::optionalAsNull(stash.folder));
    q.bindValue(":name", stash.name);
    q.bindValue(":type", stash.type);
    q.bindValue(":stash_index", ds::optionalAsNull(stash.index));
    q.bindValue(":meta_public", stash.metadata.public_.value_or(false));
    q.bindValue(":meta_folder", stash.metadata.folder.value_or(false));
    q.bindValue(":meta_colour", ds::optionalAsNull(stash.metadata.colour));
    q.bindValue(":json_fetched_at", json_fetched_at);
    q.bindValue(":json_data", json);

    if (!q.exec()) {
        ds::logQueryError("StashRepo::saveStash()", q);
        return false;
    }
    return true;
}

bool StashRepo::saveStashList(const std::vector<poe::StashTab> &stashes,
                              const QString &realm,
                              const QString &league)
{
    spdlog::debug("StashRepo: saving stash list: realm='{}', league='{}', size={}",
                  realm,
                  league,
                  stashes.size());

    if (stashes.empty()) {
        spdlog::debug("StashRepo: nothing to do");
        return true;
    }

    const QDateTime listed_at = ds::timestamp();

    std::vector<QVariantList> data{12, QVariantList()};
    for (auto list : data) {
        list.reserve(stashes.size());
    }

    for (const auto &stash : stashes) {
        data[0].push_back(realm);
        data[1].push_back(league);
        data[2].push_back(stash.id);
        data[3].push_back(ds::optionalAsNull(stash.parent));
        data[4].push_back(ds::optionalAsNull(stash.folder));
        data[5].push_back(stash.name);
        data[6].push_back(stash.type);
        data[7].push_back(ds::optionalAsNull(stash.index));
        data[8].push_back(stash.metadata.public_.value_or(false));
        data[9].push_back(stash.metadata.folder.value_or(false));
        data[10].push_back(ds::optionalAsNull(stash.metadata.colour));
        data[11].push_back(listed_at);
    }

    QSqlQuery q(m_db);
    if (!q.prepare(UPSERT_STASH_ENTRY)) {
        spdlog::error("StashRepo: prepare() failed: {}", q.lastError().text());
        return false;
    }

    q.bindValue(":realm", data[0]);
    q.bindValue(":league", data[1]);
    q.bindValue(":id", data[2]);
    q.bindValue(":parent", data[3]);
    q.bindValue(":folder", data[4]);
    q.bindValue(":name", data[5]);
    q.bindValue(":type", data[6]);
    q.bindValue(":stash_index", data[7]);
    q.bindValue(":meta_public", data[8]);
    q.bindValue(":meta_folder", data[9]);
    q.bindValue(":meta_colour", data[10]);
    q.bindValue(":listed_at", data[11]);

    if (!q.execBatch(QSqlQuery::ValuesAsRows)) {
        ds::logQueryError("StashRepo::saveStashList()", q);
        return false;
    }
    return true;
}

std::optional<poe::StashTab> StashRepo::getStash(const QString &id,
                                                 const QString &realm,
                                                 const QString &league)
{
    spdlog::debug("StashRepo: getting stash: id='{}', realm='{}', league='{}'", id, realm, league);

    QSqlQuery q(m_db);

    if (!q.prepare("SELECT json_data"
                   " FROM stashes"
                   " WHERE realm = :realm AND league = :league AND id = :id")) {
        ds::logQueryError("StashRepo::getStash()", q);
        return std::nullopt;
    }

    q.bindValue(":id", id);
    q.bindValue(":realm", realm);
    q.bindValue(":league", league);

    if (!q.exec()) {
        ds::logQueryError("StashRepo::getStash()", q);
        return std::nullopt;
    }

    if (!q.next()) {
        spdlog::error("StashRepo: stash not found: id='{}', realm='{}', league='{}'",
                      id,
                      realm,
                      league);
        return std::nullopt;
    }

    if (q.isNull(0)) {
        spdlog::debug("StashRepo: stash has not been fetched: id='{}', realm='{}', league='{}'",
                      id,
                      realm,
                      league);
        return std::nullopt;
    }

    const auto json = q.value(0).toByteArray();
    return json::readStash(json);
}

std::vector<poe::StashTab> StashRepo::getStashList(const QString &realm,
                                                   const QString &league,
                                                   const std::optional<QString> type)
{
    if (type) {
        spdlog::debug("StashRepo: getting stash list: realm='{}', league='{}', type='{}'",
                      realm,
                      league,
                      *type);
    } else {
        spdlog::debug("StashRepo: getting stash list: realm='{}', league='{}'", realm, league);
    }

    QString sql{"SELECT realm, league, id, parent, folder, name, type, stash_index, meta_public, "
                "meta_folder, meta_colour"
                " FROM stashes"
                " WHERE realm = :realm AND league = :league"};

    if (type) {
        sql += " AND type = :type";
    }

    QSqlQuery q(m_db);

    if (!q.prepare(sql)) {
        ds::logQueryError("StashRepo::getStashList()", q);
        return {};
    }

    q.bindValue(":realm", realm);
    q.bindValue(":league", league);
    if (type) {
        q.bindValue(":type", *type);
    }

    if (!q.exec()) {
        ds::logQueryError("StashRepo::getStashList()", q);
        return {};
    }

    std::vector<poe::StashTab> stashes;

    while (q.next()) {
        poe::StashTab stash;
        stash.id = q.value("id").toString();
        if (!q.isNull("parent")) {
            stash.parent = q.value("parent").toString();
        }
        if (!q.isNull("folder")) {
            stash.folder = q.value("folder").toString();
        }
        stash.name = q.value("name").toString();
        stash.type = q.value("type").toString();
        if (!q.isNull("stash_index")) {
            stash.index = q.value("stash_index").toUInt();
        }
        stash.metadata.public_ = q.value("meta_public").toBool();
        stash.metadata.folder = q.value("meta_folder").toBool();
        if (!q.isNull("meta_colour")) {
            stash.metadata.colour = q.value("meta_colour").toString();
        }
        stashes.push_back(stash);
    }

    spdlog::debug("StashRepo: returning {} stashes", stashes.size());
    return stashes;
}

std::vector<poe::StashTab> StashRepo::getStashChildren(const QString &id,
                                                       const QString &realm,
                                                       const QString &league)
{
    spdlog::debug("StashRepo: getting stash children: realm='{}', league='{}', id='{}'",
                  realm,
                  league,
                  id);

    QString sql{"SELECT json_data"
                " FROM stashes"
                " WHERE realm = :realm AND league = :league AND parent = :parent"};

    QSqlQuery q(m_db);

    if (!q.prepare(sql)) {
        ds::logQueryError("StashRepo::getStashChildren()", q);
        return {};
    }

    q.bindValue(":realm", realm);
    q.bindValue(":league", league);
    q.bindValue(":parent", id);

    if (!q.exec()) {
        ds::logQueryError("StashRepo::getStashChildren()", q);
        return {};
    }

    std::vector<poe::StashTab> stashes;

    while (q.next()) {
        const QByteArray json = q.value(0).toByteArray();
        const auto result = json::readStash(json);
        if (result) {
            stashes.push_back(*result);
        }
    }

    spdlog::debug("getStashChildren: returning {} stashes", stashes.size());
    return stashes;
}
