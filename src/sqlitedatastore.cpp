/*
	Copyright 2014 Ilya Zhuravlev

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
#include <QMutexLocker>
#include <QSqlError>
#include <QSqlQuery>

#include <list>

#include "QsLog.h"
#include "currencymanager.h"

DataStoreConnectionManager SqliteDataStore::manager_;

SqliteDataStore::SqliteDataStore(const QString& filename) :
	filename_(filename)
{
	QDir dir(QDir::cleanPath(filename + "/.."));
	if (!dir.exists())
		QDir().mkpath(dir.path());

	CreateTable("data", "key TEXT PRIMARY KEY, value BLOB");
	CreateTable("tabs", "type INT PRIMARY KEY, value BLOB");
	CreateTable("items", "loc TEXT PRIMARY KEY, value BLOB");
	CreateTable("currency", "timestamp INTEGER PRIMARY KEY, value TEXT");
	CleanItemsTable();

	auto& db = manager_.GetConnection(filename_);
	QMutexLocker locker(db.mutex);
	QSqlQuery query(db.database);
	query.prepare("VACUUM");
	if (query.exec() == false) {
		QLOG_ERROR() << "Failed to vacuum sqlite3 database:" << filename << ":" << db.database.lastError().text();
	};
}

void SqliteDataStore::CreateTable(const std::string& name, const std::string& fields) {
	const QString qname = QString::fromStdString(name);
	const QString qfields = QString::fromStdString(fields);
	auto& db = manager_.GetConnection(filename_);
	QMutexLocker locker(db.mutex);
	QSqlQuery query(db.database);
	query.prepare("CREATE TABLE IF NOT EXISTS " + qname + "(" + qfields + ")");
	if (query.exec() == false) {
		QLOG_ERROR() << "CreatTable faile on" << qname << ":" << query.lastError().text();
	};
}

void SqliteDataStore::CleanItemsTable() {
	auto& db = manager_.GetConnection(filename_);
	QMutexLocker locker(db.mutex);
	QSqlQuery query(db.database);
	query.prepare("DELETE FROM items WHERE loc IS NULL");
	if (query.exec() == false) {
		QLOG_ERROR() << "CleanItemsTable(): error deleting items where loc is null.";
		return;
	};

	//If tabs table contains two records which are not empty or NULL (i.e. type column is equal to 0 or 1 for the two records)
	//  * check all "db.items" record keys against 'id' or 'name' values in the "db.tabs" data,
	//    remove record from 'items' if not anywhere in either 'tabs' record.
	locker.unlock();
	const Locations stash_tabs = GetTabs(ItemLocationType::STASH);
	const Locations character_tabs = GetTabs(ItemLocationType::CHARACTER);
	locker.relock();

	if (stash_tabs.empty() || character_tabs.empty()) {
		return;
	};

	query = QSqlQuery(db.database);
	query.setForwardOnly(true);
	query.prepare("SELECT loc FROM items");
	if (query.exec() == false) {
		QLOG_ERROR() << "CleanItemsTable(): error selecting loc from items.";
		return;
	};

	std::list<std::string> locs;
	while (query.next()) {
		if (query.lastError().isValid()) {
			QLOG_ERROR() << "CleanItemsTable(): error moving to next loc";
			return;
		};
		locs.push_back(query.value(0).toString().toStdString());
	};
	query.finish();

	// Keep track of the number of orphaned locations we find.
	int n = 0;

	for (const auto& loc : locs) {

		bool foundLoc = false;

		// Check stash tabs
		for (const auto& stash : stash_tabs) {
			if (stash.get_tab_uniq_id() == loc) {
				foundLoc = true;
				break;
			};
		};

		// Check character tabs
		if (!foundLoc) {
			for (const auto& character : character_tabs) {
				if (character.get_tab_uniq_id() == loc) {
					foundLoc = true;
					break;
				};
			};
		};

		// Delete items in this location since the location appears to be orphaned.
		if (!foundLoc) {
			query = QSqlQuery(db.database);
			query.prepare("DELETE FROM items WHERE loc = ?");
			query.bindValue(0, QString::fromStdString(loc));
			if (query.exec() == false) {
				QLOG_ERROR() << "Error deleting items where loc is" << loc;
			};
			++n;
		};
	};
	QLOG_INFO() << "Items from" << n << "orphaned locations were removed from the database.";
}

std::string SqliteDataStore::Get(const std::string& key, const std::string& default_value) {
	auto& db = manager_.GetConnection(filename_);
	QMutexLocker locker(db.mutex);
	QSqlQuery query(db.database);
	query.prepare("SELECT value FROM data WHERE key = ?");
	query.bindValue(0, QString::fromStdString(key));
	if (query.exec() == false) {
		QLOG_ERROR() << "Error getting data for" << key.c_str() << ":" << query.lastError().text();
		return default_value;
	};
	if (query.next() == false) {
		if (query.isActive() == false) {
			QLOG_ERROR() << "Error getting result for" << key.c_str() << ":" << query.lastError().text();
		};
		return default_value;
	};
	std::string result = query.value(0).toByteArray().toStdString();
	return result;
}

std::vector<ItemLocation> SqliteDataStore::GetTabs(const ItemLocationType& type) {
	auto& db = manager_.GetConnection(filename_);
	QMutexLocker locker(db.mutex);
	QSqlQuery query(db.database);
	query.prepare("SELECT value FROM tabs WHERE type = ?");
	query.bindValue(0, (int)type);
	if (query.exec() == false) {
		QLOG_ERROR() << "Error getting tabs for type" << (int)type << ":" << query.lastError().text();
		return {};
	};
	if (query.next() == false) {
		if (query.isActive() == false) {
			QLOG_ERROR() << "Error getting result for" << (int)type << ":" << query.lastError().text();
		};
		return {};
	};
	const QString result = query.value(0).toString();
	return DeserializeTabs(result);
}

Items SqliteDataStore::GetItems(const ItemLocation& loc) {
	const QString tab_uid = QString::fromStdString(loc.get_tab_uniq_id());
	auto& db = manager_.GetConnection(filename_);
	QMutexLocker locker(db.mutex);
	QSqlQuery query(db.database);
	query.prepare("SELECT value FROM items WHERE loc = ?");
	query.bindValue(0, tab_uid);
	if (query.exec() == false) {
		QLOG_ERROR() << "Error getting items for" << tab_uid << ":" << query.lastError().text();
		return {};
	};
	if (query.next() == false) {
		if (query.isActive() == false) {
			QLOG_ERROR() << "Error getting result for" << tab_uid << ":" << query.lastError().text();
		};
		return {};
	};
	const QString result = query.value(0).toString();
	return DeserializeItems(result, loc);
}

void SqliteDataStore::Set(const std::string& key, const std::string& value) {
	auto& db = manager_.GetConnection(filename_);
	QMutexLocker locker(db.mutex);
	QSqlQuery query(db.database);
	query.prepare("INSERT OR REPLACE INTO data (key, value) VALUES (?, ?)");
	query.bindValue(0, QString::fromStdString(key));
	query.bindValue(1, QString::fromStdString(value));
	if (query.exec() == false) {
		QLOG_ERROR() << "Error setting value" << key.c_str();
	};
}

void SqliteDataStore::SetTabs(const ItemLocationType& type, const LocationList& tabs) {
	auto& db = manager_.GetConnection(filename_);
	QMutexLocker locker(db.mutex);
	QSqlQuery query(db.database);
	query.prepare("INSERT OR REPLACE INTO tabs (type, value) VALUES (?, ?)");
	query.bindValue(0, (int)type);
	query.bindValue(1, Serialize(tabs));
	if (query.exec() == false) {
		QLOG_ERROR() << "Error setting tabs for type" << (int)type;
	};
}

void SqliteDataStore::SetItems(const ItemLocation& loc, const ItemList& items) {
	const QString location_uid = QString::fromStdString(loc.get_tab_uniq_id());
	if (location_uid.isEmpty()) {
		QLOG_WARN() << "Cannot set items because the location is empty";
		return;
	};
	auto& db = manager_.GetConnection(filename_);
	QMutexLocker locker(db.mutex);
	QSqlQuery query(db.database);
	query.prepare("INSERT OR REPLACE INTO items (loc, value) VALUES (?, ?)");
	query.bindValue(0, QString::fromStdString(loc.get_tab_uniq_id()));
	query.bindValue(1, Serialize(items));
	if (query.exec() == false) {
		QLOG_ERROR() << "Error saving items for location" << location_uid;
	};
}

void SqliteDataStore::InsertCurrencyUpdate(const CurrencyUpdate& update) {
	auto& db = manager_.GetConnection(filename_);
	QMutexLocker locker(db.mutex);
	QSqlQuery query(db.database);
	query.prepare("INSERT INTO currency (timestamp, value) VALUES (?, ?)");
	query.bindValue(0, update.timestamp);
	query.bindValue(1, QString::fromStdString(update.value));
	if (query.exec() == false) {
		QLOG_ERROR() << "Error inserting currency update.";
	};
}

std::vector<CurrencyUpdate> SqliteDataStore::GetAllCurrency() {
	auto& db = manager_.GetConnection(filename_);
	QMutexLocker locker(db.mutex);
	QSqlQuery query(db.database);
	query.prepare("SELECT timestamp, value FROM currency ORDER BY timestamp ASC");
	std::vector<CurrencyUpdate> result;
	while (query.next()) {
		if (query.lastError().isValid()) {
			QLOG_ERROR() << "Error getting currency.";
			return {};
		};
		CurrencyUpdate update = CurrencyUpdate();
		update.timestamp = query.value(0).toLongLong();
		update.value = query.value(0).toByteArray().toStdString();
		result.push_back(update);
	};
	return result;
}

void SqliteDataStore::SetBool(const std::string& key, bool value) {
	SetInt(key, static_cast<int>(value));
}

bool SqliteDataStore::GetBool(const std::string& key, bool default_value) {
	return static_cast<bool>(GetInt(key, static_cast<int>(default_value)));
}

void SqliteDataStore::SetInt(const std::string& key, int value) {
	Set(key, std::to_string(value));
}

int SqliteDataStore::GetInt(const std::string& key, int default_value) {
	return std::stoi(Get(key, std::to_string(default_value)));
}

SqliteDataStore::~SqliteDataStore() {
	QLOG_TRACE() << "Sqlite data store is begin destroyed:" << filename_;
	manager_.Disconnect(filename_);
}

QString SqliteDataStore::MakeFilename(const std::string& name, const std::string& league) {
	std::string key = name + "|" + league;
	return QString(QCryptographicHash::hash(key.c_str(), QCryptographicHash::Md5).toHex());
}
