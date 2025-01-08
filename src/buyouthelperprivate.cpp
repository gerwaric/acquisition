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

#include "buyouthelperprivate.h"

#include <QFile>
#include <QLocale>
#include <QMessageBox>
#include <QRegularExpression>
#include <QSqlDatabase>
#include <QSqlError>
#include <QSqlQuery>

#include <set>

#include "util/fatalerror.h"

constexpr const char* FILE_REQUEST_URL = "https://www.dropbox.com/request/cjbOVQUhS5JC1wrB0BtI";

void UserWarning(const QString& message) {
    auto result = QMessageBox::warning(nullptr, "Warning", message);
}

QString LegacyItem::effectiveTypeLine() const {
    QString result;
    // Acquisition uses the base typeline for vaal gems.
    if (hybrid) {
        if (hybrid->isVaalGem && (*hybrid->isVaalGem == true)) {
            result = typeLine;
        } else {
            result = hybrid->baseTypeName;
        };
    } else {
        result = typeLine;
    };
    // Remove legacy set information.
    const QRegularExpression re("^(<<.*?>>)*");
    result.replace(re, "");
    return result;
}

QString LegacyItem::hash() const {

    if (_character && _tab_label) {
        QLOG_ERROR() << "LegacyItem::hash() item contains both '_carhacter' and '_tab_label':" << name << id;
        return QString();
    };

    // This code is intended to exactly replicate the hash calulculated by leqacy acquisition.
    QString input = name + "~" + effectiveTypeLine() + "~";

    // Add explicit mods.
    if (explicitMods) {
        for (const auto& mod : *explicitMods) {
            input += mod + "~";
        };
    };

    // Add implicit mods.
    if (implicitMods) {
        for (const auto& mod : *implicitMods) {
            input += mod + "~";
        };
    };

    // Add properties.
    if (properties) {
        for (const auto& prop : *properties) {
            input += prop.name + "~";
            for (const auto& val : prop.values) {
                input += std::get<0>(val) + "~";
            };
        };
    };
    input += "~";

    // Add additional properties.
    if (additionalProperties) {
        for (const auto& prop : *additionalProperties) {
            input += prop.name + "~";
            for (const auto& val : prop.values) {
                input += std::get<0>(val) + "~";
            };
        };
    };
    input += "~";

    // Add sockets.
    if (sockets) {
        for (const auto& socket : *sockets) {
            if (socket.attr) {
                input += QString::number(socket.group) + "~" + *socket.attr + "~";
            };
        };
    };

    // Finish with the location tag.
    if (_character) {
        input += "~character:" + *_character;
    } else {
        input += "~stash:" + *_tab_label;
    };

    const QString result = QString(QCryptographicHash::hash(input.toUtf8(), QCryptographicHash::Md5).toHex());
    return result;
}

BuyoutHelperPrivate::BuyoutHelperPrivate(const QString& filename) {

    if (!QFile::exists(filename)) {
        QLOG_ERROR() << "BuyoutCollection: file not found:" << filename;
        return;
    };

    m_db = QSqlDatabase::addDatabase("QSQLITE", "Buyout Helper");
    m_db.setConnectOptions("QSQLITE_OPEN_READONLY");
    m_db.setDatabaseName(filename);
    if (!m_db.open()) {
        QLOG_ERROR() << "BuyoutCollection: cannot open" << filename << "due to error:" << m_db.lastError().text();
    };

    m_item_buyouts = getDatabaseStruct<LegacyBuyoutMap>("SELECT value FROM data WHERE (key = 'buyouts')");
    m_tab_buyouts = getDatabaseStruct<LegacyBuyoutMap>("SELECT value FROM data WHERE (key = 'tab_buyouts')");
    m_stash_tabs = getDatabaseStruct<LegacyStashList>("SELECT value FROM tabs WHERE (type = 0)");
    m_character_tabs = getDatabaseStruct<LegacyCharacterList>("SELECT value FROM tabs WHERE (type = 1)");
}

BuyoutHelperPrivate::~BuyoutHelperPrivate() {
    m_db.close();
    m_db.removeDatabase("Buyout Helper");
}

QByteArray BuyoutHelperPrivate::getDatabaseValue(const QString& query) {
    QSqlQuery q(m_db);
    q.setForwardOnly(true);
    q.prepare(query);
    if (!q.exec()) {
        QLOG_ERROR() << "BuyoutCollection: exec() failed for query:" << query << ":" << m_db.lastError().text();
        return {};
    };
    if (!q.next()) {
        QLOG_ERROR() << "BuyoutCollection: next() failed for query:" << query << ":" << m_db.lastError().text();
        return {};
    };
    return q.value(0).toByteArray();
}

bool BuyoutHelperPrivate::validate() {
    bool ok = true;
    validateTabBuyouts(ok);
    validateItemBuyouts(ok);
    return ok;
}

