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

#include "buyoutimporter.h"

#include <QFileInfo>
#include <QSqlDatabase>
#include <QSqlError>
#include <QSqlQuery>

#include <optional>
#include <string>
#include <unordered_map>

#include "boost/algorithm/string/join.hpp"
#include "json_struct/json_struct.h"

#include "QsLog.h"
#include "util.h"
#include "item.h"
#include "itemlocation.h"

struct ImportedStashTab {
	std::string n;
	std::string id;
	JS_OBJ(n, id);
};

struct ImportedStashTabArray {
	ImportedStashTabArray(const std::string& json);
	std::vector<ImportedStashTab> stashes;
	JS_OBJ(stashes);
};

struct ImportedCharacter {
	std::string name;
	JS_OBJ(name);
};

struct ImportedCharacterArray {
	ImportedCharacterArray(const std::string& json);
	std::vector<ImportedCharacter> characters;
	JS_OBJ(characters);
};

struct ImportedSocket {
	unsigned int group;
	std::optional<std::string> attr;
	std::optional<std::string> sColour;
	JS_OBJ(group, attr, sColour);
};

struct ImportedHybrid {
	std::optional<bool> isVaalGem;
	std::string baseTypeName;
	JS_OBJ(isVaalGem, baseTypeName);
};

struct ImportedProperty {
	std::string name;
	std::vector<std::tuple<std::string, unsigned int>> values;
	JS_OBJ(name, values);
};

struct ImportedItem {
	void Finish();
	const std::vector<std::string>& hashes() const { return hashes_; };
	const std::vector<std::string>& hash_inputs() const { return hash_inputs_; };
	std::string id;
	std::optional<std::vector<ImportedSocket>> sockets;
	std::string name;
	std::string typeLine;
	std::optional<std::vector<ImportedProperty>> properties;
	std::optional<std::vector<ImportedProperty>> additionalProperties;
	std::optional<std::vector<std::string>> implicitMods;
	std::optional<std::vector<std::string>> explicitMods;
	std::optional<ImportedHybrid> hybrid;
	int _type;
	std::optional<std::string> _tab_label;
	std::optional<std::string> _character;
	JS_OBJ(id, sockets, name, typeLine, properties, additionalProperties, implicitMods, explicitMods, hybrid, _type, _tab_label, _character);
private:
	void FixLegacyName(std::string& name);
	void PrepareHashes();
	std::string CommonHashString() const;
	bool finished{ false };
	std::vector<std::string> hashes_;
	std::vector<std::string> hash_inputs_;
};

struct ImportedItemArray {
	ImportedItemArray(const std::string& json);
	std::vector<ImportedItem> items;
	JS_OBJ(items);
};

struct ImportedBuyout {
	Buyout AsBuyout() const;
	double value;
	unsigned long long last_update;
	std::string type;
	std::string currency;
	std::string source;
	bool inherited;
	JS_OBJ(value, last_update, type, currency, source, inherited);
};

struct ImportedBuyoutArray {
	ImportedBuyoutArray(const std::string& json);
	std::unordered_map<std::string, ImportedBuyout> buyouts;
	JS_OBJ(buyouts);
};

QString GetStringData(const QString& name, QSqlDatabase& db);
std::map<std::string, ImportedBuyout> GetBuyouts(const QString& name, QSqlDatabase& db);
std::vector<ImportedStashTab> GetStashTabs(QSqlDatabase& db);
std::vector<ImportedCharacter> GetCharacters(QSqlDatabase& db);
std::vector<ImportedItem> GetItems(QSqlDatabase& db);


//==================================================================================

ImportedStashTabArray::ImportedStashTabArray(const std::string& json) {
	const std::string data = "{\"stashes\":" + json + "}";
	JS::ParseContext context(data);
	JS::Error error = context.parseTo(*this);
	if (error != JS::Error::NoError) {
		QLOG_ERROR() << "Error parsing stash tabs:" << context.makeErrorString();
	};
}

ImportedCharacterArray::ImportedCharacterArray(const std::string& json) {
	const std::string data = "{\"characters\":" + json + "}";
	JS::ParseContext context(data);
	JS::Error error = context.parseTo(*this);
	if (error != JS::Error::NoError) {
		QLOG_ERROR() << "Error parting characters:" << context.makeErrorString();
	};
}

ImportedItemArray::ImportedItemArray(const std::string& json) {
	const std::string data = "{\"items\":" + json + "}";
	JS::ParseContext context(data);
	JS::Error error = context.parseTo(*this);
	if (error != JS::Error::NoError) {
		QLOG_ERROR() << "Error parsing items:" << context.makeErrorString();
	};
	for (auto& item : items) {
		item.Finish();
	};
}

ImportedBuyoutArray::ImportedBuyoutArray(const std::string& json) {
	const std::string data = "{\"buyouts\":" + json + "}";
	JS::ParseContext context(data);
	JS::Error error = context.parseTo(*this);
	if (error != JS::Error::NoError) {
		QLOG_ERROR() << "Error parsing buyouts:" << context.makeErrorString();
	};
}

