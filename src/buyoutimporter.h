/*
	Copyright 2023 Gerwaric

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

#include <QString>

#include <map>
#include <string>

#include "buyout.h"

class QSqlDatabase;

class BuyoutImporter {
public:
	BuyoutImporter(const QString& filename);
	const std::string version() const { return version_; };
	const std::string db_version() const { return db_version_; };
	const std::map<std::string, Buyout>& item_buyouts() const { return item_buyouts_; };
	const std::map<std::string, Buyout>& stash_buyouts() const { return stash_buyouts_; };
	const std::map<std::string, Buyout>& character_buyouts() const { return character_buyouts_; };
private:
	void ImportTabBuyouts(QSqlDatabase& db);
	void ImportItemBuyouts(QSqlDatabase& db);
	const QString filename_;
	std::string version_;
	std::string db_version_;
	std::map<std::string, Buyout> item_buyouts_;
	std::map<std::string, Buyout> stash_buyouts_;
	std::map<std::string, Buyout> character_buyouts_;
};
