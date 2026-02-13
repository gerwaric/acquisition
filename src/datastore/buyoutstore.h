// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#pragma once

#include <QObject>
#include <QString>

#include <unordered_map>

#include "buyout.h"

class Item;
class ItemLocation;

class BuyoutStore : public QObject
{
    Q_OBJECT
public:
    explicit BuyoutStore(QStringView connName);

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
    QString m_connName;

    std::unordered_map<QString, Buyout> getBuyouts(const QString &buyout_type);

    bool saveBuyout(const Buyout &buyout,
                    const ItemLocation &location,
                    const std::optional<Item> item);

    void removeBuyout(const ItemLocation &location, const std::optional<Item> item);
};
