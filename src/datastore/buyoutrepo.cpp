// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#include "buyoutrepo.h"

#include <QSqlDatabase>
#include <QSqlError>
#include <QSqlQuery>

#include "currency.h"
#include "datastore/datastore_utils.h"
#include "item.h"
#include "itemlocation.h"

constexpr const char *CREATE_ITEM_BUYOUT_TABLE{R"(
CREATE TABLE IF NOT EXISTS item_buyouts (
    item_id         TEXT PRIMARY KEY,
    location_id     TEXT NOT NULL,
    location_type   TEXT NOT NULL CHECK (location_type IN ('character', 'stash')),
    currency        TEXT NOT NULL,
    inherited       INTEGER NOT NULL CHECK (inherited IN (0,1)),
    last_update     INTEGER NOT NULL,
    source          TEXT NOT NULL,
    type            TEXT NOT NULL,
    value           REAL NOT NULL
)
)"};

constexpr const char *CREATE_LOCATION_BUYOUT_TABLE{R"(
CREATE TABLE IF NOT EXISTS location_buyouts (
    location_id     TEXT PRIMARY KEY,
    location_type   TEXT NOT NULL CHECK (location_type IN ('character', 'stash')),
    currency        TEXT NOT NULL,
    inherited       INTEGER NOT NULL CHECK (inherited IN (0,1)),
    last_update     INTEGER NOT NULL,
    source          TEXT NOT NULL,
    type            TEXT NOT NULL,
    value           REAL NOT NULL
)
)"};

constexpr const char *UPSERT_ITEM_BUYOUT{R"(
INSERT INTO item_buyouts (
    item_id, location_id, location_type, currency, inherited, last_update, source, type, value
) VALUES (
    :item_id, :location_id, :location_type, :currency, :inherited, :last_update, :source, :type, :value
)
ON CONFLICT(item_id) DO UPDATE SET
    location_id     = excluded.location_id,
    location_type   = excluded.location_type,
    currency        = excluded.currency,
    inherited       = excluded.inherited,
    last_update     = excluded.last_update,
    source          = excluded.source,
    type            = excluded.type,
    value           = excluded.value
)"};

constexpr const char *UPSERT_LOCATION_BUYOUT{R"(
INSERT INTO location_buyouts (
    location_id, location_type, currency, inherited, last_update, source, type, value
) VALUES (
    :location_id, :location_type, :currency, :inherited, :last_update, :source, :type, :value
)
ON CONFLICT(location_id) DO UPDATE SET
    location_type   = excluded.location_type,
    currency        = excluded.currency,
    inherited       = excluded.inherited,
    last_update     = excluded.last_update,
    source          = excluded.source,
    type            = excluded.type,
    value           = excluded.value
)"};

BuyoutRepo::BuyoutRepo(QSqlDatabase &db)
    : m_db(db) {};

bool BuyoutRepo::resetRepo()
{
    QSqlQuery q(m_db);

    if (!q.exec("DROP TABLE IF EXISTS buyouts")) {
        ds::logQueryError("BuyoutRepo::reset", q);
        return false;
    }
    return ensureSchema();
}

bool BuyoutRepo::ensureSchema()
{
    constexpr std::array statements{
        CREATE_ITEM_BUYOUT_TABLE,
        CREATE_LOCATION_BUYOUT_TABLE,
    };

    QSqlQuery q(m_db);

    // Run setup statements.
    for (const auto &sql : statements) {
        if (!q.exec(sql)) {
            ds::logQueryError("BuyoutRepo::ensureSchema", q);
            return false;
        }
    }

    return true;
}

std::unordered_map<QString, Buyout> BuyoutRepo::getItemBuyouts()
{
    spdlog::debug("BuyoutRepo: getting item buyouts");

    QSqlQuery q(m_db);
    if (!q.prepare("SELECT"
                   "    item_id, location_id, location_type,"
                   "    currency, inherited, last_update, source, type, value"
                   " FROM item_buyouts")) {
        spdlog::error("BuyoutRepo: prepare() failed: {}", q.lastError().text());
        return {};
    }

    if (!q.exec()) {
        ds::logQueryError("BuyoutRepo::getItemBuyouts()", q);
        return {};
    }

    std::unordered_map<QString, Buyout> buyouts;

    while (q.next()) {
        const QString item_id = q.value(0).toString();
        Buyout buyout;
        buyout.currency = Currency::FromTag(q.value(3).toString());
        buyout.inherited = q.value(4).toBool();
        buyout.last_update = q.value(5).toDateTime();
        buyout.source = Buyout::TagAsBuyoutSource(q.value(6).toString());
        buyout.type = Buyout::TagAsBuyoutType(q.value(7).toString());
        buyout.value = q.value(8).toDouble();
        buyouts[item_id] = buyout;
    }
    return buyouts;
}

