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

#include "QsLog.h"

DataStoreConnection& DataStoreConnectionManager::GetConnection(const QString& filename) {

	QMutexLocker locker(&mutex_);

	// Get a thread identifier, since Qt doesn't provide one.
	QThread* thread = QThread::currentThread();
	if (thread_ids_.count(thread) == 0) {
		++thread_id_count_;
		const QString thread_id = QStringLiteral("Thread(%1)").arg(thread_id_count_);
		QLOG_DEBUG() << "Creating thread id for database connection:" << thread_id;
		thread_ids_[thread] = thread_id;
		QObject::connect(thread, &QThread::finished, this, &DataStoreConnectionManager::OnThreadFinished);
	};

	// Connections are created for each filename-thread combination.
	const QString connection_id = thread_ids_[thread] + ":" + filename;

	// Reuse an existing connection if possible.
	if (connections_.count(connection_id) > 0) {
		auto& connection = connections_[connection_id];
		connection.count++;
		return connection;
	};

	// Create a new connection for this thread and file.
	const QString thread_id = thread_ids_[thread];
	QLOG_DEBUG() << "Adding new sqlite database connection for" << thread_id << "to" << filename;

	// Save the connection info.
	connection_ids_[thread].push_back(connection_id);
	connections_.emplace(connection_id, DataStoreConnection());

	auto& connection = connections_[connection_id];
	connection.database = QSqlDatabase::addDatabase("QSQLITE", connection_id);
	connection.database.setDatabaseName(filename);
	if (connection.database.open() == false) {
		QLOG_ERROR() << "Error adding database connection for" << thread_id << "to" << filename;
	};
	connection.mutex = new QMutex;
	connection.count = 1;
	return connection;
}

void DataStoreConnectionManager::OnThreadFinished() {
	QMutexLocker locker(&mutex_);
	QThread* thread = QThread::currentThread();
	const QString thread_id = thread_ids_[thread];
	QLOG_DEBUG() << "Removing thread id from database connections:" << thread_id;
	thread_ids_.erase(thread);
	for (const QString& connection_id : connection_ids_[thread]) {
		connections_.erase(connection_id);
	};
};

void DataStoreConnectionManager::Disconnect(const QString& filename) {
	QMutexLocker locker(&mutex_);
	const QString connection_id = thread_ids_[QThread::currentThread()] + ":" + filename;
	if (connections_.count(connection_id) == 0) {
		QLOG_ERROR() << "Could not disconnect from a database that doesn't exist.";
		return;
	};
	auto& connection = connections_[connection_id];
	const QString name = connection.database.connectionName();
	QLOG_DEBUG() << "Closeing sqlite database connection for" << name;
	if (--connection.count <= 0) {
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