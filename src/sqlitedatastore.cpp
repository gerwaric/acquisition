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

#include "sqlite/sqlite3.h"
#include <QCryptographicHash>
#include <QDir>
#include <ctime>
#include <stdexcept>

#include "currencymanager.h"
#include "util.h"

SqliteDataStore::SqliteDataStore(const std::string &filename) :
	filename_(filename)
{
	QDir dir(QDir::cleanPath((filename + "/..").c_str()));
	if (!dir.exists())
		QDir().mkpath(dir.path());
	if (sqlite3_open(filename_.c_str(), &db_) != SQLITE_OK) {
		throw std::runtime_error("Failed to open sqlite3 database.");
	}
	CreateTable("data", "key TEXT PRIMARY KEY, value BLOB");
	CreateTable("tabs", "type INT PRIMARY KEY, value BLOB");
	CreateTable("items", "loc TEXT PRIMARY KEY, value BLOB");
	CreateTable("currency", "timestamp INTEGER PRIMARY KEY, value TEXT");

	CleanItemsTable();

	sqlite3_exec(db_, "VACUUM", 0, 0, 0);
}

void SqliteDataStore::CreateTable(const std::string &name, const std::string &fields) {
	std::string query = "CREATE TABLE IF NOT EXISTS " + name + "(" + fields + ")";
	if (sqlite3_exec(db_, query.c_str(), 0, 0, 0) != SQLITE_OK) {
		throw std::runtime_error("Failed to execute creation statement for table " + name + ".");
	}
}

