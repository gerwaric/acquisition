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

#include <string>
#include <vector>

#include "datastore.h"
#include "porting.h"

class Application;
struct CurrencyUpdate;
struct sqlite3;

struct blob_info {
	byte* info;
	int len;
};

class SqliteDataStore : public DataStore {
public:
	SqliteDataStore(const std::string& filename_);
	~SqliteDataStore();
	void Set(const std::string& key, const std::string& value);
	void SetTabs(const ItemLocationType& type, const std::string& value);
	void SetItems(const ItemLocation& loc, const std::string& value);
	std::string Get(const std::string& key, const std::string& default_value = "");
	std::string GetTabs(const ItemLocationType& type, const std::string& default_value = "");
	std::string GetItems(const ItemLocation& loc, const std::string& default_value = "");
	void InsertCurrencyUpdate(const CurrencyUpdate& update);
	std::vector<CurrencyUpdate> GetAllCurrency();
	void SetBool(const std::string& key, bool value);
	bool GetBool(const std::string& key, bool default_value = false);
	void SetInt(const std::string& key, int value);
	int GetInt(const std::string& key, int default_value = 0);
	static std::string MakeFilename(const std::string& name, const std::string& league);
private:
	void CreateTable(const std::string& name, const std::string& fields);
	void CleanItemsTable();

	std::string filename_;
	sqlite3* db_;
};
