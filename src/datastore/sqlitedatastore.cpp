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

#include "sqlitedatastore.h"

#include <QCryptographicHash>
#include <QDir>
#include <QFileInfo>
#include <QSqlError>
#include <QSqlQuery>
#include <QThread>

#include <util/spdlog_qt.h>

#include <currencymanager.h>

SqliteDataStore::SqliteDataStore(const QString& filename)
    : m_filename(filename)
{
    QDir dir(QDir::cleanPath(m_filename + "/.."));
    if (!dir.exists()) {
        QDir().mkpath(dir.path());
    };

    if (!QFile(filename).exists()) {
        // If the file doesn't exist, it's possible there's an old data file from
        // before the addition of account name discriminators. Look for one of those
        // files and rename if it found.
        const QString old_filename = m_filename.chopped(5);
        if (QFile(old_filename).exists()) {
            spdlog::warn("Renaming old data file with new account discriminator: {}", m_filename);
            if (!QFile(old_filename).rename(m_filename)) {
                spdlog::error("Unable to rename file: {}", old_filename);
            };
        };
    };

    // Open the database and make sure tables are created if they don't exist.
    QSqlDatabase db = getThreadLocalDatabase();
    CreateTable("data", "key TEXT PRIMARY KEY, value BLOB");
    CreateTable("tabs", "type INT PRIMARY KEY, value BLOB");
    CreateTable("items", "loc TEXT PRIMARY KEY, value BLOB");
    CreateTable("currency", "timestamp INTEGER PRIMARY KEY, value TEXT");
    CleanItemsTable();

    QSqlQuery query(db);
    query.prepare("VACUUM");
    if (query.exec() == false) {
        spdlog::error("SqliteDataStore: failed to vacuum QSQLITE database: {}: {}", filename, db.lastError().text());
    };
}


SqliteDataStore::~SqliteDataStore() {
    QMutexLocker locker(&m_mutex);
    const auto& connections = m_connection_names;
    for (const QString& connection : connections) {
        if (QSqlDatabase::contains(connection)) {
            // Close and remove each database connection.
            QSqlDatabase db = QSqlDatabase::database(connection);
            db.close();
            QSqlDatabase::removeDatabase(connection);
        };
    };
    m_connection_names.clear();
}

QString SqliteDataStore::getThreadLocalConnectionName() const {
    // Create a database connection name for this thread.
    const QString file_name = QFileInfo(m_filename).fileName();
    const quintptr thread_id = reinterpret_cast<quintptr>(QThread::currentThread());
    return QString("sqlite-%1-%2").arg(file_name).arg(thread_id);
}

QSqlDatabase SqliteDataStore::getThreadLocalDatabase() {
    QString connection = getThreadLocalConnectionName();
    if (!QSqlDatabase::contains(connection)) {
        QSqlDatabase db = QSqlDatabase::addDatabase("QSQLITE", connection);
        db.setDatabaseName(m_filename);
        if (!db.open()) {
            spdlog::error("Failed to open database {}: {}", m_filename, db.lastError().text());
        } else {
            // Add the connection to the list so we can close it later.
            QMutexLocker locker(&m_mutex);
            m_connection_names.insert(connection);
        }
    };
    return QSqlDatabase::database(connection);
}

void SqliteDataStore::CreateTable(const QString& name, const QString& fields) {
    QSqlDatabase db = getThreadLocalDatabase();
    QSqlQuery query(db);
    query.prepare("CREATE TABLE IF NOT EXISTS " + name + "(" + fields + ")");
    if (query.exec() == false) {
        spdlog::error("CreateTable(): failed to create {}: {}", name, query.lastError().text());
    };
}

