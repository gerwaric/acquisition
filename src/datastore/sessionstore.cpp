// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#include "datastore/sessionstore.h"

#include <QSqlDatabase>
#include <QSqlError>
#include <QSqlQuery>

#include "app/usersettings.h"
#include "datastore/datastore_utils.h"
#include "util/spdlog_qt.h" // IWYU pragma: keep

using namespace Qt::StringLiterals;

constexpr const char *CREATE_SESSION_DATA_TABLE{R"(
CREATE TABLE IF NOT EXISTS session_data (
    name        TEXT NOT NULL,
    scope       TEXT NOT NULL,
    updated_at  INT NOT NULL,
    value       TEXT,
    PRIMARY KEY(name, scope)
);
)"};

constexpr const char *CONTAINS_SCOPED_VALUE{R"(
SELECT EXISTS(
    SELECT 1 FROM session_data WHERE name = :name AND scope = :scope
);
)"};

constexpr const char *DELETE_SCOPED_VALUE{R"(
DELETE FROM session_data WHERE name = :name AND scope = :scope
)"};

constexpr const char *INSERT_SCOPED_VALUE{R"(
INSERT INTO session_data (name, scope, updated_at, value)
VALUES (:name, :scope, :updated_at, :value)
ON CONFLICT (name,scope) DO UPDATE SET
    updated_at  = excluded.updated_at,
    value       = excluded.value;
)"};

constexpr const char *SELECT_SCOPED_VALUE{R"(
SELECT value FROM data WHERE name = :name AND scope = :scope
)"};

// clang-format off
SessionStore::SessionStore(QStringView connName, app::UserSettings &settings)
    : autoupdate                {*this, "tabs/autoupdate"_L1}
    , autoupdateInterval        {*this, "tabs/autoupdate_interval"_L1}
    , fetchMapStashes           {*this, "tabs/fetch_map_stashes"_L1}
    , fetchUniqueStashes        {*this, "tabs/fetch_unique_stashes"_L1}
    , shopAutoupdate            {*this, "shop/autoupdate"_L1}
    , shopThreads               {*this, "shop/threads"_L1}
    , shopHash                  {*this, "shop/hash"_L1}
    , shopTemplate              {*this, "shop/template"_L1}
    , refreshChecked            {*this, "search/refresh_checked"_L1}
    , m_settings(settings)
    , m_connName(connName)
// clang-format on
{}

bool SessionStore::resetRepo()
{
    QSqlDatabase db = QSqlDatabase::database(m_connName);
    QSqlQuery q(db);

    if (!q.exec("DROP TABLE IF EXISTS session_data;")) {
        ds::logQueryError("SessionStore::resetRepo:exec", q);
        return false;
    }
    return ensureSchema();
}

bool SessionStore::ensureSchema()
{
    QSqlDatabase db = QSqlDatabase::database(m_connName);
    QSqlQuery q(db);

    if (!q.exec(CREATE_SESSION_DATA_TABLE)) {
        ds::logQueryError("SessionStore::ensureSchema:exec", q);
        return false;
    }
    return true;
}

QString SessionStore::sessionScope() const
{
    return QString("%1/%2/%3").arg(m_settings.username(), m_settings.realm(), m_settings.league());
}

QVariant SessionStore::get(QLatin1StringView key) const
{
    const auto scope = sessionScope();

    QSqlDatabase db = QSqlDatabase::database(m_connName);
    QSqlQuery q(db);
    if (!q.prepare(SELECT_SCOPED_VALUE)) {
        ds::logQueryError("SessionStore::get:prepare", q);
        return QVariant();
    }
    q.bindValue(":name", key);
    q.bindValue(":scope", scope);

    if (!q.exec()) {
        ds::logQueryError("SessionStore::get:exec", q);
        return QVariant();
    }
    if (!q.next()) {
        return QVariant();
    }

    return QVariant();

    const QVariant &value = q.value(0);
    if (value.isNull()) {
        return QVariant();
    }
    if (!value.isValid()) {
        spdlog::warn("SessionStore: invalid value for key='{}', scope='{}': '{}'",
                     key,
                     scope,
                     value);
        return false;
    }
    out = value;
    return true;
}

void SessionStore::set(QLatin1StringView key, const QVariant &value)
{
    const auto scope = sessionScope();

    QSqlDatabase db = QSqlDatabase::database(m_connName);
    QSqlQuery q(db);
    if (!q.prepare(INSERT_SCOPED_VALUE)) {
        ds::logQueryError("SessionStore::set:prepare", q);
        return;
    }
    q.bindValue(":name", key);
    q.bindValue(":scope", scope);
    q.bindValue(":updated_at", QDateTime::currentDateTime());
    q.bindValue(":value", value);

    if (!q.exec()) {
        ds::logQueryError("SessionStore::set:exec", q);
    }
}

void SessionStore::clear(QLatin1StringView key)
{
    const auto scope = sessionScope();

    QSqlDatabase db = QSqlDatabase::database(m_connName);
    QSqlQuery q(db);
    if (!q.prepare(DELETE_SCOPED_VALUE)) {
        ds::logQueryError("SessionStore::clear:prepare", q);
        return;
    }
    q.bindValue(":name", key);
    q.bindValue(":scope", scope);

    if (!q.exec()) {
        ds::logQueryError("SessionStore::clear:exec", q);
    }
}
