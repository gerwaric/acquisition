/*
    Copyright (C) 2014-2024 Acquisition Contributors

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
            QLOG_ERROR() << "Database error calling exec():" << query << ":" << db.lastError().text();
            return false;
        };
        if (!q.next()) {
            QLOG_ERROR() << "Database error calling next():" << query << ":" << db.lastError().text();
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
            QLOG_ERROR() << "Database error calling exec():" << query << ":" << db.lastError().text();
            return false;
        };
        if (!q.next()) {
            QLOG_ERROR() << "Database error calling next():" << query << ":" << db.lastError().text();
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
            QLOG_ERROR() << "Json error parsing" << type << "from" << query << ":" << error;
            return false;
        };
        return true;
    };

}

//-------------------------------------------------------------------------------------------

LegacyDataStore::Imported::Imported(const QString& filename) {

    if (!QFile::exists(filename)) {
        ok = false;
        QLOG_ERROR() << "BuyoutCollection: file not found:" << filename;
        return;
    };

    QSqlDatabase db = QSqlDatabase::addDatabase("QSQLITE", "LegacyDataStore");
    db.setConnectOptions("QSQLITE_OPEN_READONLY");
    db.setDatabaseName(filename);
    if (!db.open()) {
        ok = false;
        QLOG_ERROR() << "BuyoutCollection: cannot open" << filename << "due to error:" << db.lastError().text();
        return;
    };

    ok &= getString(db, "SELECT value FROM data WHERE (key = 'db_version')", data.db_version);
    ok &= getString(db, "SELECT value FROM data WHERE (key = 'version')", data.version);
    ok &= getStruct(db, "SELECT value FROM data WHERE (key = 'buyouts')", data.buyouts);
    ok &= getStruct(db, "SELECT value FROM data WHERE (key = 'tab_buyouts')", data.tab_buyouts);
    ok &= getStruct(db, "SELECT value FROM tabs WHERE (type = 0)", tabs.stashes);
    ok &= getStruct(db, "SELECT value FROM tabs WHERE (type = 1)", tabs.characters);

    const QString statement = "SELECT loc, value FROM items";
    QSqlQuery query(db);
    query.setForwardOnly(true);
    query.prepare(statement);
    if (!query.exec()) {
        ok = false;
        QLOG_ERROR() << "LegacyDataStore: error executing '" + statement + "':" << query.lastError().text();
    };

    while (query.next()) {
        const QString loc = query.value(0).toString();
        const QByteArray bytes = query.value(1).toByteArray();
        std::vector<LegacyItem> result;
        JS::ParseContext context(bytes);
        if (context.parseTo(result) != JS::Error::NoError) {
            ok = false;
            QLOG_ERROR() << "LegacyDataStore: error parsing 'items':" << QString::fromUtf8(context.makeErrorString());
            break;
        };
        items[loc] = result;
    };

    if (query.lastError().isValid()) {
        ok = false;
        QLOG_ERROR() << "LegacyDataStore: error moving to next record in 'items':" << query.lastError().text();
    };

    query.finish();
    db.close();
    db.removeDatabase("LegacyDataStore");
}


LegacyDataStore::LegacyDataStore(const QString& filename)
    : m_datastore(filename)
{}

LegacyDataStore::ValidationStatus LegacyDataStore::validate() {
    validateTabBuyouts();
    validateItemBuyouts();
    return LegacyDataStore::ValidationStatus::Valid;
}

void LegacyDataStore::exportTo(const QString& filename, ExportFormat format)
{
    switch (format) {
    case ExportFormat::JSON: exportJson(filename); break;
    case ExportFormat::TGZ: exportTgz(filename); break;
    default:
        QLOG_ERROR() << "Unhandled export format:" << static_cast<int>(format);
        break;
    };
}

void LegacyDataStore::validateTabBuyouts() {

    const auto& stashes = m_datastore.tabs.stashes;
    const auto& characters = m_datastore.tabs.characters;
    const auto& buyouts = m_datastore.data.tab_buyouts;

    QLocale locale = QLocale::system();
    QLOG_INFO() << "Validating tab buyouts:";
    QLOG_INFO() << "Found" << locale.toString(stashes.size()) << "stash tabs";
    QLOG_INFO() << "Found" << locale.toString(characters.size()) << "characters";
    QLOG_INFO() << "Found" << locale.toString(buyouts.size()) << "tab buyouts";

    using Location = QString;

    std::set<Location> all_locations;
    std::set<Location> duplicated_locations;
    std::set<Location> duplicated_buyouts;
    std::set<Location> ambiguous_buyouts;
    std::set<Location> matched_buyouts;
    std::set<Location> orphaned_buyouts;

    // Add stash tab location tags.
    for (const auto& location : stashes) {
        const Location tag = "stash:" + location.name;
        if (all_locations.count(tag) <= 0) {
            all_locations.insert(tag);
        } else {
            duplicated_locations.insert(tag);
        };
    };

    // Add character location tags.
    for (const auto& location : characters) {
        const Location tag = "character:" + location.name;
        if (all_locations.count(tag) <= 0) {
            all_locations.insert(tag);
        } else {
            duplicated_locations.insert(tag);
        };
    };

    // Validate all the tab buyouts.
    for (const auto& buyout : buyouts) {
        const Location& tag = buyout.first;
        if (matched_buyouts.count(tag) > 0) {
            duplicated_buyouts.insert(tag);
        } else if (all_locations.count(tag) > 0) {
            matched_buyouts.insert(tag);
        } else {
            orphaned_buyouts.insert(tag);
        };
        // If the location tag is one of the duplicated locations,
        // then we don't know which tab this buyout really belongs to.
        if (duplicated_locations.count(tag) > 0) {
            ambiguous_buyouts.insert(tag);
        };
    };

    if (duplicated_buyouts.size() > 0) {
        QLOG_WARN() << "Found" << locale.toString(duplicated_buyouts.size()) << "duplicated tab buyouts";
    };
    if (ambiguous_buyouts.size() > 0) {
        QLOG_WARN() << "Found" << locale.toString(ambiguous_buyouts.size()) << "ambiguous tab buyouts";
    };
    if (orphaned_buyouts.size() > 0) {
        QLOG_WARN() << "Found" << locale.toString(orphaned_buyouts.size()) << "orphaned buyouts";
    };
}

void LegacyDataStore::validateItemBuyouts() {

    const auto& collections = m_datastore.items;
    const auto& buyouts = m_datastore.data.buyouts;

    QLocale locale = QLocale::system();
    QLOG_INFO() << "Validating item buyouts";
    QLOG_INFO() << "Found" << locale.toString(buyouts.size()) << "item buyouts";

    using BuyoutHash = QString;

    std::set<BuyoutHash> unique_buyouts;
    std::set<BuyoutHash> duplicated_buyouts;
    std::set<BuyoutHash> matched_buyouts;
    std::set<BuyoutHash> orphaned_buyouts;

    for (const auto& pair : buyouts) {
        const BuyoutHash& hash = pair.first;
        if (unique_buyouts.count(hash) <= 0) {
            unique_buyouts.insert(hash);
        } else {
            duplicated_buyouts.insert(hash);
        };
    };

    size_t item_count = 0;
    for (const auto& collection : collections) {
        const QString& loc = collection.first;
        const std::vector<LegacyItem>& items = collection.second;
        for (const auto& item : items) {
            const BuyoutHash hash = item.hash();
            if (matched_buyouts.count(hash) > 0) {
                duplicated_buyouts.insert(hash);
            } else if (buyouts.count(hash) > 0) {
                matched_buyouts.insert(hash);
            };
        };
        item_count += items.size();
    };
    QLOG_INFO() << "Found" << locale.toString(item_count) << "items";

    // Now go back and make sure all of the buyouts have beem matched.
    for (const BuyoutHash& hash : unique_buyouts) {
        if (matched_buyouts.count(hash) <= 0) {
            orphaned_buyouts.insert(hash);
        };
    };

    if (duplicated_buyouts.size() > 0) {
        QLOG_WARN() << "Found" << locale.toString(duplicated_buyouts.size()) << "duplicated item buyouts";
    };
    if (orphaned_buyouts.size() > 0) {
        QLOG_WARN() << "Found" << locale.toString(orphaned_buyouts.size()) << "orphaned item buyouts";
    };
}


void LegacyDataStore::exportJson(const QString& filename) {
    QFile file(filename);
    if (!file.open(QIODevice::WriteOnly | QIODevice::Text)) {
        QLOG_ERROR() << "Export failed: could not open json file:" << file.errorString();
        return;
    };
    const QByteArray data(JS::serializeStruct(m_datastore, JS::SerializerOptions::Compact));
    file.write(data);
    file.close();
}

void LegacyDataStore::exportTgz(const QString& filename) {

    // Use a temporary working directory.
    QTemporaryDir dir;
    if (!dir.isValid()) {
        QLOG_ERROR() << "Export failed: could not create a temporary directory:" << dir.errorString();
        return;
    };

    const QString basename = "export_data.json";
    const QString tempfile = dir.filePath(basename);

    // First export to a temporary .json file.
    exportJson(tempfile);

    const QString command = "tar";
    const QStringList arguments = { "czvf", filename, "-C", dir.path(), basename };

    // Next compress the temporary file into a tgz.
    QProcess process;
    process.start(command, arguments);
    if (!process.waitForFinished()) {
        QLOG_ERROR() << "Export failed: process failed:" << process.errorString();
    };
    if (process.exitCode() != 0) {
        QLOG_ERROR() << "Export failed: tar error:" << process.errorString();
    };

    // Remove the temporary .json file.
    QFile(tempfile).remove();
}
