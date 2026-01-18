// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#include "buyoutrepo.h"

#include <QSqlDatabase>
#include <QSqlQuery>

#include "datastore/datastore_utils.h"

constexpr const char *CREATE_BUYOUT_TABLE{R"(
CREATE TABLE IF NOT EXISTS buyouts (
    item_id         TEXT PRIMARY KEY,
    item_location   TEXT NOT NULL,
    location_type   TEXT NOT NULL
)
)"};

bool BuyoutRepo::reset()
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
