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

#include <string>
#include <unordered_map>
#include <vector>

#include "currencymanager.h"
#include "itemlocation.h"

struct DataStoreConnection {
	QSqlDatabase database;
	QMutex* mutex;
	int count;
};

class DataStoreConnectionManager : public QObject {
	Q_OBJECT
public:
	DataStoreConnection& GetConnection(const QString& filename);
	void Disconnect(const QString& filename);
private slots:
	void OnThreadFinished();
private:
	int thread_id_count_{ 0 };
	std::unordered_map<QThread*, QString> thread_ids_;
	std::unordered_map<QThread*, QStringList> connection_ids_;
	std::unordered_map<QString, DataStoreConnection> connections_;
	QMutex mutex_;
};

class DataStore {
public:
	virtual ~DataStore() {};
	virtual void Set(const std::string& key, const std::string& value) = 0;
	virtual void SetTabs(const ItemLocationType& type, const std::string& value) = 0;
	virtual void SetItems(const ItemLocation& loc, const std::string& value) = 0;
	virtual std::string Get(const std::string& key, const std::string& default_value = "") = 0;
	virtual std::string GetTabs(const ItemLocationType& type, const std::string& default_value = "") = 0;
	virtual std::string GetItems(const ItemLocation& loc, const std::string& default_value = "") = 0;
	virtual void InsertCurrencyUpdate(const CurrencyUpdate& update) = 0;
	virtual std::vector<CurrencyUpdate> GetAllCurrency() = 0;
	virtual void SetBool(const std::string& key, bool value) = 0;
	virtual bool GetBool(const std::string& key, bool default_value = false) = 0;
	virtual void SetInt(const std::string& key, int value) = 0;
	virtual int GetInt(const std::string& key, int default_value = 0) = 0;
};
