/*
    Copyright (C) 2014-2024 Acquisition Contributors

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

#include <QByteArray>
#include <QString>

#include <map>

#include <QsLog/QsLog.h>
#include <rapidjson/document.h>
#include <rapidjson/error/en.h>

#include "filters.h"

class CATEGORY_DATA {
private:
    CATEGORY_DATA() {};
public:
    static CATEGORY_DATA& instance() {
        QLOG_TRACE() << "CATEGORY_DATA::instance() entered";
        static CATEGORY_DATA data;
        return data;
    };
    std::map<QString, QString> itemClassKeyToValue;
    std::map<QString, QString> itemClassValueToKey;
    std::map<QString, QString> itemBaseType_NameToClass;
    QStringList categories;
};

void InitItemClasses(const QByteArray& classes) {
    QLOG_TRACE() << "InitItemClasses() entered";
    static bool classes_initialized = false;
    QLOG_TRACE() << "InitItemClasses() classes_initialized =" << classes_initialized;

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

    QLOG_TRACE() << "InitItemClasses() processing data";
    QSet<QString> cats;
    for (auto itr = doc.MemberBegin(); itr != doc.MemberEnd(); ++itr) {
        QString key = itr->name.GetString();
        QString value = itr->value.FindMember("name")->value.GetString();
        if (value.isEmpty()) {
            QLOG_DEBUG() << "Item class for" << key << "is empty";
            continue;
        };
        data.itemClassKeyToValue.insert(std::make_pair(key, value));
        data.itemClassValueToKey.insert(std::make_pair(value, key));
        cats.insert(value);
    };
    data.categories = cats.values();
    data.categories.append(CategorySearchFilter::k_Default);
    data.categories.sort();

    classes_initialized = true;
}

void InitItemBaseTypes(const QByteArray& baseTypes) {
    QLOG_TRACE() << "InitItemBaseTypes() entered";
    static bool basetypes_initialized = false;
    QLOG_TRACE() << "InitItemBaseTypes() basetypes_initialized =" << basetypes_initialized;

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

    QLOG_TRACE() << "InitItemBaseTypes() processing data";
    static auto& data = CATEGORY_DATA::instance();
    data.itemBaseType_NameToClass.clear();
    for (auto itr = doc.MemberBegin(); itr != doc.MemberEnd(); ++itr) {
        QString item_class = itr->value.FindMember("item_class")->value.GetString();
        QString name = itr->value.FindMember("name")->value.GetString();
        data.itemBaseType_NameToClass.insert(std::make_pair(name, item_class));
    };

    basetypes_initialized = true;
}

QString GetItemCategory(const QString& baseType) {

    static auto& data = CATEGORY_DATA::instance();

    if (data.itemBaseType_NameToClass.empty()) {
        QLOG_ERROR() << "Item base types have not been initialized";
        return "";
    };

    auto rslt = data.itemBaseType_NameToClass.find(baseType);
    if (rslt != data.itemBaseType_NameToClass.end()) {
        QString key = rslt->second;
        rslt = data.itemClassKeyToValue.find(key);
        if (rslt != data.itemClassKeyToValue.end()) {
            QString category = rslt->second.toLower();
            QLOG_TRACE() << "GetItemCategory: category is" << category;
            return category;
        };
    };

    QLOG_TRACE() << "GetItemCategory: could not categorize baseType:" << baseType;
    return "";
}

const QStringList& GetItemCategories() {
    QLOG_TRACE() << "GetItemCategories() entered";
    static auto& data = CATEGORY_DATA::instance();
    if (data.categories.isEmpty()) {
        QLOG_ERROR() << "Item categories have not been initialized";
    };
    return data.categories;
}