void BuyoutHelperPrivate::validateTabBuyouts(bool& ok) {

    QLocale locale = QLocale::system();

    QLOG_INFO() << "Validating tab buyouts:";
    QLOG_INFO() << "Found" << locale.toString(m_stash_tabs.size()) << "stash tabs";
    QLOG_INFO() << "Found" << locale.toString(m_character_tabs.size()) << "characters";
    QLOG_INFO() << "Found" << locale.toString(m_tab_buyouts.size()) << "tab buyouts";

    using LocationTag = QString;
    
    std::set<LocationTag> locations;
    std::set<LocationTag> duplicated_locations;

    std::set<LocationTag> duplicated_buyouts;
    std::set<LocationTag> ambiguous_buyouts;
    std::set<LocationTag> matched_buyouts;
    std::set<LocationTag> orphaned_buyouts;

    // Add stash tab location tags.
    for (const auto& location : m_stash_tabs) {
        const LocationTag tag = "stash:" + location.name;
        if (locations.count(tag) <= 0) {
            locations.insert(tag);
        } else {
            duplicated_locations.insert(tag);
        };
    }; 

    // Add character location tags.
    for (const auto& location : m_character_tabs) {
        const LocationTag tag = "character:" + location.name;
        if (locations.count(tag) <= 0) {
            locations.insert(tag);
        } else {
            duplicated_locations.insert(tag);
        };
    };

    // Validate all the tab buyouts.
    for (const auto& buyout : m_tab_buyouts) {
        const LocationTag& tag = buyout.first;
        if (matched_buyouts.count(tag) > 0) {
            duplicated_buyouts.insert(tag);
        } else if (locations.count(tag) > 0) {
            matched_buyouts.insert(tag);
        } else {
            orphaned_buyouts.insert(tag);
        };
        // If the location tag is one of the duplicated locations,
        // then we don't know which tab this buyout really belongs to.
        if (duplicated_locations.count(tag) > 0) {
            ambiguous_buyouts.insert(tag);
        };
    };

    if (duplicated_buyouts.size() > 0) {
        QLOG_WARN() << "Found" << locale.toString(duplicated_buyouts.size()) << "duplicated tab buyouts";
        ok = false;
    };

    if (ambiguous_buyouts.size() > 0) {
        QLOG_WARN() << "Found" << locale.toString(ambiguous_buyouts.size()) << "ambiguous tab buyouts";
        ok = false;
    };

    if (orphaned_buyouts.size() > 0) {
        QLOG_WARN() << "Found" << locale.toString(orphaned_buyouts.size()) << "orphaned buyouts";
        ok = false;
    };
}

void BuyoutHelperPrivate::validateItemBuyouts(bool& ok) {

    QLocale locale = QLocale::system();

    QLOG_INFO() << "Validating item buyouts";
    QLOG_INFO() << "Found" << locale.toString(m_item_buyouts.size()) << "item buyouts";

    using BuyoutHash = QString;
    
    std::set<BuyoutHash> buyouts;

    std::set<BuyoutHash> duplicated_buyouts;
    std::set<BuyoutHash> matched_buyouts;
    std::set<BuyoutHash> orphaned_buyouts;

    for (const auto& pair : m_item_buyouts) {
        const BuyoutHash& hash = pair.first;
        if (buyouts.count(hash) <= 0) {
            buyouts.insert(hash);
        } else {
            duplicated_buyouts.insert(hash);
        };
    };

    const QString statement = "SELECT (loc, value) FROM items";
    QSqlQuery query(m_db);
    query.setForwardOnly(true);
    query.prepare(statement);
    if (!query.exec()) {
        QLOG_ERROR() << "BuyoutHelperPrivate: exec() failed for query: '" + statement + "' :" << m_db.lastError().text();
    };

    int item_count = 0;

    while (query.next()) {
        const QString location = query.value(0).toString();
        const QByteArray value = query.value(0).toByteArray();
        std::vector<LegacyItem> items;
        JS::ParseContext context(value);
        if (context.parseTo(items) != JS::Error::NoError) {
            QLOG_ERROR() << "Error parsing items:" << QString::fromStdString(context.makeErrorString());
        };
        QLOG_DEBUG() << "Checking" << locale.toString(items.size()) << "items in location" << location;
        for (const auto& item : items) {
            const BuyoutHash hash = item.hash();
            if (matched_buyouts.count(hash) > 0) {
                duplicated_buyouts.insert(hash);
            } else if (buyouts.count(hash) > 0) {
                matched_buyouts.insert(hash);
            };
        };
        item_count += items.size();
    };

    // Check for errors
    if (query.lastError().isValid()) {
        QLOG_ERROR() << "BuyoutHelperPrivate: SQL error in results:" << query.lastError().text();
    };

    QLOG_INFO() << "Found" << locale.toString(item_count) << "items";

    // Now go back and make sure all of the buyouts have beem matched.
    for (const BuyoutHash& hash : buyouts) {
        if (matched_buyouts.count(hash) <= 0) {
            orphaned_buyouts.insert(hash);
        };
    };

    if (duplicated_buyouts.size() > 0) {
        QLOG_WARN() << "Found" << locale.toString(duplicated_buyouts.size()) << "duplicated item buyouts";
        ok = false;
    };

    if (orphaned_buyouts.size() > 0) {
        QLOG_WARN() << "Found" << locale.toString(orphaned_buyouts.size()) << "orphaned item buyouts";
        ok = false;
    };


}