//----------------------------------------------------------------------

void ImportedItem::Finish()
{
	if (finished) {
		QLOG_ERROR() << "Imported item is already finished:" << name << typeLine;
		return;
	};

	// Fixup name fields the same way acquistion v0.9.7 and earlier did.
	FixLegacyName(name);
	FixLegacyName(typeLine);
	if (hybrid) {
		FixLegacyName(hybrid->baseTypeName);
	};

	// Make sure this tab looks like either a stash or a character.
	switch (ItemLocationType(_type)) {
	case ItemLocationType::STASH:
		if (!_tab_label) { QLOG_ERROR() << "Imported item: _type is STASH, but _tab_label is missing:" << name << typeLine; };
		if (_character) { QLOG_ERROR() << "Imported item: _type is STASH, but _character is present:" << name << typeLine; };
		break;
	case ItemLocationType::CHARACTER:
		if (!_character) { QLOG_ERROR() << "Imported item: _type is CHARACTER, but _character is missing:" << name << typeLine; };
		if (_tab_label) { QLOG_ERROR() << "Imported item: _type is CHARACTER, but _tab_label is present:" << name << typeLine; };
		break;
	default:
		QLOG_ERROR() << "Imported item: _type is invalid:" << _type << ":" << name << typeLine;
		break;
	};

	PrepareHashes();

	finished = true;
}

// Fix up names by removing all <<set:X>> modifiers
void ImportedItem::FixLegacyName(std::string& name) {
	std::string::size_type right_shift = name.rfind(">>");
	if (right_shift != std::string::npos) {
		name = name.substr(right_shift + 2);
	};
}

void ImportedItem::PrepareHashes() {

	// Build a list of potential names.
	std::list<std::string> possible_names;
	possible_names.push_back(name);
	possible_names.push_back("<<set:MS>><<set:M>><<set:S>>" + name);

	// Build a list of potential typelines.
	std::list<std::string> possible_typelines;
	possible_typelines.push_back(typeLine);
	if (hybrid) {
		possible_typelines.push_back(hybrid->baseTypeName);
	};

	hashes_.clear();
	hash_inputs_.clear();

	const std::string common(CommonHashString());
	for (auto& possible_name : possible_names) {
		for (auto& possible_typeline : possible_typelines) {
			const auto input = possible_name + "~" + possible_typeline + "~" + common;
			hashes_.push_back(Util::Md5(input));
			hash_inputs_.push_back(input);
		};
	};
}

std::string ImportedItem::CommonHashString() const {

	// These items will be joined with "~" and hashed with md5.
	std::list<std::string> parts;

	// Add mods.
	for (auto mods : { explicitMods, implicitMods }) {
		if (mods) {
			for (auto& mod : *mods) {
				parts.push_back(mod);
			};
		};
	};

	// Add properties.
	for (auto properties : { properties, additionalProperties }) {
		if (properties) {
			for (auto& prop : *properties) {
				parts.push_back(prop.name);
				for (auto& value : prop.values) {
					parts.push_back(std::get<0>(value));
				};
			};
		};
		parts.push_back("");
	};

	// Add sockets.
	if (sockets) {
		for (auto& socket : *sockets) {
			if (socket.attr) {
				parts.push_back(std::to_string(socket.group));
				parts.push_back(*socket.attr);
			};
		};
	};

	// Add location identifier.
	switch (ItemLocationType(_type)) {
	case ItemLocationType::STASH:
		parts.push_back("~stash:" + *_tab_label);
		break;
	case ItemLocationType::CHARACTER:
		parts.push_back("~character:" + *_character);
		break;
	default:
		QLOG_ERROR() << "Legacy item has invalid _type:" << _type;
		break;
	};

	return boost::algorithm::join(parts, "~");
}

Buyout ImportedBuyout::AsBuyout() const {
	Buyout buyout;
	buyout.value = value;
	buyout.type = Buyout::TagAsBuyoutType(type);
	buyout.source = Buyout::TagAsBuyoutSource(source);
	buyout.currency = Currency::FromTag(currency);
	buyout.last_update = QDateTime::fromSecsSinceEpoch(last_update);
	return buyout;
}

//=========================================================================

QString GetStringData(const QString& name, QSqlDatabase& db)
{
	QSqlQuery query(db);
	query.prepare("SELECT value FROM data WHERE key = ?");
	query.bindValue(0, name);
	query.exec();
	if (!query.isActive() || !query.next()) {
		QLOG_ERROR() << "Buyout importer cannot GET" << name << ":" << query.lastError().text();
		return "";
	};
	return query.value(0).toString();
}

std::map<std::string, ImportedBuyout> GetBuyouts(const QString& name, QSqlDatabase& db)
{
	QString data = GetStringData(name, db);
	if (data == "") {
		return {};
	};

	std::map<std::string, ImportedBuyout> buyouts;

	ImportedBuyoutArray imported(data.toStdString());
	for (auto& item : imported.buyouts) {
		const auto& hash = item.first;
		const auto& imported_buyout = item.second;
		buyouts[hash] = imported_buyout;
	};

	return buyouts;
}

