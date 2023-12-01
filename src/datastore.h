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
#include <QObject>
#include <QSqlDatabase>
#include <QString>
#include <QStringList>
#include <QThread>

#include <list>
#include <memory>
#include <string>
#include <unordered_map>
#include <vector>

#include "poe_api/poe_typedefs.h"

#include "currencymanager.h"
#include "itemlocation.h"
#include "item.h"
#include "mainwindow.h"

struct DataStoreConnection {
	QSqlDatabase database;
	QMutex* mutex;
	int count;
};

class DataStoreConnectionManager : public QObject {
	Q_OBJECT
public:
	static DataStoreConnectionManager& instance();
	DataStoreConnection& GetConnection(const QString filename);
	void Disconnect(const QString filename);
private slots:
	void OnThreadFinished();
private:
	DataStoreConnectionManager() {};
	int thread_id_count_{ 0 };
	std::unordered_map<QThread*, QString> thread_ids_;
	std::unordered_map<QThread*, QStringList> connection_ids_;
	std::unordered_map<QString, DataStoreConnection> connections_;
	QMutex mutex_;
};

class DataStore : public QObject {
	Q_OBJECT

public:
	static QString MakeFilename(const std::string name, const std::string league);

	DataStore(const QString& filename_);
	~DataStore();

	const QString filename() const { return filename_; };

	void Set(const std::string key, const std::string value);
	void SetInt(const std::string key, int value);
	void SetBool(const std::string key, bool value);
	void SetStash(const PoE::StashTab& stash);
	void SetCharacter(const PoE::Character& character);
	void InsertCurrencyUpdate(const CurrencyUpdate& update);

	std::string Get(const std::string key, const std::string default_value = "");
	int GetInt(const std::string key, int default_value = 0);
	bool GetBool(const std::string key, bool default_value = false);
	std::unordered_map<PoE::StashId, PoE::StashTab> GetStashes();
	std::unordered_map<PoE::CharacterName, PoE::Character> GetCharacters();
	std::vector<CurrencyUpdate> GetAllCurrency();

signals:
	void StatusUpdate(const CurrentStatusUpdate& status);

private:
	void SimpleQuery(DataStoreConnection& db, const QString& query_str);
	const QString filename_;
	DataStoreConnectionManager& manager_;
};