std::unordered_map<QString, Buyout> BuyoutRepo::getLocationBuyouts()
{
    spdlog::debug("BuyoutRepo: getting location buyouts");

    QSqlQuery q(m_db);
    if (!q.prepare("SELECT"
                   "    location_id, location_type,"
                   "    currency, inherited, last_update, source, type, value"
                   " FROM location_buyouts")) {
        spdlog::error("BuyoutRepo: prepare() failed: {}", q.lastError().text());
        return {};
    }

    if (!q.exec()) {
        ds::logQueryError("BuyoutRepo::getLocationBuyouts()", q);
        return {};
    }

    std::unordered_map<QString, Buyout> buyouts;

    while (q.next()) {
        const QString location_id = q.value(0).toString();
        Buyout buyout;
        buyout.currency = Currency::FromTag(q.value(2).toString());
        buyout.inherited = q.value(3).toBool();
        buyout.last_update = q.value(4).toDateTime();
        buyout.source = Buyout::TagAsBuyoutSource(q.value(5).toString());
        buyout.type = Buyout::TagAsBuyoutType(q.value(6).toString());
        buyout.value = q.value(7).toDouble();
        buyouts[location_id] = buyout;
    }
    return buyouts;
}

bool BuyoutRepo::saveItemBuyout(const Buyout &buyout, const Item &item)
{
    spdlog::debug("BuyoutRepo: saving item buyout: PrettyName='{}' ({}), buyout='{}'",
                  item.PrettyName(),
                  item.id(),
                  buyout.AsText());

    const ItemLocation &location = item.location();

    QString location_type;
    switch (location.get_type()) {
    case ItemLocationType::STASH:
        location_type = "stash";
        break;
    case ItemLocationType::CHARACTER:
        location_type = "character";
        break;
    default:
        spdlog::error("BuyoutRepo::saveItemBuyout: invalid item location type: {}",
                      location.get_type());
        return false;
    }

    QSqlQuery q(m_db);
    if (!q.prepare(UPSERT_ITEM_BUYOUT)) {
        spdlog::error("BuyoutRepo: prepare() failed: {}", q.lastError().text());
        return false;
    }

    q.bindValue(":item_id", item.id());
    q.bindValue(":location_id", location.get_tab_uniq_id());
    q.bindValue(":location_type", location_type);
    q.bindValue(":currency", buyout.CurrencyAsTag());
    q.bindValue(":inherited", buyout.inherited);
    q.bindValue(":last_update", buyout.last_update);
    q.bindValue(":source", buyout.BuyoutSourceAsTag());
    q.bindValue(":type", buyout.BuyoutTypeAsTag());
    q.bindValue(":value", buyout.value);

    if (!q.exec()) {
        ds::logQueryError("BuyoutRepo::saveItemBuyout()", q);
        return false;
    }
    return true;
}

bool BuyoutRepo::saveLocationBuyout(const Buyout &buyout, const ItemLocation &location)
{
    spdlog::debug("BuyoutRepo: saving location buyout: location='{}' ({}), buyout='{}'",
                  location.GetHeader(),
                  location.get_tab_uniq_id(),
                  buyout.AsText());

    QString location_type;
    switch (location.get_type()) {
    case ItemLocationType::STASH:
        location_type = "stash";
        break;
    case ItemLocationType::CHARACTER:
        location_type = "character";
        break;
    default:
        spdlog::error("BuyoutRepo::saveLocationBuyout: invalid item location type: {}",
                      location.get_type());
        return false;
    }

    QSqlQuery q(m_db);
    if (!q.prepare(UPSERT_LOCATION_BUYOUT)) {
        spdlog::error("BuyoutRepo: prepare() failed: {}", q.lastError().text());
        return false;
    }

    q.bindValue(":location_id", location.get_tab_uniq_id());
    q.bindValue(":location_type", location_type);
    q.bindValue(":currency", buyout.CurrencyAsTag());
    q.bindValue(":inherited", buyout.inherited);
    q.bindValue(":last_update", buyout.last_update);
    q.bindValue(":source", buyout.BuyoutSourceAsTag());
    q.bindValue(":type", buyout.BuyoutTypeAsTag());
    q.bindValue(":value", buyout.value);

    if (!q.exec()) {
        ds::logQueryError("BuyoutRepo::saveLocationBuyout()", q);
        return false;
    }
    return true;
}

void BuyoutRepo::removeItemBuyout(const Item &item)
{
    spdlog::debug("BuyoutRepo: removing item buyout: PrettyName='{}' ({})",
                  item.PrettyName(),
                  item.id());

    QSqlQuery q(m_db);
    if (!q.prepare("DELETE FROM item_buyouts WHERE item_id = :item_id")) {
        spdlog::error("BuyoutRepo: prepare() failed: {}", q.lastError().text());
        return;
    }

    q.bindValue(":item_id", item.id());

    if (!q.exec()) {
        ds::logQueryError("BuyoutRepo::removeItemBuyout()", q);
    }
}

void BuyoutRepo::removeLocationBuyout(const ItemLocation &location)
{
    spdlog::debug("BuyoutRepo: removing location buyout: '{}' ({})",
                  location.GetHeader(),
                  location.get_tab_uniq_id());

    QSqlQuery q(m_db);
    if (!q.prepare("DELETE FROM location_buyouts WHERE location_id = :location_id")) {
        spdlog::error("BuyoutRepo: prepare() failed: {}", q.lastError().text());
        return;
    }

    q.bindValue(":location_id", location.get_tab_uniq_id());

    if (!q.exec()) {
        ds::logQueryError("BuyoutRepo::removeLocationBuyout()", q);
    }
}
