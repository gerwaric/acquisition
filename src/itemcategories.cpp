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

namespace {

	bool classes_initialized{ false };
	bool basetypes_initialized{ false };

	std::map<std::string, std::string> itemClassKeyToValue;
	std::map<std::string, std::string> itemClassValueToKey;
	std::map<std::string, std::string> itemBaseType_NameToClass;

	QStringList categories;

}

void InitItemClasses(const QByteArray& classes) {

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

	itemClassKeyToValue.clear();
	itemClassValueToKey.clear();

	QSet<QString> cats;
	for (auto itr = doc.MemberBegin(); itr != doc.MemberEnd(); ++itr) {
		std::string key = itr->name.GetString();
		std::string value = itr->value.FindMember("name")->value.GetString();
		if (value.empty()) {
			QLOG_DEBUG() << "Item class for" << key << "is empty";
			continue;
		};
		itemClassKeyToValue.insert(std::make_pair(key, value));
		itemClassValueToKey.insert(std::make_pair(value, key));
		cats.insert(QString::fromStdString(value));
	};
	categories = cats.values();
	categories.append(QString::fromStdString(CategorySearchFilter::k_Default));
	categories.sort();

	classes_initialized = true;
}

void InitItemBaseTypes(const QByteArray& baseTypes) {

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

	itemBaseType_NameToClass.clear();
	for (auto itr = doc.MemberBegin(); itr != doc.MemberEnd(); ++itr) {
		std::string item_class = itr->value.FindMember("item_class")->value.GetString();
		std::string name = itr->value.FindMember("name")->value.GetString();
		itemBaseType_NameToClass.insert(std::make_pair(name, item_class));
	};

	basetypes_initialized = true;
}

std::string GetItemCategory(const std::string& baseType) {

	auto rslt = itemBaseType_NameToClass.find(baseType);
	if (rslt != itemBaseType_NameToClass.end()) {
		std::string key = rslt->second;
		rslt = itemClassKeyToValue.find(key);
		if (rslt != itemClassKeyToValue.end()) {
			std::string category = rslt->second;
			boost::to_lower(category);
			return category;
		};
	};
	return "";
}

const QStringList& GetItemCategories() {
	return categories;
}
