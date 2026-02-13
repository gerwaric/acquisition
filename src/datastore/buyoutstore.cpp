// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#include "datastore/buyoutstore.h"

#include <QSqlDatabase>
#include <QSqlError>
#include <QSqlQuery>

#include "currency.h"
#include "datastore/datastore_utils.h"
#include "item.h"
#include "itemlocation.h"

constexpr const char *CREATE_BUYOUT_TABLE{R"(
CREATE TABLE IF NOT EXISTS buyouts (
    buyout_type     TEXT NOT NULL CHECK (buyout_type IN ('item', 'location')),
    item_id         TEXT NOT NULL,
    location_id     TEXT NOT NULL,
    location_type   TEXT NOT NULL CHECK (location_type IN ('character', 'stash')),
    currency        TEXT NOT NULL,
    inherited       INTEGER NOT NULL CHECK (inherited IN (0,1)),
    last_update     INTEGER NOT NULL,
    source          TEXT NOT NULL,
    type            TEXT NOT NULL,
    value           REAL NOT NULL,
    PRIMARY KEY (buyout_type, item_id, location_id)
);
)"};

constexpr const char *UPSERT_BUYOUT{R"(
INSERT INTO buyouts (
    buyout_type, item_id, location_id, location_type, currency, inherited, last_update, source, type, value
) VALUES (
    :buyout_type, :item_id, :location_id, :location_type, :currency, :inherited, :last_update, :source, :type, :value
)
ON CONFLICT(buyout_type, item_id, location_id) DO UPDATE SET
    location_type   = excluded.location_type,
    currency        = excluded.currency,
    inherited       = excluded.inherited,
    last_update     = excluded.last_update,
    source          = excluded.source,
    type            = excluded.type,
    value           = excluded.value;
)"};

constexpr const char *SELECT_BUYOUTS{R"(
SELECT item_id, location_id, currency, inherited, last_update, source, type, value
FROM buyouts WHERE buyout_type = :buyout_type;
)"};

constexpr const char *DELETE_BUYOUT{R"(
DELETE FROM buyouts
WHERE buyout_type = :buyout_type AND item_id = :item_id AND location_id = :location_id;
)"};

BuyoutStore::BuyoutStore(QStringView connName)
    : m_connName(connName) {};

bool BuyoutStore::resetRepo()
{
    QSqlDatabase db = QSqlDatabase::database(m_connName);
    QSqlQuery q(db);

    if (!q.exec("DROP TABLE IF EXISTS buyouts;")) {
        ds::logQueryError("BuyoutStore::resetRepo:exec", q);
        return false;
    }
    return ensureSchema();
}

bool BuyoutStore::ensureSchema()
{
    QSqlDatabase db = QSqlDatabase::database(m_connName);
    QSqlQuery q(db);

    if (!q.exec(CREATE_BUYOUT_TABLE)) {
        ds::logQueryError("BuyoutStore::ensureSchema:exec", q);
        return false;
    }
    return true;
}

std::unordered_map<QString, Buyout> BuyoutStore::getItemBuyouts()
{
    return getBuyouts("item");
}

std::unordered_map<QString, Buyout> BuyoutStore::getLocationBuyouts()
{
    return getBuyouts("location");
}

std::unordered_map<QString, Buyout> BuyoutStore::getBuyouts(const QString &buyout_type)
{
    spdlog::debug("BuyoutStore: getting '{}' buyouts", buyout_type);

    QSqlDatabase db = QSqlDatabase::database(m_connName);
    QSqlQuery q(db);
    if (!q.prepare(SELECT_BUYOUTS)) {
        ds::logQueryError("BuyoutStore::getBuyouts:prepare", q);
        return {};
    }

    q.bindValue(":buyout_type", buyout_type);

    if (!q.exec()) {
        ds::logQueryError("BuyoutStore::getBuyouts:exec", q);
        return {};
    }

    std::unordered_map<QString, Buyout> buyouts;

    const int id_index = (buyout_type == "item") ? 0 : 1;

    while (q.next()) {
        Buyout buyout;
        buyout.currency = Currency::FromTag(q.value(2).toString());
        buyout.inherited = q.value(3).toBool();
        buyout.last_update = q.value(4).toDateTime();
        buyout.source = Buyout::TagAsBuyoutSource(q.value(5).toString());
        buyout.type = Buyout::TagAsBuyoutType(q.value(6).toString());
        buyout.value = q.value(7).toDouble();

        const QString id = q.value(id_index).toString();
        buyouts[id] = buyout;
    }
    return buyouts;
}

bool BuyoutStore::saveItemBuyout(const Buyout &buyout, const Item &item)
{
    return saveBuyout(buyout, item.location(), item);
}

bool BuyoutStore::saveLocationBuyout(const Buyout &buyout, const ItemLocation &location)
{
    return saveBuyout(buyout, location, std::nullopt);
}

bool BuyoutStore::saveBuyout(const Buyout &buyout,
                             const ItemLocation &location,
                             const std::optional<Item> item)
{
    const QString buyout_type = item ? "item" : "location";
    const QString item_id = item ? item->id() : "";
    const QString item_name = item ? item->PrettyName() : "";

    spdlog::debug(
        "BuyoutStore: saving '{}' buyout: item='{}' ({}), location='{}' ({}), buyout='{}'",
        buyout_type,
        item_name,
        item_id,
        location.GetHeader(),
        location.id(),
        buyout.AsText());

    QSqlDatabase db = QSqlDatabase::database(m_connName);
    QSqlQuery q(db);
    if (!q.prepare(UPSERT_BUYOUT)) {
        ds::logQueryError("BuyoutRep::saveBuyout:prepare", q);
        return false;
    }

    q.bindValue(":buyout_type", buyout_type);
    q.bindValue(":item_id", item_id);
    q.bindValue(":location_id", location.id());
    q.bindValue(":location_type", location.typeAsString());
    q.bindValue(":currency", buyout.CurrencyAsTag());
    q.bindValue(":inherited", buyout.inherited);
    q.bindValue(":last_update", buyout.last_update);
    q.bindValue(":source", buyout.BuyoutSourceAsTag());
    q.bindValue(":type", buyout.BuyoutTypeAsTag());
    q.bindValue(":value", buyout.value);

    if (!q.exec()) {
        ds::logQueryError("BuyoutStore::saveBuyout:exec", q);
        return false;
    }
    return true;
}

void BuyoutStore::removeItemBuyout(const Item &item)
{
    removeBuyout(item.location(), item);
}

void BuyoutStore::removeLocationBuyout(const ItemLocation &location)
{
    removeBuyout(location, std::nullopt);
}

void BuyoutStore::removeBuyout(const ItemLocation &location, const std::optional<Item> item)
{
    const QString buyout_type = item ? "item" : "location";
    const QString item_id = item ? item->id() : "";
    const QString item_name = item ? item->PrettyName() : "";

    spdlog::debug("BuyoutStore: removing '{}' buyout: item='{}' ({}), location='{}' ({})",
                  buyout_type,
                  item_name,
                  item_id,
                  location.GetHeader(),
                  location.id());

    QSqlDatabase db = QSqlDatabase::database(m_connName);
    QSqlQuery q(db);
    if (!q.prepare(DELETE_BUYOUT)) {
        ds::logQueryError("BuyoutStore::removeBuyout:prepare", q);
        return;
    }

    q.bindValue(":buyout_type", buyout_type);
    q.bindValue(":item_id", item_id);
    q.bindValue(":location_id", location.id());

    if (!q.exec()) {
        ds::logQueryError("BuyoutStore::removeBuyout:exec", q);
    }
}
