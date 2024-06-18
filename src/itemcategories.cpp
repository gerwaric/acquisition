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

#include "itemcategories.h"

#include <QSet>
#include <QString>

#include <map>
#include <boost/algorithm/string.hpp>

#include "rapidjson/document.h"
#include "rapidjson/error/en.h"
#include "QsLog.h"

#include "filters.h"

class CATEGORY_DATA {
private:
	CATEGORY_DATA() {};
public:
	static CATEGORY_DATA& instance() {
		static CATEGORY_DATA data;
		return data;
	};
	std::map<std::string, std::string> itemClassKeyToValue;
	std::map<std::string, std::string> itemClassValueToKey;
	std::map<std::string, std::string> itemBaseType_NameToClass;
	QStringList categories;
};

void InitItemClasses(const QByteArray& classes) {

	static bool classes_initialized = false;

	rapidjson::Document doc;
	doc.Parse(classes.constData());
	if (doc.HasParseError()) {
		const auto error = doc.GetParseError();
		const auto reason = rapidjson::GetParseError_En(error);
		QLOG_ERROR() << "Error parsing RePoE item classes:" << reason;
		return;
	};

	QLOG_INFO() << "Loading item classes from RePoE.";
	if (classes_initialized) {
		QLOG_WARN() << "Item classes have already been loaded. They will be overwritten.";
	};

	static auto& data = CATEGORY_DATA::instance();
	data.itemClassKeyToValue.clear();
	data.itemClassValueToKey.clear();

	QSet<QString> cats;
	for (auto itr = doc.MemberBegin(); itr != doc.MemberEnd(); ++itr) {
		std::string key = itr->name.GetString();
		std::string value = itr->value.FindMember("name")->value.GetString();
		if (value.empty()) {
			QLOG_DEBUG() << "Item class for" << key << "is empty";
			continue;
		};
		data.itemClassKeyToValue.insert(std::make_pair(key, value));
		data.itemClassValueToKey.insert(std::make_pair(value, key));
		cats.insert(QString::fromStdString(value));
	};
	data.categories = cats.values();
	data.categories.append(QString::fromStdString(CategorySearchFilter::k_Default));
	data.categories.sort();

	classes_initialized = true;
}

void InitItemBaseTypes(const QByteArray& baseTypes) {

    static bool basetypes_initialized = false;

	rapidjson::Document doc;
	doc.Parse(baseTypes.constData());
	if (doc.HasParseError()) {
		const auto error = doc.GetParseError();
		const auto reason = rapidjson::GetParseError_En(error);
		QLOG_ERROR() << "Error parsing RePoE item base types:" << reason;
		return;
	};

	QLOG_INFO() << "Loading item base types from RePoE.";
	if (basetypes_initialized) {
		QLOG_WARN() << "Item base types have already been loaded. They will be overwritten.";
	};

	static auto& data = CATEGORY_DATA::instance();
	data.itemBaseType_NameToClass.clear();
	for (auto itr = doc.MemberBegin(); itr != doc.MemberEnd(); ++itr) {
		std::string item_class = itr->value.FindMember("item_class")->value.GetString();
		std::string name = itr->value.FindMember("name")->value.GetString();
		data.itemBaseType_NameToClass.insert(std::make_pair(name, item_class));
	};

	basetypes_initialized = true;
}

std::string GetItemCategory(const std::string& baseType) {

	static auto& data = CATEGORY_DATA::instance();

	auto rslt = data.itemBaseType_NameToClass.find(baseType);
	if (rslt != data.itemBaseType_NameToClass.end()) {
		std::string key = rslt->second;
		rslt = data.itemClassKeyToValue.find(key);
		if (rslt != data.itemClassKeyToValue.end()) {
			std::string category = rslt->second;
			boost::to_lower(category);
			return category;
		};
	};
	return "";
}

const QStringList& GetItemCategories() {
	static auto& data = CATEGORY_DATA::instance();
	return data.categories;
}
