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

#include "legacydatastore.h"

#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QProcess>
#include <QSqlError>
#include <QSqlQuery>
#include <QTemporaryDir>

namespace {

    static bool getByteArray(QSqlDatabase db, const QString& query, QByteArray& value) {
        QSqlQuery q(db);
        q.setForwardOnly(true);
        q.prepare(query);
        if (!q.exec()) {
            spdlog::error("Database error calling exec(): {}: {}", query, db.lastError().text());
            return false;
        };
        if (!q.next()) {
            spdlog::error("Database error calling next(): {}: {}", query, db.lastError().text());
            return false;
        };
        value = q.value(0).toByteArray();
        return true;
    }

    static bool getString(QSqlDatabase db, const QString& query, QString& value) {
        QSqlQuery q(db);
        q.setForwardOnly(true);
        q.prepare(query);
        if (!q.exec()) {
            spdlog::error("Database error calling exec(): {}: {}", query, db.lastError().text());
            return false;
        };
        if (!q.next()) {
            spdlog::error("Database error calling next(): {}: {}", query, db.lastError().text());
            return false;
        };
        value = q.value(0).toString();
        return true;
    }

    template<typename T>
    static bool getStruct(QSqlDatabase db, const QString& query, T& value) {
        QByteArray data;
        if (!getByteArray(db, query, data)) {
            return false;
        };
        JS::ParseContext context(data);
        context.allow_missing_members = false;
        context.allow_unasigned_required_members = false;
        if (context.parseTo(value) != JS::Error::NoError) {
            const auto type = typeid(T).name();
            const auto error = context.makeErrorString();
            spdlog::error("Json error parsing {} from '{}': {}", type, query, error);
            return false;
        };
        return true;
    };

}

//-------------------------------------------------------------------------------------------

LegacyDataStore::LegacyDataStore(const QString& filename) {

    if (!QFile::exists(filename)) {
        spdlog::error("BuyoutCollection: file not found: {}", filename);
        return;
    };

    QSqlDatabase db = QSqlDatabase::addDatabase("QSQLITE", "LegacyDataStore");
    db.setConnectOptions("QSQLITE_OPEN_READONLY");
    db.setDatabaseName(filename);
    if (!db.open()) {
        spdlog::error("BuyoutCollection: cannot open {} due to error: {}", filename, db.lastError().text());
        return;
    };

    bool ok = true;
    ok &= getString(db, "SELECT value FROM data WHERE (key = 'db_version')", m_data.db_version);
    ok &= getString(db, "SELECT value FROM data WHERE (key = 'version')", m_data.version);
    ok &= getStruct(db, "SELECT value FROM data WHERE (key = 'buyouts')", m_data.buyouts);
    ok &= getStruct(db, "SELECT value FROM data WHERE (key = 'tab_buyouts')", m_data.tab_buyouts);
    ok &= getStruct(db, "SELECT value FROM tabs WHERE (type = 0)", m_tabs.stashes);
    ok &= getStruct(db, "SELECT value FROM tabs WHERE (type = 1)", m_tabs.characters);
    if (!ok) {
        spdlog::error("LegacyDataStore: unable to load all data from {}", filename);
        return;
    };

    const QString statement = "SELECT loc, value FROM items";
    QSqlQuery query(db);
    query.setForwardOnly(true);
    query.prepare(statement);
    if (!query.exec()) {
        spdlog::error("LegacyDataStore: error executing '{}': {}", statement, query.lastError().text());
        return;
    };

    m_item_count = 0;
    while (query.next()) {
        const QString loc = query.value(0).toString();
        const QByteArray bytes = query.value(1).toByteArray();
        std::vector<LegacyItem> result;
        JS::ParseContext context(bytes);
        if (context.parseTo(result) != JS::Error::NoError) {
            ok = false;
            spdlog::error("LegacyDataStore: error parsing 'items': {}", QString::fromUtf8(context.makeErrorString()));
            return;
        };
        m_items[loc] = result;
        m_item_count += static_cast<qint64>(result.size());
    };

    if (query.lastError().isValid()) {
        spdlog::error("LegacyDataStore: error moving to next record in 'items': {}", query.lastError().text());
        return;
    };

    query.finish();
    db.close();
    db.removeDatabase("LegacyDataStore");

    m_valid = true;
}

bool LegacyDataStore::exportJson(const QString& filename) const {
    QFile file(filename);
    if (!file.open(QIODevice::WriteOnly | QIODevice::Text)) {
        spdlog::error("Export failed: could not open json file: {}", file.errorString());
        return false;
    };
    const QByteArray data(JS::serializeStruct(*this, JS::SerializerOptions::Compact));
    file.write(data);
    file.close();
    return true;
}

bool LegacyDataStore::exportTgz(const QString& filename) const {

    // Use a temporary working directory.
    QTemporaryDir dir;
    if (!dir.isValid()) {
        spdlog::error("Export failed: could not create a temporary directory: {}", dir.errorString());
        return false;
    };

    // First export to a temporary .json file.
    const QString tempfile = dir.filePath("export.json");
    if (!exportJson(tempfile)) {
        return false;
    };

    // Next compress the temporary file into a tgz.
    const QString command = "tar";
    const QStringList arguments = { "czvf", filename, "-C", dir.path(), "export.json" };
    QProcess process;
    process.start(command, arguments);
    if (!process.waitForFinished()) {
        spdlog::error("Export failed: process failed: {}", process.errorString());
        return false;
    };
    if (process.exitCode() != 0) {
        spdlog::error("Export failed: tar error: {}", process.errorString());
        return false;
    };

    // Remove the temporary .json file.
    if (!QFile(tempfile).remove()) {
        spdlog::warn("Error removing temporary json file: {}", tempfile);
    };
    return true;
}