void SqliteDataStore::CleanItemsTable() {
    QSqlDatabase db = getThreadLocalDatabase();
    QSqlQuery query(db);
    query.prepare("DELETE FROM items WHERE loc IS NULL");
    if (query.exec() == false) {
        spdlog::error("CleanItemsTable(): error deleting items where loc is null.");
        return;
    };

    //If tabs table contains two records which are not empty or NULL (i.e. type column is equal to 0 or 1 for the two records)
    //  * check all "db.items" record keys against 'id' or 'name' values in the "db.tabs" data,
    //    remove record from 'items' if not anywhere in either 'tabs' record.
    Locations stashTabData = SqliteDataStore::GetTabs(ItemLocationType::STASH);
    Locations charsData = SqliteDataStore::GetTabs(ItemLocationType::CHARACTER);

    if (!stashTabData.empty() && !charsData.empty()) {
        QStringList locs;

        query = QSqlQuery(db);
        query.setForwardOnly(true);
        query.prepare("SELECT loc FROM items");
        if (query.exec() == false) {
            spdlog::error("CleanItemsTable(): error selecting loc from items.");
            return;
        };
        while (query.next()) {
            locs.push_back(query.value(0).toString());
        };
        if (query.lastError().isValid()) {
            spdlog::error("CleanItemsTable(): error moving to next loc: {}", query.lastError().text());
        };
        query.finish();

        for (const auto& loc : locs) {

            rapidjson::Document doc;
            bool foundLoc = false;

            //check stash tabs
            for (const auto& stashTab : stashTabData) {
                if (loc == stashTab.get_tab_uniq_id()) {
                    foundLoc = true;
                    break;
                }
            }

            //check character tabs
            if (!foundLoc) {
                for (const auto& charTab : charsData) {
                    if (loc == charTab.get_character()) {
                        foundLoc = true;
                        break;
                    }
                }
            }

            //loc not found in either tab storage, delete record from 'items'
            if (!foundLoc) {
                query = QSqlQuery(db);
                query.prepare("DELETE FROM items WHERE loc = ?");
                query.bindValue(0, loc);
                if (query.exec() == false) {
                    spdlog::error("Error deleting items where loc is {}", loc);
                };
            }
        }
    }
}

QString SqliteDataStore::Get(const QString& key, const QString& default_value) {
    QSqlDatabase db = getThreadLocalDatabase();
    QSqlQuery query(db);
    query.prepare("SELECT value FROM data WHERE key = ?");
    query.bindValue(0, key);
    if (query.exec() == false) {
        spdlog::error("Error getting data for {}: {}", key, query.lastError().text());
        return default_value;
    };
    if (query.next() == false) {
        if (query.isActive() == false) {
            spdlog::error("Error getting result for {}: {}", key, query.lastError().text());
        };
        return default_value;
    };
    QString result = query.value(0).toByteArray();
    return result;
}

Locations SqliteDataStore::GetTabs(const ItemLocationType type) {
    QSqlDatabase db = getThreadLocalDatabase();
    QSqlQuery query(db);
    query.prepare("SELECT value FROM tabs WHERE type = ?");
    query.bindValue(0, (int)type);
    if (query.exec() == false) {
        spdlog::error("Error getting tabs for type {}: {}", (int)type, query.lastError().text());
        return {};
    };
    if (query.next() == false) {
        if (query.isActive() == false) {
            spdlog::error("Error getting result for {}: {}", (int)type, query.lastError().text());
        };
        return {};
    };
    const QString json = query.value(0).toString();
    return DeserializeTabs(json);
}

Items SqliteDataStore::GetItems(const ItemLocation& loc) {
    const QString tab_uid = loc.get_tab_uniq_id();
    QSqlDatabase db = getThreadLocalDatabase();
    QSqlQuery query(db);
    query.prepare("SELECT value FROM items WHERE loc = ?");
    query.bindValue(0, tab_uid);
    if (query.exec() == false) {
        spdlog::error("Error getting items for {}: {}", tab_uid, query.lastError().text());
        return {};
    };
    if (query.next() == false) {
        if (query.isActive() == false) {
            spdlog::error("Error getting result for {}: {}", tab_uid, query.lastError().text());
        };
        return {};
    };
    const QString json = query.value(0).toString();
    return DeserializeItems(json, loc);
}

