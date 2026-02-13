// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#include "datastore/datarepo.h"

#include <QSqlDatabase>
#include <QSqlError>
#include <QSqlQuery>

#include "datastore/datastore_utils.h"
#include "util/json_utils.h"

constexpr const char *CREATE_DATA_TABLE{R"(
CREATE TABLE IF NOT EXISTS data (
    name        TEXT NOT NULL,
    scope       TEXT NOT NULL,
    updated_at  INT NOT NULL,
    value       TEXT,
    PRIMARY KEY(name, scope)
);
)"};

constexpr const char *CONTAINS_SCOPED_VALUE{R"(
SELECT EXISTS(
    SELECT 1 FROM data WHERE name = :name AND scope = :scope
);
)"};

constexpr const char *DELETE_SCOPED_VALUE{R"(
DELETE FROM data WHERE name = :name AND scope = :scope
)"};

constexpr const char *INSERT_SCOPED_VALUE{R"(
INSERT INTO data (name, scope, updated_at, value)
VALUES (:name, :scope, :updated_at, :value)
ON CONFLICT (name,scope) DO UPDATE SET
    updated_at  = excluded.updated_at,
    value       = excluded.value;
)"};

constexpr const char *SELECT_SCOPED_VALUE{R"(
SELECT value FROM data WHERE name = :name AND scope = :scope
)"};

QString DataRepo::Key::toString() const
{
    return write_json(*this);
}

bool DataRepo::Key::isValid() const
{
    return !name.isEmpty() && !scope.isEmpty();
}

DataRepo::DataRepo(QSqlDatabase &db, QObject *parent)
    : QObject(parent)
    , m_db(db)
{}

bool DataRepo::resetRepo()
{
    QSqlQuery q(m_db);

    if (!q.exec("DROP TABLE IF EXISTS data;")) {
        ds::logQueryError("DataRepo::resetRepo:exec", q);
        return false;
    }
    return ensureSchema();
}

bool DataRepo::ensureSchema()
{
    QSqlQuery q(m_db);

    if (!q.exec(CREATE_DATA_TABLE)) {
        ds::logQueryError("DataRepo::ensureSchema:exec", q);
        return false;
    }
    return true;
}

bool DataRepo::contains(const DataRepo::Key &key)
{
    if (!key.isValid()) {
        spdlog::error("DataRepo::contains: invalid key: {}", key.toString());
        return false;
    }

    QSqlQuery q(m_db);
    if (!q.prepare(CONTAINS_SCOPED_VALUE)) {
        ds::logQueryError("DataRepo::contains:prepare", q);
        return false;
    }
    q.bindValue(":name", key.name);
    q.bindValue(":scope", key.scope);

    if (!q.exec()) {
        ds::logQueryError("DataRepo::contains:exec", q);
        return false;
    }
    if (!q.next()) {
        ds::logQueryError("DataRepo::contains:next", q);
        return false;
    }
    return q.value(0).toBool();
}

bool DataRepo::remove(const DataRepo::Key &key)
{
    if (!key.isValid()) {
        spdlog::error("DataRepo::remove: invalid key: {}", key.toString());
        return false;
    }

    QSqlQuery q(m_db);
    if (!q.prepare(DELETE_SCOPED_VALUE)) {
        ds::logQueryError("DataRepo::contains", q);
        return false;
    }
    q.bindValue(":name", key.name);
    q.bindValue(":scope", key.scope);

    if (!q.exec()) {
        ds::logQueryError("DataRepo::contains:exec", q);
        return false;
    }
    return true;
}

bool DataRepo::setByteArray(const DataRepo::Key &key, const QByteArray &value)
{
    return setValue(key, QVariant::fromValue(value));
}

bool DataRepo::setString(const DataRepo::Key &key, const QString &value)
{
    return setValue(key, QVariant::fromValue(value));
}

bool DataRepo::setBool(const DataRepo::Key &key, bool value)
{
    return setValue(key, QVariant::fromValue(value));
}

bool DataRepo::setInt(const DataRepo::Key &key, int value)
{
    return setValue(key, QVariant::fromValue(value));
}

bool DataRepo::setUInt(const DataRepo::Key &key, uint value)
{
    return setValue(key, QVariant::fromValue(value));
}

bool DataRepo::setDouble(const DataRepo::Key &key, double value)
{
    return setValue(key, QVariant::fromValue(value));
}

bool DataRepo::setDateTime(const DataRepo::Key &key, const QDateTime &value)
{
    return setValue(key, QVariant::fromValue(value));
}

bool DataRepo::setValue(const DataRepo::Key &key, const QVariant &value)
{
    if (!key.isValid()) {
        spdlog::error("DataRepo::setValue: invalid key: {}", key.toString());
        return false;
    }

    QSqlQuery q(m_db);
    if (!q.prepare(INSERT_SCOPED_VALUE)) {
        ds::logQueryError("DataRepo::setValue", q);
        return false;
    }
    q.bindValue(":name", key.name);
    q.bindValue(":scope", key.scope);
    q.bindValue(":updated_at", QDateTime::currentDateTime());
    q.bindValue(":value", value);

    if (!q.exec()) {
        ds::logQueryError("DataRepo::setValue:exec", q);
        return false;
    }
    return true;
}

QByteArray DataRepo::getByteArray(const DataRepo::Key &key, const QByteArray &default_value)
{
    QVariant out;
    return getValue(key, out) ? out.toByteArray() : default_value;
}

QString DataRepo::getString(const DataRepo::Key &key, const QString &default_value)
{
    QVariant out;
    return getValue(key, out) ? out.toString() : default_value;
}

bool DataRepo::getBool(const DataRepo::Key &key, bool default_value)
{
    QVariant out;
    return getValue(key, out) ? out.toBool() : default_value;
}

int DataRepo::getInt(const DataRepo::Key &key, int default_value)
{
    QVariant out;
    return getValue(key, out) ? out.toInt() : default_value;
}

uint DataRepo::getUInt(const DataRepo::Key &key, uint default_value)
{
    QVariant out;
    return getValue(key, out) ? out.toUInt() : default_value;
}

QDateTime DataRepo::getDateTime(const DataRepo::Key &key, const QDateTime default_value)
{
    QVariant out;
    return getValue(key, out) ? out.toDateTime() : default_value;
}

bool DataRepo::getValue(const DataRepo::Key &key, QVariant &out)
{
    if (!key.isValid()) {
        spdlog::error("DataRepo::getValue: invalid key: {}", key.toString());
        return false;
    }

    QSqlQuery q(m_db);
    if (!q.prepare(SELECT_SCOPED_VALUE)) {
        ds::logQueryError("DataRepo::getValue:prepare", q);
        return false;
    }
    q.bindValue(":name", key.name);
    q.bindValue(":scope", key.scope);

    if (!q.exec()) {
        ds::logQueryError("DataRepo::getValue:exec", q);
        return false;
    }
    if (!q.next()) {
        return false;
    }

    const QVariant &value = q.value(0);
    if (value.isNull()) {
        return false;
    }
    if (!value.isValid()) {
        spdlog::warn("DataRepo: invalid value for name='{}', scope='{}': '{}'",
                     key.name,
                     key.scope,
                     value);
        return false;
    }
    out = value;
    return true;
}
