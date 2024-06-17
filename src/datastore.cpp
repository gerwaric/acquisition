#include "datastore.h"

#include "QsLog.h"
#include "rapidjson/error/en.h"
#include "rapidjson/rapidjson.h"
#include "rapidjson_util.h"
#include "util.h"

void DataStore::SetBool(const std::string& key, bool value) {
	SetInt(key, static_cast<int>(value));
}

bool DataStore::GetBool(const std::string& key, bool default_value) {
	return static_cast<bool>(GetInt(key, static_cast<int>(default_value)));
}

void DataStore::SetInt(const std::string& key, int value) {
	Set(key, std::to_string(value));
}

int DataStore::GetInt(const std::string& key, int default_value) {
	return std::stoi(Get(key, std::to_string(default_value)));
}

QString DataStore::Serialize(const Locations& tabs) {
	QStringList json;
	json.reserve(tabs.size());
	for (auto& tab : tabs) {
		json.push_back(QString::fromStdString(tab.get_json()));
	};
	return "[" + json.join(",") + "]";
}

QString DataStore::Serialize(const Items& items) {
	QStringList json;
	json.reserve(items.size());
	for (auto& item : items) {
		json.push_back(QString::fromStdString(item->json()));
	};
	return "[" + json.join(",") + "]";
}

Locations DataStore::DeserializeTabs(const QString& json) {

	if (json.isEmpty()) {
		QLOG_DEBUG() << "No tabs to deserialize.";
		return {};
	};

	rapidjson::Document doc;
	doc.Parse(json.toStdString().c_str());
	if (doc.HasParseError()) {
		QLOG_ERROR() << "Error parsing serialized tabs:" << rapidjson::GetParseError_En(doc.GetParseError());
		QLOG_ERROR() << "The malformed json is" << json;
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
		ItemLocationType type = (tab_json.HasMember("class"))
			? ItemLocationType::CHARACTER
			: ItemLocationType::STASH;

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
		ItemLocation loc(static_cast<int>(index), tabUniqueId, name, type, r, g, b, tab_json, doc.GetAllocator());
		tabs.push_back(loc);
		tab_id_index_.insert(loc.get_tab_uniq_id());
	};
	return tabs;
}

Items DataStore::DeserializeItems(const QString& json, const ItemLocation& tab) {

	// Parsed the serialized json and check for errors.
	rapidjson::Document doc;
	doc.Parse(json.toStdString().c_str());
	if (doc.HasParseError()) {
		QLOG_ERROR() << "Error parsing serialized items:" << rapidjson::GetParseError_En(doc.GetParseError());
		QLOG_ERROR() << "The malformed json is" << json;
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