void SqliteDataStore::Set(const QString& key, const QString& value) {
    QSqlDatabase db = getThreadLocalDatabase();
    QSqlQuery query(db);
    query.prepare("INSERT OR REPLACE INTO data (key, value) VALUES (?, ?)");
    query.bindValue(0, key);
    query.bindValue(1, value);
    if (query.exec() == false) {
        spdlog::error("Error setting value {}", key);
    };
}

void SqliteDataStore::SetTabs(const ItemLocationType type, const Locations& tabs) {
    QSqlDatabase db = getThreadLocalDatabase();
    QSqlQuery query(db);
    query.prepare("INSERT OR REPLACE INTO tabs (type, value) VALUES (?, ?)");
    query.bindValue(0, (int)type);
    query.bindValue(1, Serialize(tabs));
    if (query.exec() == false) {
        spdlog::error("Error setting tabs for type {}", (int)type);
    };
}

void SqliteDataStore::SetItems(const ItemLocation& loc, const Items& items) {
    if (loc.get_tab_uniq_id().isEmpty()) {
        spdlog::warn("Cannot set items because the location is empty");
        return;
    };
    QSqlDatabase db = getThreadLocalDatabase();
    QSqlQuery query(db);
    query.prepare("INSERT OR REPLACE INTO items (loc, value) VALUES (?, ?)");
    query.bindValue(0, loc.get_tab_uniq_id());
    query.bindValue(1, Serialize(items));
    if (query.exec() == false) {
        spdlog::error("Error setting tabs for type {}", loc.get_tab_uniq_id());
    };
}

void SqliteDataStore::InsertCurrencyUpdate(const CurrencyUpdate& update) {
    QSqlDatabase db = getThreadLocalDatabase();
    QSqlQuery query(db);
    query.prepare("INSERT INTO currency (timestamp, value) VALUES (?, ?)");
    query.bindValue(0, update.timestamp);
    query.bindValue(1, update.value);
    if (query.exec() == false) {
        spdlog::error("Error inserting currency update.");
    };
}

std::vector<CurrencyUpdate> SqliteDataStore::GetAllCurrency() {
    QSqlDatabase db = getThreadLocalDatabase();
    QSqlQuery query(db);
    query.prepare("SELECT timestamp, value FROM currency ORDER BY timestamp ASC");
    std::vector<CurrencyUpdate> result;
    if (query.exec() == false) {
        spdlog::error("Error getting currency updates: {}", query.lastError().text());
        return {};
    };
    while (query.next()) {
        CurrencyUpdate update = CurrencyUpdate();
        update.timestamp = query.value(0).toLongLong();
        update.value = query.value(1).toString();
        result.push_back(update);
    };
    if (query.lastError().isValid()) {
        spdlog::error("Error getting currency.");
        return {};
    };
    return result;
}

QString SqliteDataStore::MakeFilename(const QString& username, const QString& league) {
    // We somehow have to manage the fact that usernames now have a numeric discriminator,
    // e.g. GERWARIC#7694 instead of just GERWARIC.
    if (username.contains("#")) {
        // Build the filename as though the username did not have a discriminator,
        // then append the discriminator. This approach makes it possible to recognise
        // old data files more easily, because the discriminator is kept out of the hash.
        const QStringList parts = username.split("#");
        const QString base_username = parts[0];
        const QString discriminator = parts[1];
        const std::string key = base_username.toStdString() + "|" + league.toStdString();
        const QString datafile = QString(QCryptographicHash::hash(key.c_str(), QCryptographicHash::Md5).toHex()) + "-" + discriminator;
        return datafile;
    } else {
        const std::string key = username.toStdString() + "|" + league.toStdString();
        const QString datafile = QString(QCryptographicHash::hash(key.c_str(), QCryptographicHash::Md5).toHex());
        return datafile;
    };
}
