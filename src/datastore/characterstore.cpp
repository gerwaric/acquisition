// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#include "datastore/characterstore.h"

#include <QSqlDatabase>
#include <QSqlError>
#include <QSqlQuery>
#include <QString>

#include "datastore/datastore_utils.h"
#include "poe/types/character.h"
#include "util/json_readers.h"
#include "util/json_writers.h"
#include "util/spdlog_qt.h" // IWYU pragma: keep

constexpr const char *CREATE_CHARACTER_TABLE{R"(
CREATE TABLE IF NOT EXISTS characters (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    realm           TEXT NOT NULL,
    league          TEXT,
    listed_at       TEXT NOT NULL,
    json_fetched_at TEXT,
    json_data       TEXT
)
)"};

constexpr const char *UPSERT_CHARACTER_ENTRY{R"(
INSERT INTO characters (id, name, realm, league, listed_at)
VALUES (:id, :name, :realm, :league, :listed_at)
ON CONFLICT(id) DO UPDATE SET
    name            = excluded.name,
    realm           = excluded.realm,
    league          = excluded.league,
    listed_at       = excluded.listed_at
)"};

constexpr const char *UPDATE_CHARACTER{R"(
UPDATE characters
SET
    name            = :name,
    realm           = :realm,
    league          = :league,
    json_fetched_at = :json_fetched_at,
    json_data       = :json_data
WHERE id = :id
)"};

CharacterStore::CharacterStore(QStringView connName)
    : m_connName(connName) {};

bool CharacterStore::resetRepo()
{
    QSqlDatabase db = QSqlDatabase::database(m_connName);
    QSqlQuery q(db);

    if (!q.exec("DROP TABLE IF EXISTS characters")) {
        ds::logQueryError("CharacterStore::resetRepo:exec", q);
        return false;
    }
    return ensureSchema();
}

bool CharacterStore::ensureSchema()
{
    QSqlDatabase db = QSqlDatabase::database(m_connName);
    QSqlQuery q(db);

    if (!q.exec(CREATE_CHARACTER_TABLE)) {
        ds::logQueryError("CharacterStore::ensureSchema:exec", q);
        return false;
    }
    return true;
}

bool CharacterStore::saveCharacter(const poe::Character &character)
{
    spdlog::debug("CharacterStore: saving character: name='{}', id='{}', realm='{}', league='{}'",
                  character.name,
                  character.id,
                  character.realm,
                  character.league.value_or(""));

    QSqlDatabase db = QSqlDatabase::database(m_connName);
    QSqlQuery q(db);
    if (!q.prepare(UPDATE_CHARACTER)) {
        ds::logQueryError("CharacterStore::saveCharacter:prepare", q);
        return false;
    }

    const QDateTime json_fetched_at = ds::timestamp();
    const QByteArray json = json::writeCharacter(character);

    q.bindValue(":id", character.id);
    q.bindValue(":name", character.name);
    q.bindValue(":realm", character.realm);
    q.bindValue(":league", ds::optionalAsNull(character.league));
    q.bindValue(":json_fetched_at", json_fetched_at);
    q.bindValue(":json_data", json);

    if (!q.exec()) {
        ds::logQueryError("CharacterStore::saveCharacter:exec", q);
        return false;
    }
    return true;
}

bool CharacterStore::saveCharacterList(const std::vector<poe::Character> &characters)
{
    spdlog::debug("CharacterStore: saving character list: size={}", characters.size());

    if (characters.empty()) {
        spdlog::debug("CharacterStore: nothing to do");
        return true;
    }

    const QDateTime listed_at = ds::timestamp();

    std::vector<QVariantList> data{5, QVariantList()};
    for (auto list : data) {
        list.reserve(characters.size());
    }

    for (const auto &character : characters) {
        data[0].push_back(character.id);
        data[1].push_back(character.name);
        data[2].push_back(character.realm);
        data[3].push_back(ds::optionalAsNull(character.league));
        data[4].push_back(listed_at);
    }

    QSqlDatabase db = QSqlDatabase::database(m_connName);
    QSqlQuery q(db);
    if (!q.prepare(UPSERT_CHARACTER_ENTRY)) {
        ds::logQueryError("CharacterStore::saveCharacterList:prepare", q);
        return false;
    }

    q.bindValue(":id", data[0]);
    q.bindValue(":name", data[1]);
    q.bindValue(":realm", data[2]);
    q.bindValue(":league", data[3]);
    q.bindValue(":listed_at", data[4]);

    if (!q.execBatch(QSqlQuery::ValuesAsRows)) {
        ds::logQueryError("CharacterStore::saveCharacterList:execBatch", q);
        return false;
    }
    return true;
}

std::optional<poe::Character> CharacterStore::getCharacter(const QString &name, const QString &realm)
{
    spdlog::debug("CharacterStore: getting character: name='{}', realm='{}'", name, realm);

    QSqlDatabase db = QSqlDatabase::database(m_connName);
    QSqlQuery q(db);
    if (!q.prepare("SELECT json_data FROM characters WHERE name = :name AND realm = :realm")) {
        ds::logQueryError("CharacterStore::getCharacter:prepare", q);
        return std::nullopt;
    }

    q.bindValue(":name", name);
    q.bindValue(":realm", realm);

    if (!q.exec()) {
        ds::logQueryError("CharacterStore::getCharacter:exec", q);
        return std::nullopt;
    }

    if (q.size() > 1) {
        spdlog::error("CharacterStore: multiple characters found: name='{}', realm='{}'",
                      name,
                      realm);
        return std::nullopt;
    }

    if (!q.next()) {
        spdlog::error("CharacterStore: character not found: name='{}', realm='{}'", name, realm);
        return std::nullopt;
    }

    if (q.isNull(0)) {
        spdlog::debug("CharacterStore: character has not been fetched: name='{}', realm='{}'",
                      name,
                      realm);
        return std::nullopt;
    }

    const auto json = q.value(0).toByteArray();
    return json::readCharacter(json);
}

std::vector<poe::Character> CharacterStore::getCharacterList(const QString &realm,
                                                             const std::optional<QString> league)
{
    if (league) {
        spdlog::debug("CharacterStore: getting character list for realm='{}', league='{}'",
                      realm,
                      *league);
    } else {
        spdlog::debug("CharacterStore: getting character list for realm='{}'", realm);
    }

    QString sql{"SELECT id, name, realm, league FROM characters WHERE realm = :realm"};
    if (league) {
        sql += " AND league = :league";
    }

    QSqlDatabase db = QSqlDatabase::database(m_connName);
    QSqlQuery q(db);
    if (!q.prepare(sql)) {
        ds::logQueryError("CharacterStore::getCharacterList:prepare", q);
        return {};
    }

    q.bindValue(":realm", realm);
    if (league) {
        q.bindValue(":league", *league);
    }

    if (!q.exec()) {
        ds::logQueryError("CharacterStore::getCharacterList:exec", q);
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

    spdlog::debug("CharacterStore: returning {} characters", characters.size());
    return characters;
}