void SqliteDataStore::CleanItemsTable() {
	std::string query = "DELETE FROM items WHERE loc IS NULL";
	sqlite3_exec(db_, query.c_str(), 0, 0, 0);

	//If tabs table contains two records which are not empty or NULL (i.e. type column is equal to 0 or 1 for the two records)
	//  * check all "db.items" record keys against 'id' or 'name' values in the "db.tabs" data,
	//    remove record from 'items' if not anywhere in either 'tabs' record.
	std::string stashTabData = GetTabs(ItemLocationType::STASH, "NOT FOUND");
	std::string charsData = GetTabs(ItemLocationType::CHARACTER, "NOT FOUND");

	if (stashTabData.compare("NOT FOUND") != 0 && charsData.compare("NOT FOUND") != 0) {
		std::vector<std::string> locs;
		std::vector<blob_info> locs_byte;

		query = "SELECT loc FROM items";
		sqlite3_stmt* stmt;
		auto prepareResults = sqlite3_prepare_v2(db_, query.c_str(), -1, &stmt, NULL);
		while (sqlite3_step(stmt) == SQLITE_ROW) {
			int value_type = sqlite3_column_type(stmt, 0);
			std::string locStr(reinterpret_cast<const char*>(sqlite3_column_text(stmt, 0)));
			locs.push_back(locStr);

			blob_info blobI;
			blobI.len = sqlite3_column_bytes(stmt, 0);
			blobI.info = (byte*) malloc(blobI.len);
			memcpy(blobI.info, (byte*)sqlite3_column_blob(stmt, 0), blobI.len);
			
			locs_byte.push_back(blobI);
		}
		sqlite3_finalize(stmt);

		for (int i = 0; i < locs.size(); i++) {
			const std::string loc = locs[i];
			const blob_info loc_blob = locs_byte[i];

			rapidjson::Document doc;
			bool foundLoc = false;

			//check stash tabs
			doc.Parse(stashTabData.c_str());
			for (const rapidjson::Value const *tab = doc.Begin(); tab != doc.End(); ++tab) {
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
				for (const rapidjson::Value const* tab = doc.Begin(); tab != doc.End(); ++tab) {
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
				query = "DELETE FROM items WHERE hex(loc) = ?";
				sqlite3_prepare_v2(db_, query.c_str(), -1, &stmt, 0);
				sqlite3_bind_text(stmt, 1, Util::hexStr(loc_blob.info, loc_blob.len).c_str(), -1, SQLITE_TRANSIENT);

				std::string sqlQry(sqlite3_sql(stmt));
				std::string sqlExtQry(sqlite3_expanded_sql(stmt));

				sqlite3_step(stmt);
				sqlite3_finalize(stmt);
			}
		}

		for (blob_info b : locs_byte) {
			free(b.info);
		}
	}
}

std::string SqliteDataStore::Get(const std::string &key, const std::string &default_value) {
	std::string query = "SELECT value FROM data WHERE key = ?";
	sqlite3_stmt *stmt;
	sqlite3_prepare_v2(db_, query.c_str(), -1, &stmt, 0);
	sqlite3_bind_text(stmt, 1, key.c_str(), -1, SQLITE_STATIC);
	std::string result(default_value);
	if (sqlite3_step(stmt) == SQLITE_ROW)
		result = std::string(static_cast<const char*>(sqlite3_column_blob(stmt, 0)), sqlite3_column_bytes(stmt, 0));
	sqlite3_finalize(stmt);
	return result;
}

std::string SqliteDataStore::GetTabs(const ItemLocationType &type, const std::string &default_value) {
	std::string query = "SELECT value FROM tabs WHERE type = ?";
	sqlite3_stmt *stmt;
	sqlite3_prepare_v2(db_, query.c_str(), -1, &stmt, 0);
	sqlite3_bind_int(stmt, 1, (int) type);
	std::string result(default_value);
	if (sqlite3_step(stmt) == SQLITE_ROW)
		result = std::string(static_cast<const char*>(sqlite3_column_blob(stmt, 0)), sqlite3_column_bytes(stmt, 0));
	sqlite3_finalize(stmt);
	return result;
}

std::string SqliteDataStore::GetItems(const ItemLocation &loc, const std::string &default_value) {
	std::string query = "SELECT value FROM items WHERE loc = ?";
	std::string location = loc.get_tab_uniq_id();

	sqlite3_stmt *stmt;
	auto prepareResults = sqlite3_prepare_v2(db_, query.c_str(), -1, &stmt, NULL);
	auto bindResults = sqlite3_bind_text(stmt, 1, location.c_str(), -1, SQLITE_STATIC);
	std::string result(default_value);

	auto rslt = sqlite3_step(stmt);

	if (rslt == SQLITE_ROW)
		result = std::string(static_cast<const char*>(sqlite3_column_blob(stmt, 0)), sqlite3_column_bytes(stmt, 0));
	sqlite3_finalize(stmt);
	return result;
}

void SqliteDataStore::Set(const std::string &key, const std::string &value) {
	std::string query = "INSERT OR REPLACE INTO data (key, value) VALUES (?, ?)";
	sqlite3_stmt *stmt;
	sqlite3_prepare_v2(db_, query.c_str(), -1, &stmt, 0);
	sqlite3_bind_text(stmt, 1, key.c_str(), -1, SQLITE_STATIC);
	sqlite3_bind_blob(stmt, 2, value.c_str(), value.size(), SQLITE_STATIC);
	sqlite3_step(stmt);
	sqlite3_finalize(stmt);
}

void SqliteDataStore::SetTabs(const ItemLocationType &type, const std::string &value) {
	std::string query = "INSERT OR REPLACE INTO tabs (type, value) VALUES (?, ?)";
	sqlite3_stmt *stmt;
	sqlite3_prepare_v2(db_, query.c_str(), -1, &stmt, 0);
	sqlite3_bind_int(stmt, 1, (int) type);
	sqlite3_bind_blob(stmt, 2, value.c_str(), value.size(), SQLITE_STATIC);
	sqlite3_step(stmt);
	sqlite3_finalize(stmt);
}

void SqliteDataStore::SetItems(const ItemLocation &loc, const std::string &value) {
	if (loc.get_tab_uniq_id().empty())
		return;

	std::string query = "INSERT OR REPLACE INTO items (loc, value) VALUES (?, ?)";
	sqlite3_stmt *stmt;
	sqlite3_prepare_v2(db_, query.c_str(), -1, &stmt, 0);
	sqlite3_bind_text(stmt, 1, loc.get_tab_uniq_id().c_str(), -1, SQLITE_TRANSIENT);
	sqlite3_bind_blob(stmt, 2, value.c_str(), value.size(), SQLITE_STATIC);
	sqlite3_step(stmt);
	sqlite3_finalize(stmt);
}

void SqliteDataStore::InsertCurrencyUpdate(const CurrencyUpdate &update) {
	std::string query = "INSERT INTO currency (timestamp, value) VALUES (?, ?)";
	sqlite3_stmt *stmt;
	sqlite3_prepare_v2(db_, query.c_str(), -1, &stmt, 0);
	sqlite3_bind_int64(stmt, 1, update.timestamp);
	sqlite3_bind_text(stmt, 2, update.value.c_str(), -1, SQLITE_STATIC);
	sqlite3_step(stmt);
	sqlite3_finalize(stmt);
}

std::vector<CurrencyUpdate> SqliteDataStore::GetAllCurrency() {
	std::string query = "SELECT timestamp, value FROM currency ORDER BY timestamp ASC";
	sqlite3_stmt* stmt;
	sqlite3_prepare_v2(db_, query.c_str(), -1, &stmt, 0);
	std::vector<CurrencyUpdate> result;
	while (sqlite3_step(stmt) == SQLITE_ROW) {
		CurrencyUpdate update = CurrencyUpdate();
		update.timestamp = sqlite3_column_int64(stmt, 0);
		update.value = std::string(reinterpret_cast<const char*>(sqlite3_column_text(stmt, 1)));
		result.push_back(update);
	}
	sqlite3_finalize(stmt);
	return result;
}

void SqliteDataStore::SetBool(const std::string &key, bool value) {
	SetInt(key, static_cast<int>(value));
}

bool SqliteDataStore::GetBool(const std::string &key, bool default_value) {
	return static_cast<bool>(GetInt(key, static_cast<int>(default_value)));
}

void SqliteDataStore::SetInt(const std::string &key, int value) {
	Set(key, std::to_string(value));
}

int SqliteDataStore::GetInt(const std::string &key, int default_value) {
	return std::stoi(Get(key, std::to_string(default_value)));
}

SqliteDataStore::~SqliteDataStore() {
	sqlite3_close(db_);
}

std::string SqliteDataStore::MakeFilename(const std::string &name, const std::string &league) {
	std::string key = name + "|" + league;
	return QString(QCryptographicHash::hash(key.c_str(), QCryptographicHash::Md5).toHex()).toStdString();
}
