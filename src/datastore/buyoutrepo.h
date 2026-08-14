// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#pragma once

#include <QObject>

#include <unordered_map>

#include "buyout.h"
#include "itemlocation.h"

class QSqlDatabase;

class Item;
class ItemLocation;

enum class BuyoutSaveResult { Saved, Existing, Error };

class BuyoutRepo : public QObject
{
    Q_OBJECT
public:
    explicit BuyoutRepo(QSqlDatabase &db);

    std::unordered_map<QString, Buyout> getItemBuyouts();
    std::unordered_map<QString, Buyout> getLocationBuyouts();

    bool removeItemBuyout(const Item &item);
    bool removeLocationBuyout(const ItemLocation &location);

    BuyoutSaveResult saveItemBuyout(const Buyout &buyout,
                                    const QString &item_id,
                                    const QString &location_id,
                                    ItemLocationType location_type,
                                    bool overwrite_existing);
    BuyoutSaveResult saveLocationBuyout(const Buyout &buyout,
                                        const QString &location_id,
                                        ItemLocationType location_type,
                                        bool overwrite_existing);

    bool resetRepo();
    bool ensureSchema();

public slots:
    bool saveItemBuyout(const Buyout &buyout, const Item &item);
    bool saveLocationBuyout(const Buyout &buyout, const ItemLocation &location);

private:
    QSqlDatabase &m_db;
};
