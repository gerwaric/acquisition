// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#pragma once

#include <QObject>

#include <unordered_map>
#include <vector>

#include "buyout.h"
#include "itemlocation.h"

class QSqlDatabase;

class Item;
class ItemLocation;

enum class BuyoutSaveResult { Saved, Existing, ProtectedManual, Error };

struct ItemBuyoutWrite
{
    Buyout buyout;
    QString item_id;
    QString location_id;
    ItemLocationType location_type{ItemLocationType::STASH};
    // A manual row may only be overwritten when the plan knew it was
    // manual at generation time (the user saw it and chose import); a
    // manual price that appeared after planning is protected.
    bool allow_manual_overwrite{false};
};

struct LocationBuyoutWrite
{
    Buyout buyout;
    QString location_id;
    ItemLocationType location_type{ItemLocationType::STASH};
    bool allow_manual_overwrite{false};
};

struct BuyoutBatchSaveResult
{
    bool success{false};
    std::vector<BuyoutSaveResult> item_results;
    std::vector<BuyoutSaveResult> location_results;
    QString error;
};

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
    BuyoutBatchSaveResult saveImportBatch(const std::vector<ItemBuyoutWrite> &items,
                                          const std::vector<LocationBuyoutWrite> &locations);

    bool resetRepo();
    bool ensureSchema();

public slots:
    bool saveItemBuyout(const Buyout &buyout, const Item &item);
    bool saveLocationBuyout(const Buyout &buyout, const ItemLocation &location);

private:
    QSqlDatabase &m_db;
};
