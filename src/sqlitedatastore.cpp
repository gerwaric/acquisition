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

#include "QsLog.h"
#include "currencymanager.h"

QMutex SqliteDataStore::mutex_;

SqliteDataStore::SqliteDataStore(const QString& filename) :
	filename_(filename)
{
	QDir dir(QDir::cleanPath(filename + "/.."));
	if (!dir.exists())
		QDir().mkpath(dir.path());

	QMutexLocker locker(&mutex_);

	db_ = QSqlDatabase::addDatabase("QSQLITE", filename);
	db_.setDatabaseName(filename);
	if (db_.open() == false) {
		QLOG_ERROR() << "Failed to open QSQLITE database:" << filename << ":" << db_.lastError().text();
		return;
	};

	locker.unlock();
	CreateTable("data", "key TEXT PRIMARY KEY, value BLOB");
	CreateTable("tabs", "type INT PRIMARY KEY, value BLOB");
	CreateTable("items", "loc TEXT PRIMARY KEY, value BLOB");
	CreateTable("currency", "timestamp INTEGER PRIMARY KEY, value TEXT");
	CleanItemsTable();
	locker.relock();

	QSqlQuery query(db_);
	query.prepare("VACUUM");
	if (query.exec() == false) {
		QLOG_ERROR() << "SqliteDataStore: failed to vacuum QSQLITE database:" << filename << ":" << db_.lastError().text();
	};
}

void SqliteDataStore::CreateTable(const std::string& name, const std::string& fields) {
	QMutexLocker locker(&mutex_);
	const QString qname = QString::fromStdString(name);
	const QString qfields = QString::fromStdString(fields);
	QSqlQuery query(db_);
	query.prepare("CREATE TABLE IF NOT EXISTS " + qname + "(" + qfields + ")");
	if (query.exec() == false) {
		QLOG_ERROR() << "CreateTable(): failed to create" << qname << ":" << query.lastError().text();
	};
}

void SqliteDataStore::CleanItemsTable() {
	QMutexLocker locker(&mutex_);
	QSqlQuery query(db_);
	query.prepare("DELETE FROM items WHERE loc IS NULL");
	if (query.exec() == false) {
		QLOG_ERROR() << "CleanItemsTable(): error deleting items where loc is null.";
		return;
	};

	//If tabs table contains two records which are not empty or NULL (i.e. type column is equal to 0 or 1 for the two records)
	//  * check all "db.items" record keys against 'id' or 'name' values in the "db.tabs" data,
	//    remove record from 'items' if not anywhere in either 'tabs' record.
	locker.unlock();
	std::string stashTabData = GetTabs(ItemLocationType::STASH, "NOT FOUND");
	std::string charsData = GetTabs(ItemLocationType::CHARACTER, "NOT FOUND");
	locker.relock();

	if ((stashTabData.compare("NOT FOUND") != 0) && (charsData.compare("NOT FOUND") != 0)) {
		std::list<QByteArray> locs;

		query = QSqlQuery(db_);
		query.setForwardOnly(true);
		query.prepare("SELECT loc FROM items");
		if (query.exec() == false) {
			QLOG_ERROR() << "CleanItemsTable(): error selecting loc from items.";
			return;
		};
		while (query.next()) {
			if (query.lastError().isValid()) {
				QLOG_ERROR() << "CleanItemsTable(): error moving to next loc";
				return;
			};
			QByteArray bytes = query.value(0).toByteArray();
			locs.push_back(bytes);
		};
		query.finish();

		for (const auto& loc : locs) {

			rapidjson::Document doc;
			bool foundLoc = false;

			//check stash tabs
			doc.Parse(stashTabData.c_str());
			for (const rapidjson::Value* tab = doc.Begin(); tab != doc.End(); ++tab) {
				if (tab->HasMember("id") && (*tab)["id"].IsString()) {
					std::string tabLoc((*tab)["id"].GetString());
					if (tabLoc.compare(loc) == 0) {
						foundLoc = true;
						break;
					}
				}
			}

			//check character tabs
			if (!foundLoc) {
				doc.Parse(charsData.c_str());
				for (const rapidjson::Value* tab = doc.Begin(); tab != doc.End(); ++tab) {
					if (tab->HasMember("name") && (*tab)["name"].IsString()) {
						std::string tabLoc((*tab)["name"].GetString());
						if (tabLoc.compare(loc) == 0) {
							foundLoc = true;
							break;
						}
					}
				}
			}

			//loc not found in either tab storage, delete record from 'items'
			if (!foundLoc) {
				query = QSqlQuery(db_);
				query.prepare("DELETE FROM items WHERE loc = ?");
				query.bindValue(0, loc);
				if (query.exec() == false) {
					QLOG_ERROR() << "Error deleting items where loc is" << loc;
				};
			}
		}
	}
}

