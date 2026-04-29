// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#pragma once

#include <QObject>

#include <unordered_map>

#include "buyout.h"

class QSqlDatabase;

class Item;
class ItemLocation;

class BuyoutRepo : public QObject {
    Q_OBJECT
public:
    explicit BuyoutRepo(QSqlDatabase &db);

    std::unordered_map<QString, Buyout> getItemBuyouts();
    std::unordered_map<QString, Buyout> getLocationBuyouts();

    void removeItemBuyout(const Item &item);
    void removeLocationBuyout(const ItemLocation &location);

    bool resetRepo();
    bool ensureSchema();

public slots:
    bool saveItemBuyout(const Buyout &buyout, const Item &item);
    bool saveLocationBuyout(const Buyout &buyout, const ItemLocation &location);

private:
    QSqlDatabase &m_db;
};
