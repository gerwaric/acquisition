// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Auro

#pragma once

#include <QJsonObject>

#include <optional>

struct Buyout;
class BuyoutManager;
class Item;
class ItemLocation;

namespace control {

struct ProjectedItem
{
    QJsonObject value;
    qsizetype bytes;
};

QString LocationKind(const ItemLocation &location);
std::optional<ProjectedItem> ProjectItem(const Item &item,
                                       const ItemLocation &canonical_location,
                                       const Buyout &effective_buyout,
                                       qsizetype maximum_bytes);
QJsonObject ProjectTab(const ItemLocation &location,
                       const BuyoutManager &buyout_manager,
                       qsizetype item_count);

} // namespace control