std::string SqliteDataStore::Get(const std::string& key, const std::string& default_value) {
	QMutexLocker locker(&mutex_);
	QSqlQuery query(db_);
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

std::string SqliteDataStore::GetTabs(const ItemLocationType& type, const std::string& default_value) {
	QMutexLocker locker(&mutex_);
	QSqlQuery query(db_);
	query.prepare("SELECT value FROM tabs WHERE type = ?");
	query.bindValue(0, (int)type);
	if (query.exec() == false) {
		QLOG_ERROR() << "Error getting tabs for type" << (int)type << ":" << query.lastError().text();
		return default_value;
	};
	if (query.next() == false) {
		if (query.isActive() == false) {
			QLOG_ERROR() << "Error getting result for" << (int)type << ":" << query.lastError().text();
		};
		return default_value;
	};
	std::string result = query.value(0).toByteArray().toStdString();
	return result;
}

std::string SqliteDataStore::GetItems(const ItemLocation& loc, const std::string& default_value) {
	const QString tab_uid = QString::fromStdString(loc.get_tab_uniq_id());
	QMutexLocker locker(&mutex_);
	QSqlQuery query(db_);
	query.prepare("SELECT value FROM items WHERE loc = ?");
	query.bindValue(0, tab_uid);
	if (query.exec() == false) {
		QLOG_ERROR() << "Error getting items for" << tab_uid << ":" << query.lastError().text();
		return default_value;
	};
	if (query.next() == false) {
		if (query.isActive() == false) {
			QLOG_ERROR() << "Error getting result for" << tab_uid << ":" << query.lastError().text();
		};
		return default_value;
	};
	std::string result = query.value(0).toByteArray().toStdString();
	return result;
}

void SqliteDataStore::Set(const std::string& key, const std::string& value) {
	QMutexLocker locker(&mutex_);
	QSqlQuery query(db_);
	query.prepare("INSERT OR REPLACE INTO data (key, value) VALUES (?, ?)");
	query.bindValue(0, QString::fromStdString(key));
	query.bindValue(1, QString::fromStdString(value));
	if (query.exec() == false) {
		QLOG_ERROR() << "Error setting value" << key.c_str();
	};
}

void SqliteDataStore::SetTabs(const ItemLocationType& type, const std::string& value) {
	QMutexLocker locker(&mutex_);
	QSqlQuery query(db_);
	query.prepare("INSERT OR REPLACE INTO tabs (type, value) VALUES (?, ?)");
	query.bindValue(0, (int)type);
	query.bindValue(1, QString::fromStdString(value));
	if (query.exec() == false) {
		QLOG_ERROR() << "Error setting tabs for type" << (int)type;
	};
}

void SqliteDataStore::SetItems(const ItemLocation& loc, const std::string& value) {
	if (loc.get_tab_uniq_id().empty()) {
		QLOG_WARN() << "Cannot set items because the location is empty";
		return;
	};
	QMutexLocker locker(&mutex_);
	QSqlQuery query(db_);
	query.prepare("INSERT OR REPLACE INTO items (loc, value) VALUES (?, ?)");
	query.bindValue(0, QString::fromStdString(loc.get_tab_uniq_id()));
	query.bindValue(1, QString::fromStdString(value));
	if (query.exec() == false) {
		QLOG_ERROR() << "Error setting tabs for type" << loc.get_tab_uniq_id();
	};
}

void SqliteDataStore::InsertCurrencyUpdate(const CurrencyUpdate& update) {
	QMutexLocker locker(&mutex_);
	QSqlQuery query(db_);
	query.prepare("INSERT INTO currency (timestamp, value) VALUES (?, ?)");
	query.bindValue(0, update.timestamp);
	query.bindValue(1, QString::fromStdString(update.value));
	if (query.exec() == false) {
		QLOG_ERROR() << "Error inserting currency update.";
	};
}

std::vector<CurrencyUpdate> SqliteDataStore::GetAllCurrency() {
	QMutexLocker locker(&mutex_);
	QSqlQuery query(db_);
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
	QMutexLocker locker(&mutex_);
	if (db_.isValid()) {
		db_.close();
	};
}

QString SqliteDataStore::MakeFilename(const std::string& name, const std::string& league) {
	std::string key = name + "|" + league;
	return QString(QCryptographicHash::hash(key.c_str(), QCryptographicHash::Md5).toHex());
}