std::vector<ImportedStashTab> GetStashTabs(QSqlDatabase& db) {

	QSqlQuery query(db);
	query.prepare("SELECT value FROM tabs WHERE type = ?");
	query.bindValue(0, static_cast<int>(ItemLocationType::STASH));
	if (query.exec() == false) {
		QLOG_ERROR() << "Buyout importer cannot get tabs:" << query.lastError().text();
		return {};
	};
	if (!query.next()) {
		QLOG_ERROR() << "Buyout importer cannot get tabs:" << query.lastError().text();
		return {};
	};
	const QString json_array = query.value(0).toString();
	const QString json = query.value(0).toString();
	ImportedStashTabArray result(json.toStdString());
	return result.stashes;
}

std::vector<ImportedCharacter> GetCharacters(QSqlDatabase& db) {

	QSqlQuery query(db);
	query.prepare("SELECT value FROM tabs WHERE type = ?");
	query.bindValue(0, static_cast<int>(ItemLocationType::CHARACTER));
	if (query.exec() == false) {
		QLOG_ERROR() << "Buyout importer cannot get characters:" << query.lastError().text();
		return {};
	};
	if (!query.next()) {
		QLOG_ERROR() << "Buyout importer cannot get characters:" << query.lastError().text();
		return {};
	};
	ImportedCharacterArray result(query.value(0).toString().toStdString());
	return result.characters;
}

std::vector<ImportedItem> GetItems(QSqlDatabase& db) {
	QSqlQuery query("SELECT loc, value FROM items", db);
	if (!query.isActive()) {
		QLOG_ERROR() << "Buyout import error getting all items:" << query.lastError().text();
		return {};
	};
	std::list<ImportedItem> items;
	while (query.next()) {
		const QString loc = query.value(0).toString();
		const QString value = query.value(1).toString();
		ImportedItemArray imported(value.toStdString());
		for (auto& item : imported.items) {
			items.push_back(item);
		};
	};
	return { items.begin(), items.end() };
}

//==================================================================

BuyoutImporter::BuyoutImporter(const QString& filename) :
	filename_(filename)
{
	QFileInfo info(filename);
	if (!info.exists()) {
		QLOG_ERROR() << "File missing:" << filename;
		return;
	};
	if (!info.isFile()) {
		QLOG_ERROR() << "File is not a file:" << filename;
		return;
	};

	QSqlDatabase db = QSqlDatabase::addDatabase("QSQLITE", "datastore_import");
	db.setDatabaseName(filename_);
	if (db.open() == false) {
		QLOG_ERROR() << "Buyout import failed: error opening database:" << db.lastError().text();
		return;
	};

	version_ = GetStringData("version", db).toStdString();
	db_version_ = GetStringData("db_version", db).toStdString();
	ImportTabBuyouts(db);
	ImportItemBuyouts(db);
	return;
}

void BuyoutImporter::ImportTabBuyouts(QSqlDatabase& db) {

	stash_buyouts_.clear();

	// Maps tab_buyouts to stash unique ids.
	std::map<std::string, std::string> stash_map;
	const auto stashes = GetStashTabs(db);
	for (const auto& stash : stashes) {
		stash_map["stash:" + stash.n] = stash.id;
	};

	// Maps tab_buyouts to character names.
	std::map<std::string, std::string> character_map;
	const auto characters = GetCharacters(db);
	for (const auto& character : characters) {
		character_map["character:" + character.name] = character.name;
	};

	const auto tab_buyouts = GetBuyouts("tab_buyouts", db);

	for (auto& item : tab_buyouts) {
		const std::string& tag = item.first;
		const ImportedBuyout& buyout = item.second;
		if (stash_map.count(tag) > 0) {
			// Map a tab's unique id to this buyout.
			stash_buyouts_[stash_map[tag]] = buyout.AsBuyout();
		} else if (character_map.count(tag) > 0) {
			// Map a character's name to this buyout.
			character_buyouts_[character_map[tag]] = buyout.AsBuyout();
		} else {
			QLOG_ERROR() << "Orphaned tab buyout:" << tag;
		};
	};
}

void BuyoutImporter::ImportItemBuyouts(QSqlDatabase& db) {

	item_buyouts_.clear();

	const auto items = GetItems(db);

	// Maps buyout hash to item unique id.
	std::map<std::string, std::string> item_map;
	for (const auto& item : items) {
		for (const auto& hash : item.hashes()) {
			item_map[hash] = item.id;
		};
	};

	const auto item_buyouts = GetBuyouts("buyouts", db);

	for (auto& item : item_buyouts) {
		const std::string& hash = item.first;
		const ImportedBuyout& buyout = item.second;
		if (item_map.count(hash) == 0) {
			QLOG_ERROR() << "Item buyout missing:" << hash;
			continue;
		};
		item_buyouts_[item_map[hash]] = buyout.AsBuyout();
	};
}