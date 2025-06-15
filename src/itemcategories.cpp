/*
    Copyright (C) 2014-2025 Acquisition Contributors

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
#include <optional>

#include <util/json.h>
#include <util/spdlog_qt.h>
#include <util/util.h>

#include "filters.h"

struct ItemClassDTO {
    std::optional<QString> category;
    std::optional<QString> category_id;
    QString name;
    std::optional<std::vector<QString>> influence_tags;
};

struct BaseTypeDTO {
    QString item_class;
    QString name;
    QString release_state;
};

class CATEGORY_DATA {
private:
    CATEGORY_DATA() = default;
public:
    static CATEGORY_DATA& instance() {
        spdlog::trace("CATEGORY_DATA::instance() entered");
        static CATEGORY_DATA data;
        return data;
    };
    std::map<QString, QString> m_itemClassKeyToValue;
    std::map<QString, QString> m_itemClassValueToKey;
    std::map<QString, QString> m_itemBaseTypeToClass;
    QStringList categories;
};

void InitItemClasses(const QByteArray& json) {

    static bool classes_initialized = false;

    spdlog::debug("Initializing item classes");
    const auto item_classes = json::from_json_strict<std::map<QString,ItemClassDTO>>(json);

    if (item_classes.empty()) {
        spdlog::error("Error loading RePoE item classes");
        return;
    };

    if (classes_initialized) {
        spdlog::warn("Item classes have already been loaded. They will be overwritten.");
    };

    auto& data = CATEGORY_DATA::instance();
    data.m_itemClassKeyToValue.clear();
    data.m_itemClassValueToKey.clear();

    QSet<QString> cats;

    for (const auto& pair : item_classes) {

        const QString& key = pair.first;
        if (key.startsWith("DONOTUSE") || (0 == key.compare("Unarmed", Qt::CaseInsensitive))) {
            continue;
        };

        const ItemClassDTO& class_dto = pair.second;
        const QString& value = class_dto.name;
        if (value.isEmpty()) {
            spdlog::debug("Item class for {} is empty", key);
            continue;
        };

        data.m_itemClassKeyToValue[key] = value;
        data.m_itemClassValueToKey[value] = key;
        cats.insert(value);
    };

    data.categories = cats.values();
    data.categories.append(CategorySearchFilter::k_Default);
    data.categories.sort();

    classes_initialized = true;
}

void InitItemBaseTypes(const QByteArray& json) {

    static bool basetypes_initialized = false;
    
    spdlog::debug("Initializing item base types");

    // Turn off strictness for this read, since we aren't going to fully parse the data.
    const auto items = json::from_json<std::map<QString,BaseTypeDTO>>(json);

    if (items.empty()) {
        spdlog::error("Unable to load RePoE item base types");
        return;
    };

    if (basetypes_initialized) {
        spdlog::warn("Item base types have already been loaded. They will be overwritten.");
    };

    auto& data = CATEGORY_DATA::instance();
    data.m_itemBaseTypeToClass.clear();

    for (const auto& item : items) {

        const BaseTypeDTO& base_type_dto = item.second;
        if (0 == base_type_dto.release_state.compare("unreleased", Qt::CaseInsensitive)) {
            continue;
        };

        const QString& name = base_type_dto.name;
        if (name.isEmpty()
            || name.startsWith("[DO NOT USE]", Qt::CaseInsensitive)
            || name.startsWith("[UNUSED]", Qt::CaseInsensitive)
            || name.startsWith("[DNT]", Qt::CaseInsensitive)) {
            continue;
        };

        const QString& item_class = base_type_dto.item_class;
        data.m_itemBaseTypeToClass[name] = item_class;
    };
    basetypes_initialized = true;
}

QString GetItemCategory(const QString& baseType) {

    auto& data = CATEGORY_DATA::instance();

    if (data.m_itemClassKeyToValue.empty()) {
        spdlog::error("Item classes have not been initialized");
        return "";
    };

    if (data.m_itemBaseTypeToClass.empty()) {
        spdlog::error("Item base types have not been initialized");
        return "";
    };

    auto rslt = data.m_itemBaseTypeToClass.find(baseType);
    if (rslt != data.m_itemBaseTypeToClass.end()) {
        QString key = rslt->second;
        rslt = data.m_itemClassKeyToValue.find(key);
        if (rslt != data.m_itemClassKeyToValue.end()) {
            QString category = rslt->second.toLower();
            spdlog::trace("GetItemCategory: category is {}", category);
            return category;
        };
    };

    spdlog::trace("GetItemCategory: could not categorize baseType: {}", baseType);
    return "";
}

const QStringList& GetItemCategories() {
    spdlog::trace("GetItemCategories() entered");
    auto& data = CATEGORY_DATA::instance();
    if (data.categories.isEmpty()) {
        spdlog::error("Item categories have not been initialized");
    };
    return data.categories;
}
