// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#pragma once

#include <QObject>

#include <optional>
#include <vector>

#include "buyout.h"

class QSqlDatabase;

class Item;
class ItemLocation;

class BuyoutRepo : public QObject {
    Q_OBJECT
public:
    explicit BuyoutRepo(QSqlDatabase &db);

    std::optional<Buyout> getBuyout(const QString &name, const QString &realm);
    std::vector<Buyout> getBuyoutList(const QString &realm,
                                      const std::optional<QString> league = {});

    bool resetRepo();
    bool ensureSchema();

public slots:
    bool saveItemBuyout(const Buyout &buyout, const Item &item);
    bool saveLocationBuyout(const Buyout &buyout, const ItemLocation &location);

private:
    QSqlDatabase &m_db;
};
