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

#pragma once

#include <QMutex>
#include <QString>
#include <QSqlDatabase>

#include <string>
#include <vector>

#include "datastore.h"
#include "porting.h"

class Application;
struct CurrencyUpdate;

class SqliteDataStore : public DataStore {
public:
	SqliteDataStore(const QString& filename_);
	~SqliteDataStore();
	void Set(const std::string& key, const std::string& value);
	void SetTabs(const ItemLocationType& type, const LocationList& tabs);
	void SetItems(const ItemLocation& loc, const ItemList& items);
	std::string Get(const std::string& key, const std::string& default_value = "");
	Locations GetTabs(const ItemLocationType& type);
	Items GetItems(const ItemLocation& loc);
	void InsertCurrencyUpdate(const CurrencyUpdate& update);
	std::vector<CurrencyUpdate> GetAllCurrency();
	static QString MakeFilename(const std::string& name, const std::string& league);
private:
	void SimpleQuery(DataStoreConnection& db, const QString& query_str);
	//void CreateTable(const std::string& name, const std::string& fields);
	//void CleanItemsTable();

	const QString filename_;

	DataStoreConnection& GetConnection();
	static DataStoreConnectionManager manager_;
};
