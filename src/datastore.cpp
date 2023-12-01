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
#include <QSqlQuery>

#include "QsLog.h"
#include "util.h"

#include "poe_api/poe_character.h"
#include "poe_api/poe_stash.h"

DataStoreConnectionManager& DataStoreConnectionManager::instance() {
	static DataStoreConnectionManager manager;
	return manager;
}

DataStoreConnection& DataStoreConnectionManager::GetConnection(const QString filename) {

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

void DataStoreConnectionManager::Disconnect(const QString filename) {
	const QString thread_id = thread_ids_[QThread::currentThread()];
	const QString connection_id = thread_id + ":" + filename;
	QMutexLocker locker(&mutex_);
	if (connections_.count(connection_id) == 0) {
		// The current thread is not connected to this datastore's file.
		return;
	};
	QLOG_DEBUG() << "Closing data store connection:" << connection_id << "from" << thread_id;
	auto& connection = connections_[connection_id];
	if (--connection.count <= 0) {
		QLOG_DEBUG() << "Removing unused data store connection:" << connection_id;
		connection.database.close();
		delete(connection.mutex);
		connections_.erase(connection_id);
	};
}

DataStore::DataStore(const QString& filename) :
	filename_(filename),
	manager_(DataStoreConnectionManager::instance())
{
	if (filename_ != ":memory:") {
		QDir dir(QDir::cleanPath(filename + "/.."));
		if (!dir.exists()) {
			QDir().mkpath(dir.path());
		};
	};

	auto& db = manager_.GetConnection(filename_);
	SimpleQuery(db, "CREATE TABLE IF NOT EXISTS data (key TEXT PRIMARY KEY, value BLOB)");
	SimpleQuery(db, "CREATE TABLE IF NOT EXISTS stashes (id TEXT PRIMARY KEY, value BLOB)");
	SimpleQuery(db, "CREATE TABLE IF NOT EXISTS characters (name TEXT PRIMARY KEY, value BLOB)");
	SimpleQuery(db,
		"CREATE TABLE IF NOT EXISTS buyouts ("
		"  type INT NOT NULL,"
		"  id TEXT NOT NULL,"
		"  value BLOB,"
		"  PRIMARY KEY (type, id)"
		")");
	SimpleQuery(db, "CREATE TABLE IF NOT EXISTS currency (timestamp INT PRIMARY KEY, value TEXT)");
	SimpleQuery(db, "VACUUM");
}

DataStore::~DataStore() {
	QLOG_TRACE() << "Data store is being destroyed:" << filename_;
	manager_.Disconnect(filename_);
}

std::string DataStore::Get(const std::string key, const std::string default_value) {
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
	return query.value(0).toString().toStdString();
}

int DataStore::GetInt(const std::string key, int default_value) {
	return std::stoi(Get(key, std::to_string(default_value)));
}

bool DataStore::GetBool(const std::string key, bool default_value) {
	return static_cast<bool>(GetInt(key, static_cast<int>(default_value)));
}

std::unordered_map<PoE::StashId, PoE::StashTab> DataStore::GetStashes() {
	auto& db = manager_.GetConnection(filename_);
	QMutexLocker locker(db.mutex);
	QSqlQuery query(db.database);
	query.prepare("SELECT value FROM stashes");
	if (query.exec() == false) {
		QLOG_ERROR() << "Error selecting stashes:" << query.lastError().text();
		return {};
	};
	size_t count = 0;
	std::unordered_map<PoE::StashId, PoE::StashTab> result;
	while (query.next()) {
		++count;
		if (query.isActive() == false) {
			QLOG_ERROR() << "Error getting stash" << count << ":" << query.lastError().text();
			return {};
		};
		auto stash = PoE::StashTab(query.value(0).toString().toStdString());
		result.emplace(stash.id, stash);
	};
	return result;
}

std::unordered_map<PoE::CharacterName, PoE::Character> DataStore::GetCharacters() {
	auto& db = manager_.GetConnection(filename_);
	QMutexLocker locker(db.mutex);
	QSqlQuery query(db.database);
	query.prepare("SELECT value FROM characters");
	if (query.exec() == false) {
		QLOG_ERROR() << "Error selecting characters:" << query.lastError().text();
		return {};
	};
	size_t count = 0;
	std::unordered_map<PoE::CharacterName, PoE::Character> result;
	while (query.next()) {
		++count;
		if (query.isActive() == false) {
			QLOG_ERROR() << "Error getting character" << count << ":" << query.lastError().text();
			return {};
		};
		auto character = PoE::Character(query.value(0).toString().toStdString());
		result.emplace(character.name, character);
	};
	return result;
}

void DataStore::Set(const std::string key, const std::string value) {
	auto& db = manager_.GetConnection(filename_);
	QMutexLocker locker(db.mutex);
	QSqlQuery query(db.database);
	query.prepare("INSERT OR REPLACE INTO data (key, value) VALUES (?, ?)");
	query.bindValue(0, QString::fromStdString(key));
	query.bindValue(1, QString::fromStdString(value));
	if (query.exec() == false) {
		QLOG_ERROR() << "Error setting value for" << query.boundValue(0) << ":" << query.lastError().text();
	};
}

void DataStore::SetInt(const std::string key, int value) {
	Set(key, std::to_string(value));
}

void DataStore::SetBool(const std::string key, bool value) {
	SetInt(key, static_cast<int>(value));
}

void DataStore::SetStash(const PoE::StashTab& stash) {

	const QString id = QString::fromStdString(stash.id);
	const QString value = QString::fromStdString(JS::serializeStruct(stash, JS::SerializerOptions::Compact));

	auto& db = manager_.GetConnection(filename_);
	QMutexLocker locker(db.mutex);
	QSqlQuery query(db.database);
	query.prepare("INSERT OR REPLACE INTO stashes (id, value) VALUES (?, ?)");
	query.bindValue(0, id);
	query.bindValue(1, value);
	if (query.exec() == false) {
		QLOG_ERROR() << "Error inserting stash" << id << "(" << stash.type << "/" << stash.name << "):" << query.lastError().text();
	};
}

void DataStore::SetCharacter(const PoE::Character& character) {

	const QString name = QString::fromStdString(character.name);
	const QString value = QString::fromStdString(JS::serializeStruct(character, JS::SerializerOptions::Compact));

	auto& db = manager_.GetConnection(filename_);
	QMutexLocker locker(db.mutex);
	QSqlQuery query(db.database);
	query.prepare("INSERT OR REPLACE INTO characters (name, value) VALUES (?, ?)");
	query.bindValue(0, name);
	query.bindValue(1, value);
	if (query.exec() == false) {
		QLOG_ERROR() << "Error storing character:" << name << ":" << query.lastError().text();
	};
}

void DataStore::InsertCurrencyUpdate(const CurrencyUpdate& update) {
	auto& db = manager_.GetConnection(filename_);
	QMutexLocker locker(db.mutex);
	QSqlQuery query(db.database);
	query.prepare("INSERT INTO currency (timestamp, value) VALUES (?, ?)");
	query.bindValue(0, update.timestamp);
	query.bindValue(1, QString::fromStdString(update.value));
	if (query.exec() == false) {
		QLOG_ERROR() << "Error inserting currency update:" << query.lastError().text();
	};
}

std::vector<CurrencyUpdate> DataStore::GetAllCurrency() {
	auto& db = manager_.GetConnection(filename_);
	QMutexLocker locker(db.mutex);
	QSqlQuery query(db.database);
	query.prepare("SELECT timestamp, value FROM currency ORDER BY timestamp ASC");
	std::vector<CurrencyUpdate> result;
	while (query.next()) {
		if (query.lastError().isValid()) {
			QLOG_ERROR() << "Error getting currency:" << query.lastError().text();
			return {};
		};
		CurrencyUpdate update = CurrencyUpdate();
		update.timestamp = query.value(0).toLongLong();
		update.value = query.value(0).toByteArray().toStdString();
		result.push_back(update);
	};
	return result;
}

QString DataStore::MakeFilename(const std::string name, const std::string league) {
	std::string key = name + "|" + league;
	return QString(QCryptographicHash::hash(key.c_str(), QCryptographicHash::Md5).toHex());
}

void DataStore::SimpleQuery(DataStoreConnection& db, const QString& query_string) {
	const QString simplified_query = query_string.simplified();
	QMutexLocker locker(db.mutex);
	QSqlQuery query(simplified_query, db.database);
	if (query.isActive() == false) {
		QLOG_ERROR() << "Query failed:" << query.lastError().text() << ":" << simplified_query;
	};
}
