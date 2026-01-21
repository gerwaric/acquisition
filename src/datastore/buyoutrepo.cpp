// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#include "buyoutrepo.h"

#include <QSqlDatabase>
#include <QSqlQuery>

#include "datastore/datastore_utils.h"
#include "item.h"
#include "itemlocation.h"

constexpr const char *CREATE_BUYOUT_TABLE{R"(
CREATE TABLE IF NOT EXISTS buyouts (
    buyout_id       TEXT PRIMARY KEY,
    buyout_type     TEXT NOT NULL CHECK (buyout_tyoe IN ('item', 'stash')),
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
    constexpr std::array statements{CREATE_BUYOUT_TABLE};

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

bool BuyoutRepo::saveItemBuyout(const Buyout &buyout, const Item &item)
{
    return true;
}

bool BuyoutRepo::saveLocationBuyout(const Buyout &buyout, const ItemLocation &location)
{
    return true;
}
