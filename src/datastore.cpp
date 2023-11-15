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

Locations DataStore::DeserializeTabs(const QString& tabs_json) {

	if (tabs_json.isEmpty()) {
		QLOG_DEBUG() << "No tabs to deserialize.";
		return {};
	};

	rapidjson::Document doc;
	doc.Parse(tabs_json.toStdString().c_str());
	if (doc.HasParseError()) {
		QLOG_ERROR() << "Error parsing serialized tabs:" << rapidjson::GetParseError_En(doc.GetParseError());
		QLOG_ERROR() << "The malformed json is" << tabs_json;
		return {};
	};
	if (doc.IsArray() == false) {
		QLOG_ERROR() << "Error parsing serialized tabs: the json is not an array.";
		return {};
	};

	// Preallocate the return value.
	Locations tabs;
	tabs.reserve(doc.Size());

	// Keep track of which tabs have been parsed.
	std::set<std::string> tab_id_index_;

	for (auto& tab_json : doc) {
	
		// Detemine which kind of location this is.
		ItemLocationType type;
		if (tab_json.HasMember("name")) {
			type = ItemLocationType::CHARACTER;
		} else if (tab_json.HasMember("n")) {
			type = ItemLocationType::STASH;
		} else {
			QLOG_ERROR() << "Unable to determine location type during tab deserialization:" << Util::RapidjsonSerialize(tab_json);
			continue;
		};

		// Constructor values to fill in
		size_t index;
		std::string tabUniqueId, name;
		int r, g, b;

		switch (type) {
		case ItemLocationType::STASH:
			if (tab_id_index_.count(tab_json["id"].GetString())) {
				QLOG_ERROR() << "Duplicated tab found while loading data:" << tab_json["id"].GetString();
				continue;
			};
			if (!tab_json.HasMember("n") || !tab_json["n"].IsString()) {
				QLOG_ERROR() << "Malformed tabs data doesn't contain its name (field 'n'):" << Util::RapidjsonSerialize(tab_json);
				continue;
			};
			index = tab_json["i"].GetInt();
			tabUniqueId = tab_json["id"].GetString();
			name = tab_json["n"].GetString();
			r = tab_json["colour"]["r"].GetInt();
			g = tab_json["colour"]["g"].GetInt();
			b = tab_json["colour"]["b"].GetInt();
			break;
		case ItemLocationType::CHARACTER:
			if (tab_id_index_.count(tab_json["name"].GetString())) {
				continue;
			};
			if (tab_json.HasMember("i")) {
				index = tab_json["i"].GetInt();
			} else {
				index = tabs.size();
			};
			tabUniqueId = tab_json["name"].GetString();
			name = tab_json["name"].GetString();
			r = 0;
			g = 0;
			b = 0;
			break;
		};
		ItemLocation loc(static_cast<int>(index), tabUniqueId, name, type, r, g, b);
		loc.set_json(tab_json, doc.GetAllocator());
		tabs.push_back(loc);
		tab_id_index_.insert(loc.get_tab_uniq_id());
	};
	return tabs;
}

ItemLocation DataStore::DeserializeTab(const QString& tab_json) {

	if (tab_json.isEmpty()) {
		QLOG_DEBUG() << "No tab to deserialize.";
		return {};
	};

	rapidjson::Document doc;
	doc.Parse(tab_json.toStdString().c_str());
	if (doc.HasParseError()) {
		QLOG_ERROR() << "Error parsing serialized tabs:" << rapidjson::GetParseError_En(doc.GetParseError());
		QLOG_ERROR() << "The malformed json is" << tab_json;
		return {};
	};
	if (doc.IsObject() == false) {
		QLOG_ERROR() << "Error parsing serialized tab: the json is not an object:" << tab_json;
		return {};
	};

	// Detemine which kind of location this is.
	ItemLocationType type;
	if (doc.HasMember("name")) {
		type = ItemLocationType::CHARACTER;
	} else if (doc.HasMember("n")) {
		type = ItemLocationType::STASH;
	} else {
		QLOG_ERROR() << "Unable to determine location type during tab deserialization:" << tab_json;
		return {};
	};

	// Constructor values to fill in
	size_t index;
	std::string tabUniqueId, name;
	int r, g, b;

	switch (type) {
	case ItemLocationType::STASH:
		if (!doc.HasMember("n") || !doc["n"].IsString()) {
			QLOG_ERROR() << "Malformed tabs data doesn't contain its name (field 'n'):" << tab_json;
			return {};
		};
		index = doc["i"].GetInt();
		tabUniqueId = doc["id"].GetString();
		name = doc["n"].GetString();
		r = doc["colour"]["r"].GetInt();
		g = doc["colour"]["g"].GetInt();
		b = doc["colour"]["b"].GetInt();
		break;
	case ItemLocationType::CHARACTER:
		if (doc.HasMember("i")) {
			index = doc["i"].GetInt();
		} else {
			index = -1;
		};
		tabUniqueId = doc["name"].GetString();
		name = doc["name"].GetString();
		r = 0;
		g = 0;
		b = 0;
		break;
	};
	ItemLocation loc(static_cast<int>(index), tabUniqueId, name, type, r, g, b);
	loc.set_json(doc, doc.GetAllocator());
	return loc;
}

Items DataStore::DeserializeItems(const QString& items_json, const ItemLocation& tab) {

	// Parsed the serialized json and check for errors.
	rapidjson::Document doc;
	doc.Parse(items_json.toStdString().c_str());
	if (doc.HasParseError()) {
		QLOG_ERROR() << "Error parsing serialized items:" << rapidjson::GetParseError_En(doc.GetParseError());
		QLOG_ERROR() << "The malformed json is" << items_json;
		return {};
	};
	if (doc.IsArray() == false) {
		QLOG_ERROR() << "Error parsing serialized items: the json is not an array.";
		return {};
	};

	// Preallocate the return value.
	Items items;
	items.reserve(doc.Size());

	// Iterate over each item in the serialized json.
	for (auto item_json = doc.Begin(); item_json != doc.End(); ++item_json) {
		// Create a new location and make sure location-related information
		// such as x and y are pulled from the item json.
		ItemLocation loc = tab;
		loc.FromItemJson(*item_json);
		items.push_back(std::make_shared<Item>(*item_json, loc));
	};
	return items;
}
