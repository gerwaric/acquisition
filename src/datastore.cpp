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

#include "datastore.h"

#include <QSqlError>

#include "QsLog.h"
#include "rapidjson/error/en.h"
#include "util.h"

DataStoreConnection& DataStoreConnectionManager::GetConnection(const QString& filename) {

	QMutexLocker locker(&mutex_);

	// Get a thread identifier, since Qt doesn't provide one.
	QThread* thread = QThread::currentThread();
	if (thread_ids_.count(thread) == 0) {
		++thread_id_count_;
		const QString thread_id = QStringLiteral("Thread(%1)").arg(thread_id_count_);
		QLOG_DEBUG() << "Creating a new thread id for data store connections:" << thread_id;
		thread_ids_[thread] = thread_id;
		QObject::connect(thread, &QThread::finished, this, &DataStoreConnectionManager::OnThreadFinished);
	};

	// Connections are created for each filename-thread combination.
	const QString thread_id = thread_ids_[thread];
	const QString connection_id = thread_id + ":" + filename;

	// Reuse an existing connection if possible.
	if (connections_.count(connection_id) > 0) {
		auto& connection = connections_[connection_id];
		connection.count++;
		return connection;
	};

	// Create a new connection for this thread and file.
	QLOG_DEBUG() << "Creating a new data store connection:" << connection_id;

	// Save the connection info.
	connection_ids_[thread].push_back(connection_id);
	connections_.emplace(connection_id, DataStoreConnection());

	auto& connection = connections_[connection_id];
	connection.database = QSqlDatabase::addDatabase("QSQLITE", connection_id);
	connection.database.setDatabaseName(filename);
	if (connection.database.open() == false) {
		QLOG_ERROR() << "Error opening database for" << connection_id << ":" << connection.database.lastError().text();
	};
	connection.mutex = new QMutex;
	connection.count = 1;
	return connection;
}

void DataStoreConnectionManager::OnThreadFinished() {
	QMutexLocker locker(&mutex_);
	QThread* thread = QThread::currentThread();
	const QString thread_id = thread_ids_[thread];
	QLOG_DEBUG() << "Removing thread from data store connections:" << thread_id;
	thread_ids_.erase(thread);
	for (const QString& connection_id : connection_ids_[thread]) {
		QLOG_DEBUG() << "Removing data store connection:" << connection_id;
		auto& connection = connections_[connection_id];
		connection.database.close();
		delete(connection.mutex);
		connections_.erase(connection_id);
	};
};

void DataStoreConnectionManager::Disconnect(const QString& filename) {
	const QString thread_id = thread_ids_[QThread::currentThread()];
	const QString connection_id = thread_id + ":" + filename;
	QMutexLocker locker(&mutex_);
	if (connections_.count(connection_id) == 0) {
		// The current thread is not connected to this datastore's file.
		return;
	};
	QLOG_DEBUG() << "Closing data store connection:" << connection_id;
	auto& connection = connections_[connection_id];
	if (--connection.count <= 0) {
		QLOG_DEBUG() << "Removing unused data store connection:" << connection_id;
		connection.database.close();
		delete(connection.mutex);
		connections_.erase(connection_id);
	};
}

QString DataStore::Serialize(const DataStore::LocationList& tabs) {
	QStringList json_tabs;
	json_tabs.reserve(tabs.size());
	for (auto& tab : tabs) {
		json_tabs.push_back(QString::fromStdString(tab->get_json()));
	};
	return "[" + json_tabs.join(",") + "]";
}

QString DataStore::Serialize(const DataStore::ItemList& items) {
	QStringList json_items;
	json_items.reserve(items.size());
	for (auto& item : items) {
		json_items.push_back(QString::fromStdString(item->json()));
	};
	return "[" + json_items.join(",") + "]";
}
