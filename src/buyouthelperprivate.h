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

#pragma once

#include <QString>
#include <QSqlDatabase>

#include <optional>
#include <unordered_map>
#include <unordered_set>
#include <vector>

#include <json_struct/json_struct_qt.h>
#include <QsLog/QsLog.h>

void UserWarning(const QString& message);

// I'm using privately defined structures and classes here so this code is fully
// independent of the rest of acquisition. It's intended use is eventually to
// import "legacy" buyouts from the current database version (v0.12) into a
// new version after a ground-up re-write with an incompatible database.

enum class LegacyLocationType : int {
    STASH = 0,
    CHARACTER = 1
};

struct LegacyItem {

    struct Socket {
        unsigned group;
        std::optional<QString> attr;
        JS_OBJ(group, attr);
    };

    struct Property {
        QString name;
        std::vector<std::tuple<QString, int>> values;
        JS_OBJ(name, values);
    };

    struct HybridInfo {
        std::optional<bool> isVaalGem;
        QString baseTypeName;
        JS_OBJ(isVaalGem, baseTypeName);
    };

    // This is just enough of the item to calculate the legacy buyout hash.

    QString id;
    std::optional<std::vector<LegacyItem::Socket>> sockets;
    QString name;
    QString typeLine;
    std::optional<std::vector<LegacyItem::Property>> properties;
    std::optional<std::vector<LegacyItem::Property>> additionalProperties;
    std::optional<std::vector<QString>> implicitMods;
    std::optional<std::vector<QString>> explicitMods;
    std::optional<LegacyItem::HybridInfo> hybrid;
    std::optional<QString> _character;
    std::optional<QString> _tab_label;
    JS_OBJ(name, typeLine, hybrid, implicitMods, explicitMods, properties, additionalProperties, sockets, _character, _tab_label);

    // Duplicate the way acquisition handles typeLine for vaal gems.
    QString effectiveTypeLine() const;

    // Duplicate acquisition's old item hashing function.
    QString hash() const;
};

struct LegacyCharacterLocation {
    QString id;
    QString name;
    QString realm;
    QString class_;
    QString league;
    unsigned int level;
    unsigned long experience;
    std::optional<bool> current;
    JS_OBJECT(
        JS_MEMBER(id),
        JS_MEMBER(name),
        JS_MEMBER(realm),
        JS_MEMBER_WITH_NAME(class_, "class"),
        JS_MEMBER(league),
        JS_MEMBER(level),
        JS_MEMBER(experience),
        JS_MEMBER(current));
};

struct LegacyStashLocation {

    struct MapData {
        int series;
        JS_OBJ(series);
    };

    struct Metadata {
        std::optional<bool> public_;
        QString colour;
        std::optional<LegacyStashLocation::MapData> map;
        JS_OBJECT(
            JS_MEMBER_WITH_NAME(public_, "public"),
            JS_MEMBER(colour),
            JS_MEMBER(map));
    };

    struct Colour {
        int r;
        int g;
        int b;
        JS_OBJ(r, g, b);
    };

    QString id;
    QString name;
    QString type;
    int index;
    LegacyStashLocation::Metadata metadata;
    int i;
    QString n;
    LegacyStashLocation::Colour colour;
    JS_OBJ(id, name, type, index, metadata, i, n, colour);
};

struct LegacyBuyout {
    double value;
    long long last_update;
    QString type;
    QString currency;
    QString source;
    bool inherited;
    JS_OBJ(value, last_update, type, currency, source, inherited);
};

struct LegacyCurrency {
    int count;
    double chaos_ratio;
    double exalt_ratio;
    QString currency;
    JS_OBJ(count, chaos_ratio, exalt_ratio, currency);
};

using LegacyBuyoutMap = std::unordered_map<QString, LegacyBuyout>;
using LegacyCurrencyMap = std::unordered_map<QString, LegacyCurrency>;
using LegacyStashList = std::vector<LegacyStashLocation>;
using LegacyCharacterList = std::vector<LegacyCharacterLocation>;

/*
struct LegacyDataStore {
    struct DataTable {
        QString db_version;
        QString version;
        QString currency_last_value;
        LegacyBuyoutMap buyouts;
        LegacyBuyoutMap tab_buyouts;
        std::unordered_set<QString> refresh_checked_state;
        LegacyCurrencyMap currency_items;
        JS_OBJ(db_version, version, currency_last_value, buyouts, tab_buyouts, refresh_checked_state);
    };
    using CurrencyTable = std::unordered_map<unsigned long long, QString>;
    struct TabsTable {
        LegacyStashList stashes;
        LegacyCharacterList characters;
    };
    using ItemsTable = std::unordered_map<QString, std::vector<LegacyItem>>;
    DataTable data;
    CurrencyTable currency;
    TabsTable tabs;
    ItemsTable items;
    JS_OBJ(data, currency, tabs, items);
};
*/

class BuyoutHelperPrivate {
public:
    BuyoutHelperPrivate(const QString& filename);
    ~BuyoutHelperPrivate();
    const LegacyBuyoutMap& tab_buyouts() const { return m_tab_buyouts; };
    const LegacyBuyoutMap& item_buyouts() const { return m_item_buyouts; };
    const LegacyCharacterList& character_tabs() const { return m_character_tabs; };
    const LegacyStashList& stash_tabs() const { return m_stash_tabs; };
    bool validate();

private:

    void validateTabBuyouts(bool& ok);
    void validateItemBuyouts(bool& ok);

    QSqlDatabase m_db;
    LegacyBuyoutMap m_tab_buyouts;
    LegacyBuyoutMap m_item_buyouts;
    LegacyStashList m_stash_tabs;
    LegacyCharacterList m_character_tabs;

    QByteArray getDatabaseValue(const QString& query);

    template<typename T>
    T getDatabaseStruct(const QString& query) {
        T result;
        const QByteArray data = getDatabaseValue(query);
        JS::ParseContext context(data);
        context.allow_missing_members = false;
        context.allow_unasigned_required_members = false;
        if (context.parseTo(result) != JS::Error::NoError) {
            QLOG_ERROR()
                << "BuyoutHelperPrivate::getDatabaseStruct<" + QString::fromStdString(typeid(T).name()) + ">('" + query + "')"
                << "json_struct parse error: " << QString::fromUtf8(context.makeErrorString());
        };
        return result;
    };

};

